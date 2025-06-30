//! HTTP handlers for authentication endpoints.

use crate::dto::*;
use crate::jwt;
use crate::models::Authenticator;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};

/// Send OTP to the user's provided contact (e.g. email address).
pub async fn send_otp<A: Authenticator>(
    State(authenticator): State<A>,
    Json(payload): Json<SendOtpRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticator
        .send_otp(&payload.contact)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to send OTP: {}", e),
                }),
            )
        })?;

    Ok(Json(MessageResponse {
        message: "OTP sent. Please check your inbox.".to_string(),
    }))
}

/// Verify OTP and return authentication tokens.
pub async fn verify_otp<A: Authenticator>(
    State(authenticator): State<A>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session = authenticator
        .verify_otp(&payload.contact, &payload.token)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("OTP verification failed: {}", e),
                }),
            )
        })?;

    Ok(Json(session.into()))
}

/// Logout user and invalidate the session.
pub async fn logout<A: Authenticator>(
    State(authenticator): State<A>,
    headers: HeaderMap,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = jwt::extract_jwt_from_headers(&headers).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Logout failed: {}", e),
            }),
        )
    })?;

    authenticator.logout(&token).await.map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: format!("Logout failed: {}", e),
            }),
        )
    })?;

    Ok(Json(MessageResponse {
        message: "Successfully logged out".to_string(),
    }))
}

/// Refresh access token using refresh token.
pub async fn refresh_token<A: Authenticator>(
    State(authenticator): State<A>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let refresh_token = jwt::extract_jwt_from_headers(&headers).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Token refresh failed: {}", e),
            }),
        )
    })?;

    let session = authenticator
        .refresh_token(&refresh_token)
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: format!("Token refresh failed: {}", e),
                }),
            )
        })?;

    Ok(Json(session.into()))
}
