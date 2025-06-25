//! Supabase authentication backend implementation.

use crate::models::{AuthSession, Authenticator};

use async_trait::async_trait;
use supabase_auth::error as sb_error;
use supabase_auth::models as sb_models;

// Implement AuthSession for Supabase's Session type
impl AuthSession for sb_models::Session {
    fn access_token(&self) -> &str {
        &self.access_token
    }
    fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
    fn expires_at(&self) -> u64 {
        self.expires_at
    }
}

/// Supabase-based authenticator implementation.
///
/// This authenticator uses Supabase's authentication service to handle
/// OTP sending, verification, and session management. It requires
/// Supabase environment variables to be configured.
///
/// # Environment Variables
///
/// The following environment variables must be set:
/// - `SUPABASE_URL` - the Supabase project URL
/// - `SUPABASE_API_KEY` - the Supabase API key
/// - `SUPABASE_JWT_SECRET` - the JWT encryption secret
///
/// # Example
///
/// ```rust
/// use auth::models::SbAuthenticator;
///
/// // Create authenticator from environment variables
/// let authenticator = SbAuthenticator::default();
///
/// // Or create with custom client
/// let client = supabase_auth::models::AuthClient::new_from_env().unwrap();
/// let authenticator = SbAuthenticator::new(client);
/// ```
#[derive(Clone)]
pub struct SbAuthenticator {
    client: sb_models::AuthClient,
}

impl SbAuthenticator {
    /// Create a new sbAuthenticator with the provided AuthClient.
    pub fn new(client: sb_models::AuthClient) -> Self {
        Self { client }
    }

    /// Create a new SbAuthenticator from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if the required environment variables are not set
    /// or if the AuthClient cannot be initialized.
    pub fn from_env() -> Result<Self, Box<sb_error::Error>> {
        let client = sb_models::AuthClient::new_from_env()?;
        Ok(Self::new(client))
    }
}

impl Default for SbAuthenticator {
    /// Create a new SbAuthenticator using environment variables.
    ///
    /// # Panics
    ///
    /// Panics if the required environment variables are not set.
    /// For error handling, use `SbAuthenticator::from_env()` instead.
    fn default() -> Self {
        Self::from_env().expect("Failed to create SbAuthenticator from environment variables")
    }
}

#[async_trait]
impl Authenticator for SbAuthenticator {
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
            .logout(Some(sb_models::LogoutScope::Global), bearer_token)
            .await
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<Self::Session, Self::Error> {
        self.client.refresh_session(refresh_token).await
    }

    async fn verify_token(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error> {
        self.client.get_user(access_token).await.map(|u| u.id)
    }
}

// TODO: write tests!
