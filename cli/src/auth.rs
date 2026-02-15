//! Autthentication client for OTP-based authentication flow.
//!
//! Handles authentication against the Middleground backend, including:
//! - Sending OTP codes via email
//! - Verifying OTP codes to obtain access tokens
//! - Token refresh logic with automatic expiration handling
//! - Session logout

use anyhow::{Result, bail};
use colored::Colorize;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

/// Request payload for sending an OTP
#[derive(Serialize)]
struct SendOtpRequest {
    contact: String,
}

/// Request payload for verifying an OTP
#[derive(Serialize)]
struct VerifyOtpRequest {
    contact: String,
    token: String,
}

/// Response from authentication endpoints containing tokens
#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_at: u64,
}

/// HTTP client for authentication operations.
///
/// Manages the authentication lifecycle including OTP sending/verification, token management, and
/// automatic token refresh before expiration.
///
/// # Token Management
///
/// The client automatically refreshes access tokens when they're close to expiration (within 60
/// seconds). This ensures that authorization headers always contain valid tokens.
///
/// # Examplet
///
/// ```rust,no_run
/// use auth::AuthClient;
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut client = AuthClient::new("http://localhost:8080");
///
/// // send OTP
/// client.send_otp("user@example.com").await?;
///
/// // verify OTP
/// client.verify_otp("user@example.com", "12345678").await?;
///
/// // get authorization header (automatically refreshes if needed)
/// let headers = client.get_authorization_header().await;
/// # Ok(())
/// # }
/// ```
pub struct AuthClient {
    client: reqwest::Client,
    base_url: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
    tok_expiry: Option<u64>,
}

impl AuthClient {
    /// Create a new authentication client.
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the backend API (e.g. "http://localhost:8080")
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be created.
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| panic!("{} FATAL: Unable to create an HTTP client", "‼".red()));

        Self {
            client,
            base_url: base_url.to_string(),
            access_token: None,
            refresh_token: None,
            tok_expiry: None,
        }
    }

    /// Send OTP to user's contact (email).
    ///
    /// Initiates the authentication flow by requesting an OTP be sent to the specified email
    /// address.
    ///
    /// # Arguments
    /// * `contact` - Email address to send the OTP to
    ///
    /// # Errors 
    ///
    /// Returns an error if:
    /// * The HTTP request fails 
    /// * The server returns a non-success status code
    /// * The contact is invalid or not found
    pub async fn send_otp(&self, contact: &str) -> Result<()> {
        let url = format!("{}/auth/send-otp", self.base_url);
        let request = SendOtpRequest {
            contact: contact.to_string(),
        };

        let response = self.client.post(url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to send OTP ({}): {}", status, body)
        }
    }

    /// Verify an OTP and get authorization tokens.
    ///
    /// Completes the authentication flow by verifying the OTP code and obtaining access and
    /// refresh tokens.
    ///
    /// # Arguments
    /// * `contact` - Email address the OTP was sent to
    /// * `token` - The OTP code
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The HTTP request fails
    /// * The OTP is invalid or expired
    pub async fn verify_otp(&mut self, contact: &str, token: &str) -> Result<()> {
        let url = format!("{}/auth/verify-otp", self.base_url);
        let request = VerifyOtpRequest {
            contact: contact.to_string(),
            token: token.to_string(),
        };

        let response = self.client.post(url).json(&request).send().await?;

        if response.status().is_success() {
            let auth_resp = response.json::<AuthResponse>().await?;
            self.refresh_token = Some(auth_resp.refresh_token);
            self.access_token = Some(auth_resp.access_token);
            self.tok_expiry = Some(auth_resp.expires_at);

            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to verify OTP ({}): {}", status, body)
        }
    }

    /// Refresh the access token using the refresh token.
    ///
    /// This is called automatically by `get_authorization_header()` when the access token is close
    /// to expiration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The refresh token is invalid or expired
    /// * The HTTP request fails
    ///
    /// # Panics
    ///
    /// Panics if called before authentication (no refresh token available).
    async fn refresh_token(&mut self) -> Result<()> {
        assert!(
            self.refresh_token.is_some(),
            "{}",
            format!("{} FATAL: Token data is not valid.", "‼".red())
        );

        let url = format!("{}/auth/refresh", self.base_url);
        let response = self
            .client
            .post(url)
            .header(
                AUTHORIZATION,
                format!("Bearer {}", self.refresh_token.as_ref().unwrap()),
            )
            .send()
            .await?;

        if response.status().is_success() {
            let auth_resp = response.json::<AuthResponse>().await?;
            self.refresh_token = Some(auth_resp.refresh_token);
            self.access_token = Some(auth_resp.access_token);
            self.tok_expiry = Some(auth_resp.expires_at);

            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to refresh access token ({}): {}", status, body)
        }
    }

    /// Generate an authorization header with a valid access token.
    ///
    /// Automatically refreshes the access token if it's close to expiration (within 60 seconds).
    /// Guarantees that the returned header contains a valid, non-expired token.
    ///
    /// # Panics
    ///
    /// Panics if called before authentication is complete.
    ///
    /// # Returns
    ///
    /// A `HeaderMap` containing the `Authorization` header with a Bearer token.
    pub async fn get_authorization_header(&mut self) -> HeaderMap {
        assert!(
            self.tok_expiry.is_some() && self.access_token.is_some(),
            "{}",
            format!("{} FATAL: Token data is not valid.", "‼".red())
        );

        // first fetch the current Unix time (tok_expiry is in Unix epoch time)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| panic!("{} FATAL: Time has gone backwards", "‼".red()))
            .as_secs();

        // refresh if expiring within 60 seconds
        if self.tok_expiry.unwrap() < now + 60 {
            // notice: if we fail to refresh the token but know that our tokens are valid, then
            // we retry until we succeed with the refresh; the only possible point of failure was
            // information transmission
            while self.refresh_token().await.is_err() {
                std::thread::sleep(std::time::Duration::new(20, 0));
            }
        }

        // now place the access token into the header map
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.access_token.as_ref().unwrap()))
                .unwrap(), // note that neither of these unwraps can ever fail
        );
        headers
    }

    /// Logout and invalidate the current session.
    ///
    /// Terminates the current authentication session on the server side, invalidating all tokens.
    /// Note that tokens will still pass standard JWT checks that don't verify session activity.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The session is already invalid
    pub async fn logout(&mut self) -> Result<()> {
        let url = format!("{}/auth/logout", self.base_url);
        let response = self
            .client
            .post(url)
            .headers(self.get_authorization_header().await)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to logout ({}): {}", status, body)
        }
    }
}
