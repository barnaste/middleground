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

pub struct AuthState<'a> {
    client: &'a reqwest::Client,
    base_url: String,
}

impl<'a> AuthState<'a> {
    pub fn new(client: &'a reqwest::Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Send OTP to user's contact
    pub async fn send_otp(&self, contact: &str) -> Result<()> {
        let url = format!("{}/auth/send-otp", self.base_url);
        let request = SendOtpRequest { contact: contact.to_string() };

        let response = self.client.post(url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to send OTP ({}): {}", status, body)
        }
    }

    pub async fn verify_otp(&self, contact: &str, token: &str) -> Result<(String, String, u64)> {
        let url = format!("{}/auth/verify-otp", self.base_url);
        let request = VerifyOtpRequest { 
            contact: contact.to_string(),
            token: token.to_string() 
        };

        let response = self.client.post(url).json(&request).send().await?;

        if response.status().is_success() {
            let aresp = response.json::<AuthResponse>().await?;
            Ok((aresp.access_token, aresp.refresh_token, aresp.expires_at))
        } else {
            let status = response.status();
            let body = response.text().await?;
            bail!("Failed to send OTP ({}): {}", status, body)
        }
    }
}

// TODO: send_otp, verify_otp, refresh_token, logout via reqwest
