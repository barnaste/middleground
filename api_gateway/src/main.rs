use std::net::SocketAddr;

use axum::{
    Router,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

// NOTE: PASSING INFORMATION TO
//       for GET, use URL (query) parameters
//       for DELETE, use URL (query) parameters
//       for POST or PUT, send representation of object in body
// NOTE: PUT requests tend only to be implemented at endpoints where GET is also
//       implemented, so that there is the expetation that when you PUT {obj}, calling
//       GET afterwards will return {obj}
fn create_router() -> Router {
    let session_store = MemoryStore::default();
    // no authentication : a user account is not attached to the client
    // pending authentication : an unverified user account is attached to the client
    // authentication : a verified user account is attached to the client

    // these will require no authentication to access
    let public_auth_routes = Router::new()
        .route("/register", post(auth::service::register))
        .route("/login", post(root));

    // these will require pending authentication to access
    let pending_auth_routes = Router::new()
        .route("/verify-email", post(root))
        .route("/resend-verification", post(root));

    let auth_routes = Router::new()
        .route("/logout", post(root))
        // .layer(...)
        .merge(public_auth_routes)
        .merge(pending_auth_routes);

    // TODO: rate limiter middleware on *all* API calls to prevent abuse

    Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes)
        .fallback(fallback)
        .layer(auth::layer::session_manager(session_store))
}



#[tokio::main]
async fn main() {
    // OBJECTIVE: set up HTTPS (TLS) secure communication (read rustls, tokio_rustls docs)

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, create_router()).await.unwrap();
}

// TEMP SETUP SO THAT SESSION COOKIES ARE TRANSMITTED
const COUNTER_KEY: &str = "counter";

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Counter(usize);

async fn root(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get(COUNTER_KEY).await.unwrap().unwrap_or_default();
    session.insert(COUNTER_KEY, counter.0 + 1).await.unwrap();
    format!("Current count: {}", counter.0)
}

async fn fallback() -> Redirect {
    // TEMPORARY
    Redirect::to("/")
}
