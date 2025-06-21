use axum::{Router, routing::post};
use managers::sb_manager::SbManager;
use supabase_auth::models::AuthClient;

mod dto;
mod handlers;
mod jwt;
mod managers;

pub mod middleware;

/// Set up the authentication router. The auth service currently uses
/// supabase_auth, and thus requires the following environment variables
/// to be set *and loaded* prior to invoking this function:
///     SUPABASE_URL, SUPABASE_API_KEY, SUPABASE_JWT_SECRET
pub fn router() -> Router {
    let state = SbManager {
        client: AuthClient::new_from_env().unwrap(),
    };

    Router::new()
        .route("/send-otp", post(handlers::send_otp::<SbManager>))
        .route("/verify-otp", post(handlers::verify_otp::<SbManager>))
        .route("/logout", post(handlers::logout::<SbManager>))
        .route("/refresh", post(handlers::refresh_token::<SbManager>))
        .with_state(state)
}
