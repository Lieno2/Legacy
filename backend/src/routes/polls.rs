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
}

#[derive(Deserialize, ToSchema)]
pub struct AnswerPollRequest {
    pub poll_id: i64,
    pub choice_id: i64,
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
    pub choices: Vec<PollChoice>,
    /// The choice_id the current user picked, if any
    pub my_choice_id: Option<i64>,
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
    // Fetch the poll row
    let poll_row = sqlx::query!(
        r#"SELECT id, "eventId" AS event_id, question FROM "EventPolls" WHERE "eventId" = $1"#,
        q.event_id
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(poll) = poll_row else {
        return Ok(Json(None));
    };

    // Fetch choices with answer counts
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
    .bind(poll.id)
    .fetch_all(&state.db)
    .await?;

    // Current user's answer
    let my_choice_id = sqlx::query_scalar::<_, i64>(
        r#"SELECT "choiceId" FROM "EventPollAnswers" WHERE "pollId" = $1 AND "userId" = $2"#,
    )
    .bind(poll.id)
    .bind(&user.0.sub)
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(Some(PollResponse {
        id: poll.id,
        event_id: poll.event_id,
        question: poll.question,
        choices,
        my_choice_id,
    })))
}

// ── POST /api/polls ──────────────────────────────────────────────────────────
// Creates or replaces the poll for an event (event owner only)

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

    // Verify the caller owns the event
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

    // Delete existing poll (cascade removes choices + answers)
    sqlx::query!(r#"DELETE FROM "EventPolls" WHERE "eventId" = $1"#, body.event_id)
        .execute(&state.db)
        .await?;

    // Insert new poll
    let poll_id = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO "EventPolls" ("eventId", question) VALUES ($1, $2) RETURNING id"#,
    )
    .bind(body.event_id)
    .bind(&question)
    .fetch_one(&state.db)
    .await?;

    // Insert choices
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

    // Return the freshly created poll
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
        choices: poll_choices,
        my_choice_id: None,
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

    sqlx::query!(r#"DELETE FROM "EventPolls" WHERE "eventId" = $1"#, q.event_id)
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
    // Verify the choice belongs to this poll
    let valid = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(SELECT 1 FROM "EventPollChoices" WHERE id = $1 AND "pollId" = $2)"#,
    )
    .bind(body.choice_id)
    .bind(body.poll_id)
    .fetch_one(&state.db)
    .await?;

    if !valid {
        return Err(AppError::NotFound);
    }

    // Upsert answer
    sqlx::query(
        r#"INSERT INTO "EventPollAnswers" ("pollId", "userId", "choiceId")
           VALUES ($1, $2, $3)
           ON CONFLICT ("pollId", "userId") DO UPDATE SET "choiceId" = EXCLUDED."choiceId", "answeredAt" = NOW()"#,
    )
    .bind(body.poll_id)
    .bind(&user.0.sub)
    .bind(body.choice_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
