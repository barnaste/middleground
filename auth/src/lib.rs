use axum::{Router, middleware as axum_middleware, routing::post};

pub mod middleware;
pub mod services;

mod models;

pub fn router() -> Router {
    // // these will require full authentication to access
    // let full_auth_routes = Router::new()
    //     // .route("/logout", post(root))
    //     .layer(axum_middleware::from_fn(middleware::require_auth));
    //
    // // these will require pending authentication to access
    // let pending_auth_routes = Router::new()
    //     // .route("/verify-email", post(root))
    //     // .route("/resend-verification", post(root))
    //     .layer(axum_middleware::from_fn(middleware::require_pending));
    //
    Router::new()
        // .route("/register", post(services::register))
        // // .route("/login", post(root))
        // .merge(pending_auth_routes)
        // .merge(full_auth_routes)
}
