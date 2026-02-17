//! Health check endpoint and handlers.
//!
//! Provides a `/health` endpoint that verifies the operational status of critical dependencies:
//! - Database connection pool
//! - Redis connection
//!
//! Returns HTTP 200 if all services are healthy, HTTP 503 if any service is unavailable.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use shared::AppState;

/// Health check response structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall system status
    pub status: String,
    /// Individual service health
    pub services: ServiceHealth,
}

/// Individual service health status.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub database: String,
    pub redis: String,
}

/// Health check handler.
///
/// Performs lightweight checks on critical dependencies:
/// - Database: Executes `SELECT 1` to verify connection
/// - Redis: Executes `PING` command to verify connection
///
/// Returns:
/// - `200 OK` with health status if all services are operational
/// - `503 Service Unavailable` if any service is down
///
/// # Example Response (Healthy)
///
/// ```json
/// {
///   "status": "healthy",
///   "services": {
///     "database": "healthy",
///     "redis": "healthy"
///   }
/// }
/// ```
///
/// # Example Response (Unhealthy)
///
/// ```json
/// {
///   "status": "unhealthy",
///   "services": {
///     "database": "healthy",
///     "redis": "Connection refused"
///   }
/// }
/// ```
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let mut db_status = "healthy".to_string();
    let mut redis_status = "healthy".to_string();
    let mut overall_healthy = true;

    // Check database connection
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(_) => {
            tracing::debug!("Database health check: OK");
        }
        Err(e) => {
            db_status = format!("unhealthy: {}", e);
            overall_healthy = false;
            tracing::error!("Database health check failed: {}", e);
        }
    }

    // Check Redis connection
    match state.redis.get_multiplexed_async_connection().await {
        Ok(mut conn) => match redis::cmd("PING").query_async::<()>(&mut conn).await {
            Ok(_) => {
                tracing::debug!("Redis health check: OK");
            }
            Err(e) => {
                redis_status = format!("unhealthy: {}", e);
                overall_healthy = false;
                tracing::error!("Redis PING failed: {}", e);
            }
        },
        Err(e) => {
            redis_status = format!("unhealthy: {}", e);
            overall_healthy = false;
            tracing::error!("Redis connection failed: {}", e);
        }
    }

    let status_code = if overall_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status: if overall_healthy {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        services: ServiceHealth {
            database: db_status,
            redis: redis_status,
        },
    };

    (status_code, Json(response))
}
