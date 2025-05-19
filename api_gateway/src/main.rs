use axum::{response::Redirect, Router, routing::{post, get}};

// NOTE: PASSING INFORMATION TO 
//       for GET, URL (query) parameters
//       for DELETE, URL (query) parameters
//       for POST or PUT, send representation of object in body

#[tokio::main]
async fn main() {
    // OBJECTIVE 1: set up sessions as middleware for all but login (read tower-sessions, tower, tower_http docs)
    // OBJECTIVE 2: set up HTTPS (TLS) secure communication (read rustls, tokio_rustls docs)

    // TODO: set up middleware for authentication for all but {register, login}
    let auth_routes = Router::new()
        .route("/register", post(auth::register));

    let app = Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes)
        .fallback(fallback);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn fallback() -> Redirect {
    // TEMPORARY
    Redirect::to("/")
}