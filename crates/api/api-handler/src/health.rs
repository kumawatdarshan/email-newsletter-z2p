use axum::{extract::State, http::StatusCode};
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::Repository;
use tower_sessions_redis_store::fred::prelude::ClientLike;

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum HealthError {
    #[error("Redis Server is unavailable.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    RedisUnavailable,

    #[error("Redis Server is available but is down.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    RedisDown,

    #[error("Failed to make connection to SQLite DB.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    SQliteDown,
}

pub(crate) async fn health_check(
    State(repo): State<Repository>,
    State(redis_pool): State<tower_sessions_redis_store::fred::prelude::Pool>,
) -> Result<StatusCode, HealthError> {
    repo.ping().await.map_err(|_| HealthError::SQliteDown)?;

    let redis_res: String = redis_pool
        .ping(None)
        .await
        .map_err(|_| HealthError::RedisUnavailable)?;

    if redis_res != "PONG" {
        return Err(HealthError::RedisDown);
    }

    Ok(StatusCode::OK)
}
