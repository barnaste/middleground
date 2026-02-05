//! Supabase authentication backend implementation with JWKS support.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::models::{AuthSession, Authenticator};

use async_trait::async_trait;
use jsonwebtoken::{DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use supabase_auth::error as sb_error;
use supabase_auth::models as sb_models;
use tokio::sync::RwLock;

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

/// JWKS (JSON Web Key Set) response from Supabase
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// Individual JSON Web Key
#[derive(Debug, Deserialize, Clone)]
struct Jwk {
    kid: String, // key ID
    kty: String, // key type
    alg: String, // algorithm
    #[serde(rename = "use")]
    use_: Option<String>, // key usage

    // RSA-specific fields
    n: Option<String>, // RSA modulus
    e: Option<String>, // RSA exponent

    // EC-specific fields
    crv: Option<String>, // curve name
    x: Option<String>,   // x coordinate
    y: Option<String>,   // y coordinate
}

#[derive(Clone)]
struct CachedJwks {
    keys: HashMap<String, DecodingKey>,
    fetched_at: SystemTime,
    ttl: Duration, // time to live
}

impl CachedJwks {
    fn new(keys: HashMap<String, DecodingKey>, ttl: Duration) -> Self {
        Self {
            keys,
            fetched_at: SystemTime::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        SystemTime::now()
            .duration_since(self.fetched_at)
            .map(|elapsed| elapsed > self.ttl)
            .unwrap_or(true)
    }
}

/// JWT claims structure for access tokens.
#[derive(Debug, Deserialize)]
struct JwtClaims {
    pub sub: String, // subject (user UUID)
}

/// Supabase-based authenticator implementation.
///
/// This authenticator uses Supabase's authentication service to handle
/// OTP sending, verification, and session management. It uses the modern
/// JWKS endpoint for JWT verification with automatic key caching and rotation.
///
/// # Environment Variables
///
/// The following environment variables must be set:
/// - `SUPABASE_URL` - the Supabase project URL (should not include a terminating '/')
/// - `SUPABASE_API_KEY` - the Supabase API key
///
/// #JWKS Caching
///
/// The authenticator fetches public keys from Supabase's JWKS endpoint and caches
/// them for 1 hour by default. Keys are automatically refreshed when the cache
/// expires or when a token with an unknown key ID is encountered.
///
/// # Example
///
/// ```rust,no_run
/// use auth::models::SbAuthenticator;
///
/// // Create authenticator from environment variables
/// let authenticator = SbAuthenticator::default();
/// ```
#[derive(Clone)]
pub struct SbAuthenticator {
    client: sb_models::AuthClient,
    supabase_url: String,
    jwks_cache: Arc<RwLock<Option<CachedJwks>>>,
    http_client: reqwest::Client,
}

impl SbAuthenticator {
    /// Create a new sbAuthenticator with the provided AuthClient.
    pub fn new(client: sb_models::AuthClient, supabase_url: String) -> Self {
        Self {
            client,
            supabase_url,
            jwks_cache: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a new SbAuthenticator from environment variables.
    pub fn from_env() -> Result<Self, String> {
        let supabase_url = dotenvy::var("SUPABASE_URL")
            .map_err(|e| format!("SUPABASE_URL not set in environment: {e}"))?;
        let supabase_key = dotenvy::var("SUPABASE_API_KEY")
            .map_err(|e| format!("SUPABASE_KEY not set in environment: {e}"))?;

        // the last argument to this client is the JWT_SECRET, but it is never used
        let client =
            sb_models::AuthClient::new(supabase_url.clone(), supabase_key.clone(), String::new());

        Ok(Self::new(client, supabase_url))
    }

    /// Fetch JWKS from Supabase endpoint and convert to DecodingKeys.
    ///
    /// This method fetches the JSON Web Key Set from Supabase's well-known endpoint
    /// and converst the public keys into a format suitable for JWT verification.
    /// Supports both EC (ES256) and RSA (RS256) keys.
    async fn fetch_jwks(&self) -> Result<HashMap<String, DecodingKey>, String> {
        let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", self.supabase_url);

        let response = self
            .http_client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "JWKS endpoint returned error: {}",
                response.status()
            ));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {e}"))?;

        let mut keys = HashMap::new();

        for jwk in jwks.keys {
            let decoding_key = match jwk.kty.as_str() {
                "EC" => {
                    // Elliptic Curve key (ES256)
                    let x = jwk
                        .x
                        .ok_or_else(|| format!("EC key {} missing x coord", jwk.kid))?;
                    let y = jwk
                        .y
                        .ok_or_else(|| format!("EC key {} missing y coord", jwk.kid))?;

                    DecodingKey::from_ec_components(&x, &y).map_err(|e| {
                        format!("Failed to create EC decoding key for kid {}: {e}", jwk.kid)
                    })?
                }
                "RSA" => {
                    // RSA key (RS256)
                    let n = jwk
                        .n
                        .ok_or_else(|| format!("EC key {} missing modulus", jwk.kid))?;
                    let e = jwk
                        .e
                        .ok_or_else(|| format!("EC key {} missing exponent", jwk.kid))?;

                    DecodingKey::from_rsa_components(&n, &e).map_err(|e| {
                        format!("Failed to create RSA decoding key for kid {}: {e}", jwk.kid)
                    })?
                }
                other => {
                    tracing::warn!("Unsupported key type '{}' for kid {}", other, jwk.kid);
                    continue;
                }
            };

            keys.insert(jwk.kid, decoding_key);
        }

        if keys.is_empty() {
            return Err("No valid EC or RSA keys found in JWKS".to_string());
        }

        Ok(keys)
    }

    /// Get JWKS keys from cache or fetch if expired/missing.
    ///
    /// This method implements the caching logic with automatic refresh when needed.
    async fn get_jwks_keys(&self) -> Result<HashMap<String, DecodingKey>, String> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if !cached.is_expired() {
                    return Ok(cached.keys.clone());
                }
            }
        }

        // Cache is expired or doesn't exist, so we fetch new keys
        let keys = self.fetch_jwks().await?;

        // Update cache with 1-hour TTL
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some(CachedJwks::new(keys.clone(), Duration::from_secs(3600)));
        }

        Ok(keys)
    }

    /// Verify JWT using JWKS
    ///
    /// This method validates the JWT signature using public keys from the JWKS endpoint.
    /// It automatically handles key rotation by fetching updated keys when needed.
    /// Supports both ES256 (Elliptic Curve) and RS256 (RSA) algorithms.
    async fn verify_jwt(&self, token: &str) -> Result<uuid::Uuid, String> {
        // first, decode the header to get the key ID (kid) and algorithm
        let header = decode_header(token).map_err(|e| format!("Invalid JWT header: {e}"))?;

        let kid = header
            .kid
            .ok_or_else(|| "JWT missing key ID (kid)".to_string())?;

        // get keys from cache or fetch
        let mut keys = self.get_jwks_keys().await?;

        // try to find the key
        let decoding_key = match keys.get(&kid) {
            Some(key) => key.clone(),
            None => {
                // key not found in cache; refresh JWKS and try again
                keys = self.fetch_jwks().await?;

                // update cache
                {
                    let mut cache = self.jwks_cache.write().await;
                    *cache = Some(CachedJwks::new(keys.clone(), Duration::from_secs(3600)));
                }

                keys.get(&kid)
                    .ok_or_else(|| format!("Key ID {} not found in JWKS", kid))?
                    .clone()
            }
        };

        // validate the token with the detected algorithm
        let mut validation = Validation::new(header.alg);
        validation.validate_exp = true;
        validation.validate_aud = false;

        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| format!("JWT validation failed: {e}"))?;

        // parse user ID from claims
        uuid::Uuid::parse_str(&token_data.claims.sub)
            .map_err(|e| format!("Invalid user ID in token: {e}"))
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

    async fn send_otp(&self, contact: &str) -> Result<(), Self::Error> {
        self.client
            .send_email_with_otp(contact, None)
            .await
            .map(|_| ())
    }

    async fn verify_otp(&self, contact: &str, token: &str) -> Result<Self::Session, Self::Error> {
        let params = sb_models::VerifyEmailOtpParams {
            email: contact.to_string(),
            token: token.to_string(),
            otp_type: sb_models::OtpType::Email,
            options: None,
        };

        let session = self
            .client
            .verify_otp(sb_models::VerifyOtpParams::Email(params))
            .await?;

        Ok(session)
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
        // use JWKS verification only; no need for backend fallback
        self.verify_jwt(access_token).await.map_err(|_| {
            // convert string error into Supabase error
            sb_error::Error::NotAuthenticated
        })
    }

    async fn verify_token_strict(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error> {
        // first verify cryptographically with JWKS
        match self.verify_jwt(access_token).await {
            Ok(user_id) => {
                // then verify the session still exists in Supabase's database;
                // this ensures that the token hasn't been revoked
                self.client
                    .get_user(access_token)
                    .await
                    .map(|u| u.id)
                    .map(|db_user_id| {
                        // check that JWKS user ID matches database user ID
                        if db_user_id != user_id {
                            tracing::error!(
                                "User ID mismatch : JWKS claims {} but database has {}",
                                user_id,
                                db_user_id
                            );
                        }
                        user_id
                    })
            }
            Err(e) => {
                // jwks verification failed, so there's no reason to
                // check the database
                tracing::warn!("JWKS verification failed in strict mode: {}", e);
                Err(sb_error::Error::NotAuthenticated)
            }
        }
    }
}
