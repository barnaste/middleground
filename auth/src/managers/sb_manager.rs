/// An implementation of the AuthManager trait using Supabase's authentication system, directly
/// on top of the AuthClient struct as offered by supabase_auth.
use crate::managers::{AuthManager, AuthSession};

use async_trait::async_trait;
use supabase_auth::error as sb_error;
use supabase_auth::models as sb_models;

// ------------------
//    AUTH SESSION
// ------------------

impl AuthSession for sb_models::Session {
    fn access_token(&self) -> &str { &self.access_token }
    fn refresh_token(&self) -> &str { &self.refresh_token }
    fn expires_at(&self) -> u64 { self.expires_at }
}

// ------------------
//    AUTH MANAGER
// ------------------

#[derive(Clone)]
pub struct SbManager {
    pub client: sb_models::AuthClient,
}

#[async_trait]
impl AuthManager for SbManager {
    type Error = sb_error::Error;
    type Session = sb_models::Session;

    async fn send_otp(&self, email: &str) -> Result<(), Self::Error> {
        self.client
            .send_email_with_otp(email, None)
            .await
            .map(|_| ())
    }

    async fn verify_otp(&self, email: &str, token: &str) -> Result<Self::Session, Self::Error> {
        let params = sb_models::VerifyEmailOtpParams {
            email: email.to_string(),
            token: token.to_string(),
            otp_type: sb_models::OtpType::Email,
            options: None,
        };
        self.client
            .verify_otp(sb_models::VerifyOtpParams::Email(params))
            .await
    }

    async fn logout(&self, bearer_token: &str) -> Result<(), Self::Error> {
        self.client
            .logout(Some(sb_models::LogoutScope::Global), &bearer_token)
            .await
    }
    
    async fn refresh_token(&self, refresh_token: &str) -> Result<Self::Session, Self::Error> {
        self.client.refresh_session(refresh_token).await
    }
}
