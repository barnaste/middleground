//! Database Abstraction Layer
pub mod error;
pub mod queries;

pub use error::{DbError, Result};
use sqlx::postgres::PgPoolOptions;

/// Creates a new PostgreSQL connection pool.
///
/// This function reads the `DATABASE_URL` environment variable and creates
/// a connection pool with sensible defaults for production use.
///
/// # Environment Variables
///
/// - `DATABASE_URL` - required; PostgreSQL connection string
/// - `DB_MAX_CONNECTIONS` - optional; maximum number of connections in the pool (default: 10)
/// - `DB_MIN_CONNECTIONS` - optional; minimum number of connections in the pool (default: 5)
///
/// # Errors
///
/// Returns a [`DbError`] if:
/// - `DATABASE_URL` environment variable is not set or is invalid
/// - Unable to connect to the database
///
/// # Example
///
/// ```rust,no_run
/// use db::create_pool;
/// use db::error::DbError;
///
/// #[tokio::main]
/// async fn main() -> Result<(), DbError> {
///     let pool = create_pool().await?;
///     println!("Connected to database!");
///     Ok(())
/// }
/// ```
pub async fn create_pool() -> Result<sqlx::PgPool> {
    let database_url = dotenvy::var("DATABASE_URL")
        .map_err(|_| DbError::Configuration("DATABASE_URL must be set in .env file".into()))?;

    let max_connections = dotenvy::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let min_connections = dotenvy::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .connect(&database_url)
        .await
        .map_err(DbError::Connection)
}
