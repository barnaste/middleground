use crate::dto::*;
use crate::jwt;
use crate::models::Authenticator;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};

pub async fn send_otp<A: Authenticator>(
    State(state): State<A>,
    Json(payload): Json<SendOtpRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.send_otp(&payload.email).await {
        Ok(_) => Ok(Json(MessageResponse {
            message: "OTP sent to your email. Please check your inbox".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("OTP transmission failed: {}", e),
            }),
        )),
    }
}

pub async fn verify_otp<A: Authenticator>(
    State(state): State<A>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.verify_otp(&payload.email, &payload.token).await {
        Ok(session) => Ok(Json(session.into())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("OTP verification failed: {}", e),
            }),
        )),
    }
}

pub async fn logout<A: Authenticator>(
    State(state): State<A>,
    headers: HeaderMap,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let bearer_token = match jwt::extract_jwt_from_headers(&headers) {
        Ok(token) => token,
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: format!("Logout failed: {}", e),
                }),
            ));
        }
    };

    match state.logout(&bearer_token).await {
        Ok(_) => Ok(Json(MessageResponse {
            message: "Successfully logged out".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Logout failed: {}", e),
            }),
        )),
    }
}

pub async fn refresh_token<A: Authenticator>(
    State(state): State<A>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let refresh_token = match jwt::extract_jwt_from_headers(&headers) {
        Ok(token) => token,
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: format!("Token refresh failed: {}", e),
                }),
            ));
        }
    };

    match state.refresh_token(&refresh_token).await {
        Ok(state) => Ok(Json(state.into())),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: format!("Token refresh failed: {}", e),
            }),
        )),
    }
}
