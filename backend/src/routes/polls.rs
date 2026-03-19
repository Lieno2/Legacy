use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    routes::AppState,
};

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct EventIdQuery {
    pub event_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CreatePollRequest {
    pub event_id: i64,
    pub question: String,
    pub choices: Vec<String>,
    #[serde(default)]
    pub allow_multiple: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct AnswerPollRequest {
    pub poll_id: i64,
    /// List of chosen choice IDs (one or many depending on allow_multiple)
    pub choice_ids: Vec<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct PollChoice {
    pub id: i64,
    pub label: String,
    pub position: i32,
    pub answer_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PollResponse {
    pub id: i64,
    pub event_id: i64,
    pub question: String,
    pub allow_multiple: bool,
    pub choices: Vec<PollChoice>,
    /// The choice IDs the current user picked
    pub my_choice_ids: Vec<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VoterEntry {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChoiceVoters {
    pub choice_id: i64,
    pub label: String,
    pub voters: Vec<VoterEntry>,
}

// ── GET /api/polls?event_id= ─────────────────────────────────────────────────

#[utoipa::path(get, path = "/api/polls", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, body = Option<PollResponse>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_poll(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<Option<PollResponse>>> {
    let poll_row = sqlx::query_as::<_, (i64, i64, String, bool)>(
        r#"SELECT id, "eventId", question, "allowMultiple" FROM "EventPolls" WHERE "eventId" = $1"#,
    )
    .bind(q.event_id)
    .fetch_optional(&state.db)
    .await?;

    let Some((poll_id, event_id, question, allow_multiple)) = poll_row else {
        return Ok(Json(None));
    };

    let choices = sqlx::query_as::<_, PollChoice>(
        r#"SELECT
             c.id, c.label, c.position,
             COUNT(a."choiceId")::BIGINT AS answer_count
           FROM "EventPollChoices" c
           LEFT JOIN "EventPollAnswers" a ON a."choiceId" = c.id
           WHERE c."pollId" = $1
           GROUP BY c.id, c.label, c.position
           ORDER BY c.position ASC"#,
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await?;

    let my_choice_ids = sqlx::query_scalar::<_, i64>(
        r#"SELECT "choiceId" FROM "EventPollAnswers" WHERE "pollId" = $1 AND "userId" = $2"#,
    )
    .bind(poll_id)
    .bind(&user.0.sub)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(Some(PollResponse {
        id: poll_id,
        event_id,
        question,
        allow_multiple,
        choices,
        my_choice_ids,
    })))
}

// ── GET /api/polls/voters?event_id= ─────────────────────────────────────────

#[utoipa::path(get, path = "/api/polls/voters", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, body = Vec<ChoiceVoters>),
        (status = 404, description = "Poll not found"),
    )
)]
pub async fn get_voters(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<Vec<ChoiceVoters>>> {
    let poll_id = sqlx::query_scalar::<_, i64>(
        r#"SELECT id FROM "EventPolls" WHERE "eventId" = $1"#,
    )
    .bind(q.event_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    // Get choices ordered by position
    let choices = sqlx::query_as::<_, (i64, String)>(
        r#"SELECT id, label FROM "EventPollChoices" WHERE "pollId" = $1 ORDER BY position ASC"#,
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await?;

    // Get all answers with usernames in one query
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        r#"SELECT a."choiceId", a."userId", u.username
           FROM "EventPollAnswers" a
           JOIN "Users" u ON u.id = a."userId"
           WHERE a."pollId" = $1"#,
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await?;

    let result = choices.into_iter().map(|(cid, label)| {
        let voters = rows.iter()
            .filter(|(choice_id, _, _)| *choice_id == cid)
            .map(|(_, uid, uname)| VoterEntry { user_id: uid.clone(), username: uname.clone() })
            .collect();
        ChoiceVoters { choice_id: cid, label, voters }
    }).collect();

    Ok(Json(result))
}

// ── POST /api/polls ──────────────────────────────────────────────────────────

#[utoipa::path(post, path = "/api/polls", tag = "Polls",
    security(("bearer_auth" = [])),
    request_body = CreatePollRequest,
    responses(
        (status = 200, body = PollResponse),
        (status = 403, description = "Not the event owner"),
    )
)]
pub async fn upsert_poll(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreatePollRequest>,
) -> Result<Json<PollResponse>> {
    let question = body.question.trim().to_string();
    if question.is_empty() {
        return Err(AppError::BadRequest("Question is required".into()));
    }
    let choices: Vec<String> = body.choices.iter()
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();
    if choices.len() < 2 {
        return Err(AppError::BadRequest("At least 2 choices are required".into()));
    }
    if choices.len() > 6 {
        return Err(AppError::BadRequest("Maximum 6 choices allowed".into()));
    }

    let owner_id = sqlx::query_scalar::<_, String>(
        r#"SELECT "createdBy" FROM "Events" WHERE id = $1"#,
    )
    .bind(body.event_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if owner_id != user.0.sub {
        return Err(AppError::Forbidden);
    }

    sqlx::query(r#"DELETE FROM "EventPolls" WHERE "eventId" = $1"#)
        .bind(body.event_id)
        .execute(&state.db)
        .await?;

    let poll_id = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO "EventPolls" ("eventId", question, "allowMultiple") VALUES ($1, $2, $3) RETURNING id"#,
    )
    .bind(body.event_id)
    .bind(&question)
    .bind(body.allow_multiple)
    .fetch_one(&state.db)
    .await?;

    for (i, label) in choices.iter().enumerate() {
        sqlx::query(
            r#"INSERT INTO "EventPollChoices" ("pollId", label, position) VALUES ($1, $2, $3)"#,
        )
        .bind(poll_id)
        .bind(label)
        .bind(i as i32)
        .execute(&state.db)
        .await?;
    }

    let poll_choices = sqlx::query_as::<_, PollChoice>(
        r#"SELECT id, label, position, 0::BIGINT AS answer_count
           FROM "EventPollChoices" WHERE "pollId" = $1 ORDER BY position ASC"#,
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(PollResponse {
        id: poll_id,
        event_id: body.event_id,
        question,
        allow_multiple: body.allow_multiple,
        choices: poll_choices,
        my_choice_ids: vec![],
    }))
}

// ── DELETE /api/polls?event_id= ──────────────────────────────────────────────

#[utoipa::path(delete, path = "/api/polls", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 403, description = "Not the event owner"),
    )
)]
pub async fn delete_poll(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<serde_json::Value>> {
    let owner_id = sqlx::query_scalar::<_, String>(
        r#"SELECT "createdBy" FROM "Events" WHERE id = $1"#,
    )
    .bind(q.event_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if owner_id != user.0.sub {
        return Err(AppError::Forbidden);
    }

    sqlx::query(r#"DELETE FROM "EventPolls" WHERE "eventId" = $1"#)
        .bind(q.event_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ── POST /api/polls/answer ───────────────────────────────────────────────────

#[utoipa::path(post, path = "/api/polls/answer", tag = "Polls",
    security(("bearer_auth" = [])),
    request_body = AnswerPollRequest,
    responses(
        (status = 200, description = "Answer saved"),
        (status = 404, description = "Poll or choice not found"),
    )
)]
pub async fn answer_poll(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<AnswerPollRequest>,
) -> Result<Json<serde_json::Value>> {
    if body.choice_ids.is_empty() {
        return Err(AppError::BadRequest("At least one choice is required".into()));
    }

    // Verify all choices belong to this poll
    let valid_count = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM "EventPollChoices" WHERE "pollId" = $1 AND id = ANY($2)"#,
    )
    .bind(body.poll_id)
    .bind(&body.choice_ids)
    .fetch_one(&state.db)
    .await?;

    if valid_count != body.choice_ids.len() as i64 {
        return Err(AppError::NotFound);
    }

    // Remove previous answers for this user+poll, then insert fresh
    sqlx::query(
        r#"DELETE FROM "EventPollAnswers" WHERE "pollId" = $1 AND "userId" = $2"#,
    )
    .bind(body.poll_id)
    .bind(&user.0.sub)
    .execute(&state.db)
    .await?;

    for cid in &body.choice_ids {
        sqlx::query(
            r#"INSERT INTO "EventPollAnswers" ("pollId", "userId", "choiceId") VALUES ($1, $2, $3)"#,
        )
        .bind(body.poll_id)
        .bind(&user.0.sub)
        .bind(cid)
        .execute(&state.db)
        .await?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
