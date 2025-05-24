//! Main entry point for the API gateway.
//!
//! Configures and starts the HTTP server with session management.

use axum::Router;
use std::net::SocketAddr;
use tower_sessions::MemoryStore;

/// Creates the main application router with all middleware and route configurations
fn create_router() -> Router {
    Router::new()
        .nest("/auth", auth::router())
        .layer(auth::middleware::create_session_layer(
            MemoryStore::default(),
        ))
    // TODO: rate limiting
}

// TODO: set up HTTPS (TLS) secure communication; read rustls, tokio_rustls docs
#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server listening on {}", addr);
    axum::serve(listener, create_router()).await.unwrap();
}
