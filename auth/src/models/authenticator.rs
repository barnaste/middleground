//! Core authentication traits defining the interface for authentication backends.

use async_trait::async_trait;

/// Trait for types that represent an authenticated session.
///
/// This trait provides access to the essential components of an authentication
/// session: access token, refresh token, and expiration time.
pub trait AuthSession {
    /// Returns the access token for this session.
    fn access_token(&self) -> &str;

    // Returns the refresh token for this session.
    fn refresh_token(&self) -> &str;

    // Returns the expiration time as a Unix epoch timestamp.
    fn expires_at(&self) -> u64;
}

/// Trait for authentication backends that handle JWT and OTP-based authentication.
///
/// This trait defines the standard authentication operations needed for a complete
/// authentication system, including OTP sending/verification, session management,
/// and token operations.
#[async_trait]
pub trait Authenticator: Clone + Send + Sync + 'static {

    /// The error type returned by authentication operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// The session type containing authentication tokens and metadata.
    type Session: AuthSession + Send + Sync + 'static;

    /// Returns the JWT secret used for token signatures.
    ///
    /// This secret is used by middleware for local JWT validation without
    /// requiring a call to the authentication backend.
    fn jwt_secret(&self) -> &str;

    /// Send an OTP (One-Time Password) to the specified contact.
    ///
    /// # Arguments
    /// * `contact` - The contact information to which the OTP is sent
    ///
    /// # Returns
    /// * `Ok(())` if the OTP was sent successfully
    /// * `Err(Self::Error)` if the operation failed
    async fn send_otp(&self, contact: &str) -> Result<(), Self::Error>;

    /// Verify an OTP and create an authenticated session.
    ///
    /// # Arguments
    /// * `contact` - The contact information to which the OTP was sent
    /// * `token` - The OTP token to verify
    ///
    /// # Returns
    /// * `Ok(Self::Session)` with the new session if verification succeeded
    /// * `Err(Self::Error)` if verification failed
    async fn verify_otp(&self, contact: &str, token: &str) -> Result<Self::Session, Self::Error>;
    
    /// Log out a user by invalidating their session.
    ///
    /// # Arguments
    /// * `bearer_token` - The access token of the session to invalidate
    ///
    /// # Returns
    /// * `Ok(())` if logout was successful
    /// * `Err(Self::Error)` if the operation failed, including because the
    /// token was invalid or there exists no session associated to the token
    async fn logout(&self, bearer_token: &str) -> Result<(), Self::Error>;

    /// Refresh an access token using a refresh token. 
    ///
    /// # Arguments
    /// * `refresh_token` - The refresh token to use for getting a new access token
    ///
    /// # Returns
    /// * `Ok(Self::Session)` with new session if the refresh token was valid
    /// * `Err(Self::Error)` if the refresh token was invalid
    async fn refresh_token(&self, refresh_token: &str) -> Result<Self::Session, Self::Error>;

    /// Verify that an access token is valid and return the associated user ID.
    ///
    /// This method performs strict verification by checking both the token's
    /// cryptographic validity and its status in the authentication backend
    /// (e.g. checking if the associated session still exists in the database).
    ///
    /// Use this method when you need the highest level of security assurance,
    /// such as for administrative operations or sensitive data access. Otherwise,
    /// JWT validation is not implementation-specific but instead encryption 
    /// algorithm specific, so that `crate::verify::validate_jwt_hmac` can be used.
    ///
    /// # Arguments
    /// * `access_token` - The access token to verify
    ///
    /// # Returns
    /// * `Ok(uuid::Uuid)` with the user ID if the token is valid
    /// * `Err(Self::Error)` if the token is invalid or verification failed
    async fn verify_token(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error>;
}
