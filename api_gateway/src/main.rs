//! Main entry point for the API gateway.
//!
//! Configures and starts the HTTP server with session management.

use auth::{
    middleware::{auth_standard, auth_strict},
    models::SbAuthenticator,
};
use axum::Router;
use redis::{Client as RedisClient, IntoConnectionInfo, ProtocolVersion, RedisConnectionInfo};
use shared::AppState;
use std::net::SocketAddr;

use crate::health::health_check;

mod health;

/// Creates the main application router with all middleware and route configurations.
///
/// This function composes all service routers (auth, websocket, etc.) into a single application
/// router. It initializes shared state (database pool, Redis client) that is used across services.
///
/// # Panics
///
/// Panics if
/// - Database connection cannot be established
/// - Redis connection cannot be established
/// - Required environment variables are missing
async fn create_router() -> Router {
    let authenticator = SbAuthenticator::default();
    let db_pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    // initialize Redis client
    let redis_url = dotenvy::var("REDIS_URL").expect("REDIS_URL must be set in .env file");

    // configure connection to use RESP3 protocol
    let conn_info = redis_url
        .into_connection_info()
        .expect("Failed to parse REDIS_URL")
        .set_redis_settings(RedisConnectionInfo::default().set_protocol(ProtocolVersion::RESP3));
    let redis = RedisClient::open(conn_info).expect("Failed to create Redis client");

    // test the connection early to fail fast
    redis
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    let state = AppState { db_pool, redis };

    // NOTE: list all routes that need standard protection here
    let standard_prot =
        Router::new()
            .merge(ws::router(state.clone()))
            .layer(axum::middleware::from_fn_with_state(
                authenticator.clone(),
                auth_standard::<SbAuthenticator>,
            ));

    // NOTE: list all routes that need strict protection here
    let strict_prot = Router::new().layer(axum::middleware::from_fn_with_state(
        authenticator.clone(),
        auth_strict::<SbAuthenticator>,
    ));

    let health = Router::new()
        .route("/", axum::routing::get(health_check))
        .with_state(state.clone());

    // compose all service routes
    Router::new()
        .nest("/health", health)
        .nest("/auth", auth::router(authenticator.clone()))
        .merge(standard_prot)
        .merge(strict_prot)
}

/// The back-end entry point.
///
/// # Required Environment Variables
///
/// Authentication (currently uses Supabase):
/// - `SUPABASE_URL`
/// - `SUPABASE_API_KEY`
///
/// Database (PostgreSQL):
/// - `DATABASE_URL` - PostgreSQL connection string
/// - `DB_MAX_CONNECTIONS` - maximum pool size (optional, default: 15)
/// - `DB_MIN_CONNECTIONS` - minimum pool size (optional, default: 5)
///
/// Redis:
/// - `REDIS_URL` - Redis connection string
///
/// # Redis Protocol
///
/// This application uses RESP3 (Redis Serialization Protocol 3) for pub/sub messaging.
/// RESP3 provides push-based notifications which are essential for efficient WebSocket
/// broadcasting.
#[tokio::main]
async fn main() {
    // TODO: set up HTTPS (TLS) secure communication; read rustls, tokio_rustls docs

    // load .env file
    dotenvy::dotenv().expect("Unable to find .env file");

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server listening on {}", addr);
    println!("  - Health check: http://{}/health", addr);
    println!("  - Auth endpoints: http://{}/auth/*", addr);
    println!("  - WebSocket endpoint: ws://{}/ws", addr);
    println!();

    tracing_subscriber::fmt::init();
    axum::serve(listener, create_router().await).await.unwrap();
}
