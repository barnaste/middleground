// NOTE: we use anyhow::Error because it offers quick-and-easy error
// handling and propagation, and because we do not need custom error logic
// in the CLI tool, as we would in other services and libraries.
use anyhow::{Result, bail};
use colored::Colorize;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SendOtpRequest {
    contact: String,
}

#[derive(Serialize)]
struct VerifyOtpRequest {
    contact: String,
    token: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_at: u64,
}

/// Client used for HTTP requests -- handles authentication.
pub struct AuthClient {
    client: reqwest::Client,
    base_url: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
    tok_expiry: Option<u64>,
}

impl AuthClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| panic!("{} FATAL: Unable to create an HTTP CLIENT", "‼".red()));

        Self {
            client,
            base_url: base_url.to_string(),
            access_token: None,
            refresh_token: None,
            tok_expiry: None,
        }
    }

    /// Send OTP to user's contact
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

    /// Verify an OTP and get authorization tokens
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

    /// Refresh the access token.
    /// Assumes that self is initialized with valid tokens and token data.
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

    /// Generate an authorization header. Guarantees that the authorization header contains tokens
    /// that have not yet expired.
    /// Assumes that self is initialized with valid tokens and token data.
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
        if self.tok_expiry.unwrap() < now + 60 {
            // quickfix: if we fail to refresh the token but know that our tokens are valid, then
            // we retry until we succeed with the refresh -- the only possible point of failure was
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

    /// Logout and invalidate the current session
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

// TODO: refresh_token, logout via reqwest
