use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::{AuthUser, AdminUser},
    error::{AppError, Result},
    routes::AppState,
};

// ── Types ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct EventIdQuery {
    pub event_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct PollIdQuery {
    pub poll_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct TemplateIdQuery {
    pub id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CreatePollRequest {
    pub event_id: i64,
    pub question: String,
    pub poll_type: Option<String>, // 'choice' | 'text' | 'rating' | 'yesno' | 'date'
    pub choices: Option<Vec<String>>,
    #[serde(default)]
    pub allow_multiple: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct AnswerPollRequest {
    pub poll_id: i64,
    pub choice_ids: Option<Vec<i64>>,   // for 'choice' and 'yesno' and 'date'
    pub text_answer: Option<String>,     // for 'text'
    pub rating: Option<i16>,             // for 'rating' (1-5)
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub poll_type: String,
    pub question: Option<String>,
    pub choices: Option<Vec<String>>,
    pub allow_multiple: Option<bool>,
    pub global: Option<bool>, // admin only
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
    pub poll_type: String,
    pub allow_multiple: bool,
    pub choices: Vec<PollChoice>,
    pub my_choice_ids: Vec<i64>,
    pub my_text_answer: Option<String>,
    pub my_rating: Option<i16>,
    pub avg_rating: Option<f64>,
    pub rating_count: i64,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct TextAnswer {
    pub user_id: String,
    pub username: String,
    pub answer: String,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct PollTemplate {
    pub id: i64,
    pub name: String,
    pub poll_type: String,
    pub question: Option<String>,
    pub choices: Option<serde_json::Value>,
    pub allow_multiple: bool,
    pub global: bool,
    pub created_by: Option<String>,
}

// ── GET /api/polls?event_id= ────────────────────────────────────────────────────────────

#[utoipa::path(get, path = "/api/polls", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses((status = 200, body = Option<PollResponse>), (status = 401))
)]
pub async fn get_poll(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<Option<PollResponse>>> {
    let poll_row = sqlx::query_as::<_, (i64, i64, String, String, bool)>(
        r#"SELECT id, "eventId", question, poll_type, "allowMultiple"
           FROM "EventPolls" WHERE "eventId" = $1"#,
    )
        .bind(q.event_id)
        .fetch_optional(&state.db)
        .await?;

    let Some((poll_id, event_id, question, poll_type, allow_multiple)) = poll_row else {
        return Ok(Json(None));
    };

    let choices = sqlx::query_as::<_, PollChoice>(
        r#"SELECT c.id, c.label, c.position,
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

    let my_text_answer = sqlx::query_scalar::<_, String>(
        r#"SELECT answer FROM "EventPollTextAnswers" WHERE "pollId" = $1 AND "userId" = $2"#,
    )
        .bind(poll_id)
        .bind(&user.0.sub)
        .fetch_optional(&state.db)
        .await?;

    let my_rating = sqlx::query_scalar::<_, i16>(
        r#"SELECT rating FROM "EventPollRatings" WHERE "pollId" = $1 AND "userId" = $2"#,
    )
        .bind(poll_id)
        .bind(&user.0.sub)
        .fetch_optional(&state.db)
        .await?;

    let (avg_rating, rating_count) = sqlx::query_as::<_, (Option<f64>, i64)>(
        r#"SELECT AVG(rating::float8), COUNT(*)::BIGINT FROM "EventPollRatings" WHERE "pollId" = $1"#,
    )
        .bind(poll_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(Some(PollResponse {
        id: poll_id,
        event_id,
        question,
        poll_type,
        allow_multiple,
        choices,
        my_choice_ids,
        my_text_answer,
        my_rating,
        avg_rating,
        rating_count,
    })))
}

// ── POST /api/polls ───────────────────────────────────────────────────────────────────────

#[utoipa::path(post, path = "/api/polls", tag = "Polls",
    security(("bearer_auth" = [])), request_body = CreatePollRequest,
    responses((status = 200, body = PollResponse), (status = 403))
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

    let poll_type = body.poll_type.as_deref().unwrap_or("choice").to_string();

    let choices: Vec<String> = match poll_type.as_str() {
        "choice" | "date" => {
            let c: Vec<String> = body.choices.unwrap_or_default()
                .iter().map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()).collect();
            if c.len() < 2 { return Err(AppError::BadRequest("At least 2 choices required".into())); }
            if c.len() > 10 { return Err(AppError::BadRequest("Maximum 10 choices allowed".into())); }
            c
        }
        "yesno" => vec!["Yes".into(), "No".into()],
        "text" | "rating" => vec![],
        _ => return Err(AppError::BadRequest("Invalid poll type".into())),
    };

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
        r#"INSERT INTO "EventPolls" ("eventId", question, "allowMultiple", poll_type)
           VALUES ($1, $2, $3, $4) RETURNING id"#,
    )
        .bind(body.event_id)
        .bind(&question)
        .bind(body.allow_multiple)
        .bind(&poll_type)
        .fetch_one(&state.db)
        .await?;

    for (i, label) in choices.iter().enumerate() {
        sqlx::query(
            r#"INSERT INTO "EventPollChoices" ("pollId", label, position) VALUES ($1, $2, $3)"#,
        )
            .bind(poll_id).bind(label).bind(i as i32)
            .execute(&state.db).await?;
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
        poll_type,
        allow_multiple: body.allow_multiple,
        choices: poll_choices,
        my_choice_ids: vec![],
        my_text_answer: None,
        my_rating: None,
        avg_rating: None,
        rating_count: 0,
    }))
}

// ── DELETE /api/polls?event_id= ──────────────────────────────────────────────────────────────

#[utoipa::path(delete, path = "/api/polls", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses((status = 200), (status = 403))
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

    if owner_id != user.0.sub { return Err(AppError::Forbidden); }

    sqlx::query(r#"DELETE FROM "EventPolls" WHERE "eventId" = $1"#)
        .bind(q.event_id).execute(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ── POST /api/polls/answer ───────────────────────────────────────────────────────────────────

#[utoipa::path(post, path = "/api/polls/answer", tag = "Polls",
    security(("bearer_auth" = [])), request_body = AnswerPollRequest,
    responses((status = 200), (status = 400), (status = 404))
)]
pub async fn answer_poll(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<AnswerPollRequest>,
) -> Result<Json<serde_json::Value>> {
    // Fetch poll type AND allow_multiple in one query
    let poll_row = sqlx::query_as::<_, (String, bool)>(
        r#"SELECT poll_type, "allowMultiple" FROM "EventPolls" WHERE id = $1"#,
    )
        .bind(body.poll_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let (poll_type, allow_multiple) = poll_row;

    match poll_type.as_str() {
        "text" => {
            let answer = body.text_answer.as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BadRequest("Text answer is required".into()))?;

            sqlx::query(
                r#"INSERT INTO "EventPollTextAnswers" ("pollId", "userId", answer)
                   VALUES ($1, $2, $3)
                   ON CONFLICT ("pollId", "userId") DO UPDATE SET answer = EXCLUDED.answer"#,
            )
                .bind(body.poll_id).bind(&user.0.sub).bind(&answer)
                .execute(&state.db).await?;
        }
        "rating" => {
            let rating = body.rating
                .ok_or_else(|| AppError::BadRequest("Rating is required".into()))?;
            if !(1..=5).contains(&rating) {
                return Err(AppError::BadRequest("Rating must be between 1 and 5".into()));
            }
            sqlx::query(
                r#"INSERT INTO "EventPollRatings" ("pollId", "userId", rating)
                   VALUES ($1, $2, $3)
                   ON CONFLICT ("pollId", "userId") DO UPDATE SET rating = EXCLUDED.rating"#,
            )
                .bind(body.poll_id).bind(&user.0.sub).bind(rating)
                .execute(&state.db).await?;
        }
        "choice" | "yesno" | "date" => {
            let choice_ids = body.choice_ids.as_ref()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| AppError::BadRequest("At least one choice is required".into()))?;

            // Enforce allow_multiple server-side
            if !allow_multiple && choice_ids.len() > 1 {
                return Err(AppError::BadRequest(
                    "This poll does not allow multiple selections".into()
                ));
            }

            let valid_count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM "EventPollChoices" WHERE "pollId" = $1 AND id = ANY($2)"#,
            )
                .bind(body.poll_id).bind(choice_ids)
                .fetch_one(&state.db).await?;

            if valid_count != choice_ids.len() as i64 {
                return Err(AppError::NotFound);
            }

            sqlx::query(
                r#"DELETE FROM "EventPollAnswers" WHERE "pollId" = $1 AND "userId" = $2"#,
            )
                .bind(body.poll_id).bind(&user.0.sub)
                .execute(&state.db).await?;

            for cid in choice_ids {
                sqlx::query(
                    r#"INSERT INTO "EventPollAnswers" ("pollId", "userId", "choiceId") VALUES ($1, $2, $3)"#,
                )
                    .bind(body.poll_id).bind(&user.0.sub).bind(cid)
                    .execute(&state.db).await?;
            }
        }
        _ => return Err(AppError::BadRequest("Unknown poll type".into())),
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

// ── GET /api/polls/voters?event_id= ───────────────────────────────────────────────────

#[utoipa::path(get, path = "/api/polls/voters", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses((status = 200, body = Vec<ChoiceVoters>), (status = 404))
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

    let choices = sqlx::query_as::<_, (i64, String)>(
        r#"SELECT id, label FROM "EventPollChoices" WHERE "pollId" = $1 ORDER BY position ASC"#,
    )
        .bind(poll_id).fetch_all(&state.db).await?;

    let rows = sqlx::query_as::<_, (i64, String, String)>(
        r#"SELECT a."choiceId", a."userId", u.username
           FROM "EventPollAnswers" a
           JOIN "Users" u ON u.id = a."userId"
           WHERE a."pollId" = $1"#,
    )
        .bind(poll_id).fetch_all(&state.db).await?;

    let result = choices.into_iter().map(|(cid, label)| {
        let voters = rows.iter()
            .filter(|(choice_id, _, _)| *choice_id == cid)
            .map(|(_, uid, uname)| VoterEntry { user_id: uid.clone(), username: uname.clone() })
            .collect();
        ChoiceVoters { choice_id: cid, label, voters }
    }).collect();

    Ok(Json(result))
}

// ── GET /api/polls/text-answers?event_id= ─────────────────────────────────────────────

#[utoipa::path(get, path = "/api/polls/text-answers", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses((status = 200, body = Vec<TextAnswer>), (status = 404))
)]
pub async fn get_text_answers(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<Vec<TextAnswer>>> {
    let poll_id = sqlx::query_scalar::<_, i64>(
        r#"SELECT id FROM "EventPolls" WHERE "eventId" = $1"#,
    )
        .bind(q.event_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let rows = sqlx::query_as::<_, (String, String, String)>(
        r#"SELECT a."userId", u.username, a.answer
           FROM "EventPollTextAnswers" a
           JOIN "Users" u ON u.id = a."userId"
           WHERE a."pollId" = $1
           ORDER BY a."answeredAt" ASC"#,
    )
        .bind(poll_id).fetch_all(&state.db).await?;

    let result = rows.into_iter().map(|(uid, uname, answer)| TextAnswer {
        user_id: uid,
        username: uname,
        answer,
    }).collect();

    Ok(Json(result))
}

// ── GET /api/polls/templates ───────────────────────────────────────────────────────────────────

#[utoipa::path(get, path = "/api/polls/templates", tag = "Polls",
    security(("bearer_auth" = [])),
    responses((status = 200, body = Vec<PollTemplate>))
)]
pub async fn list_templates(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PollTemplate>>> {
    let templates = sqlx::query_as::<_, PollTemplate>(
        r#"SELECT id, name, poll_type, question, choices, allow_multiple, global,
           "createdBy" AS created_by
           FROM "PollTemplates"
           WHERE global = true OR "createdBy" = $1
           ORDER BY global DESC, "createdAt" ASC"#,
    )
        .bind(&user.0.sub)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(templates))
}

// ── POST /api/polls/templates ───────────────────────────────────────────────────────────────────

#[utoipa::path(post, path = "/api/polls/templates", tag = "Polls",
    security(("bearer_auth" = [])), request_body = CreateTemplateRequest,
    responses((status = 200, body = PollTemplate))
)]
pub async fn create_template(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateTemplateRequest>,
) -> Result<Json<PollTemplate>> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Template name is required".into()));
    }

    let is_global = body.global.unwrap_or(false);
    if is_global && user.0.perms < 999 {
        return Err(AppError::Forbidden);
    }

    let choices_json: Option<serde_json::Value> = body.choices.map(|c| {
        serde_json::json!(c.iter().map(|l| serde_json::json!({"label": l})).collect::<Vec<_>>())
    });

    let template = sqlx::query_as::<_, PollTemplate>(
        r#"INSERT INTO "PollTemplates" (name, poll_type, question, choices, allow_multiple, global, "createdBy")
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, name, poll_type, question, choices, allow_multiple, global,
           "createdBy" AS created_by"#,
    )
        .bind(&name)
        .bind(&body.poll_type)
        .bind(&body.question)
        .bind(&choices_json)
        .bind(body.allow_multiple.unwrap_or(false))
        .bind(is_global)
        .bind(&user.0.sub)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(template))
}

// ── DELETE /api/polls/templates?id= ────────────────────────────────────────────────────────────

#[utoipa::path(delete, path = "/api/polls/templates", tag = "Polls",
    security(("bearer_auth" = [])),
    params(("id" = i64, Query, description = "Template ID")),
    responses((status = 200), (status = 403), (status = 404))
)]
pub async fn delete_template(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<TemplateIdQuery>,
) -> Result<Json<serde_json::Value>> {
    let row = sqlx::query_as::<_, (String, bool)>(
        r#"SELECT "createdBy", global FROM "PollTemplates" WHERE id = $1"#,
    )
        .bind(q.id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let (created_by, is_global) = row;

    if is_global && user.0.perms < 999 {
        return Err(AppError::Forbidden);
    }
    if !is_global && created_by != user.0.sub {
        return Err(AppError::Forbidden);
    }

    sqlx::query(r#"DELETE FROM "PollTemplates" WHERE id = $1"#)
        .bind(q.id).execute(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
