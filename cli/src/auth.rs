// NOTE: we use anyhow::Error because it offers quick-and-easy error
// handling and propagation, and because we do not need custom error logic
// in the CLI tool, as we would in other services and libraries.
use anyhow::{Result, bail};
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
    tok_expir: Option<u64>,
}

impl AuthClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("FATAL: unable to create an HTTP client.");

        Self {
            client,
            base_url: base_url.to_string(),
            access_token: None,
            refresh_token: None,
            tok_expir: None
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
            self.tok_expir = Some(auth_resp.expires_at);

            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to send OTP ({}): {}", status, body)
        }
    }
}

// TODO: refresh_token, logout via reqwest
