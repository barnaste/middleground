use async_trait::async_trait;

/// A trait that enables access to key JWT-based auth values, namely the access
/// and refresh tokens, as well as the access token expiration time.
/// Meant to be used for types returned by the database and/or supplementary
/// services when performing authentication operations.
pub trait AuthSession {
    fn access_token(&self) -> &str;
    fn refresh_token(&self) -> &str;
    fn expires_at(&self) -> u64;
}

/// A trait that enables interfacing to the database and supplementary services
/// for standard authentication operations using JWTs and OTP-based login.
#[async_trait]
pub trait Authenticator: Clone + Send + Sync + 'static {
    /// the error type for the manager
    type Error: std::error::Error + Send + Sync + 'static;
    /// the struct containing session information, including at least the access
    /// token, its expiration, and the refresh token
    type Session: AuthSession + Send + Sync + 'static;

    async fn send_otp(&self, email: &str) -> Result<(), Self::Error>;
    async fn verify_otp(&self, email: &str, token: &str) -> Result<Self::Session, Self::Error>;
    async fn logout(&self, bearer_token: &str) -> Result<(), Self::Error>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<Self::Session, Self::Error>;

    /// Check that the access_token is a valid, untampered, JWT, *and that it refers
    /// to a user session that has not expired or been terminated*. Only use this
    /// function in situations where perfect authentication is required, as it will 
    /// check for the presence of a session (most likely in a database), which adds 
    /// additional complexity not necessary in most cases.
    async fn verify_token(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error>;
}
