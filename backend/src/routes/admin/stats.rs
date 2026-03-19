use axum::{extract::State, Json};

use crate::{
    auth::AdminUser,
    error::Result,
    routes::AppState,
};
use super::{MonthStat, ActiveUser, RsvpBreakdown, StatsResponse};

#[utoipa::path(get, path = "/api/admin/stats", tag = "Admin",
    security(("bearer_auth" = [])),
    responses((status = 200, body = StatsResponse)))]
pub async fn get_stats(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>> {
    let events_per_month = sqlx::query_as::<_, MonthStat>(
        r#"SELECT TO_CHAR(date,'YYYY-MM') AS month, COUNT(*)::BIGINT AS count
           FROM "Events"
           WHERE date >= NOW() - INTERVAL '12 months'
           GROUP BY month ORDER BY month ASC"#
    ).fetch_all(&state.db).await?;

    let most_active_users = sqlx::query_as::<_, ActiveUser>(
        r#"SELECT u.username, COUNT(em."userId")::BIGINT AS rsvp_count
           FROM "EventMembers" em
           JOIN "Users" u ON em."userId"=u.id
           WHERE em.status IN ('going','late')
           GROUP BY u.username ORDER BY rsvp_count DESC LIMIT 5"#
    ).fetch_all(&state.db).await?;

    let going     = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='going'"#).fetch_one(&state.db).await?;
    let late      = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='late'"#).fetch_one(&state.db).await?;
    let not_going = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='not_going'"#).fetch_one(&state.db).await?;
    let invited   = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventInvites""#).fetch_one(&state.db).await?;

    Ok(Json(StatsResponse {
        events_per_month,
        most_active_users,
        rsvp_breakdown: RsvpBreakdown { going, late, not_going, invited },
    }))
}
