//! Main entry point for the API gateway.
//!
//! Configures and starts the HTTP server with session management.

use auth::models::SbAuthenticator;
use axum::Router;
use std::net::SocketAddr;

/// Creates the main application router with all middleware and route configurations.
async fn create_router() -> Router {
    let authenticator = SbAuthenticator::default();
    let _pool = db::create_pool().await.unwrap();

    Router::new()
        .nest("/auth", auth::router(authenticator.clone()))
    // TODO: rate limiting
}

/// The back-end entry point.
/// The auth service currently uses supabase_auth, and thus
/// requires the following environment variables to be set:
///     SUPABASE_URL, SUPABASE_API_KEY, SUPABASE_JWT_SECRET
/// The db in use is set up using the environment variables:
///     DATABASE_URL, DATABASE_MAX_CON, DATABASE_MIN_CON
#[tokio::main]
async fn main() {
    // TODO: set up HTTPS (TLS) secure communication; read rustls, tokio_rustls docs

    // load .env file
    dotenvy::dotenv().expect("Unable to find .env file");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server listening on {}", addr);
    axum::serve(listener, create_router().await).await.unwrap();
}
