auth
----

A flexible JWT-based authentication library for axum web applications centred around OTP (One-Time Password) verification with modern JWKS support.
This crate includes plug-and-play Supabase integration using JWKS (JSON Web Key Set) for secure token verification, and two middleware options for route protection.
Moreover, it defines an interface for defining authenticators that connect to alternative backends.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-red?logo=rust&style=for-the-badge)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

#### What's New: JWKS Support
`SbAuthenticator` now uses Supabase's modern JWKS endpoint for JWT verification instead of the legacy symmetric-key approach. This update provides
- *Better Security:* Uses public key cryptography instead of shared secrets
- *Automatic Key Rotation:* Handles Supabase's key rotation without downtime
- *Performance:* Caches public keys for 1 hour, reducing API calls
- *Standards Compliance:* Follows OAuth 2.0 / OpenID Connect best practices

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
auth = { path = "path/to/auth" }
```

When using the Supabase backend, configure the environment variables below. Note that the legacy `SUPABASE_JWT_SECRET` is no longer required, as the `SbAuthenticator` relies on a JWKS for verification rather than a shared private key.

```bash
SUPABASE_URL=your_project_url
SUPABASE_API_KEY=your_api_key
```

#### Quick Start
```rust
use auth::{router, models::SbAuthenticator};
use axum::Router;

#[tokio::main]
async fn main() {
    // Initialize authenticator from environment variables
    let authenticator = SbAuthenticator::default();
    
    // Create auth router with standard endpoints
    let auth_router = router(authenticator.clone());
    
    // Build your application
    let app = Router::new()
        .nest("/auth", auth_router);
    
    // Start server...
}
```

### Authentication Flow

The crate's `router()` function provides four endpoints that handle the complete authentication lifecycle. To authenticate, users first request an OTP at `/send-otp` with their email, then verify it at `/verify-otp` to receive access and refresh tokens. These tokens can be refreshed at `/refresh` or invalidated at `/logout`.

*Sending an OTP:*
```bash
POST /auth/send-otp
Content-Type: application/json

{"contact": "user@example.com"}
```

*Verifying the OTP:*
```bash
POST /auth/verify-otp
Content-Type: application/json

{"contact": "user@example.com", "token": "123456"}

# Returns:
# {
#   "access_token": "eyJhbGc...",
#   "refresh_token": "eyJhbGc...",
#   "expires_at": 1234567890
# }
```

*Refreshing tokens:*
```bash
POST /auth/refresh
Authorization: Bearer <refresh_token>

# Returns:
# {
#   "access_token": "eyJhbGc...",
#   "refresh_token": "eyJhbGc...",
#   "expires_at": 1234567890
# }
```

*Logging out:*
```bash
POST /auth/logout
Authorization: Bearer <access_token>
```

### Protecting Routes

The crate provides two middleware options for protecting routes. 
Standard middleware performs fast local JWT validation. 
It is ideal for most use cases, particularly those that do not mutate state. 
Strict middleware may provide additional guarantees, such as ensuring that sessions are still active in the database, providing stronger security at the cost of performance.
Use this for sensitive operations like administrative functions.

Both middleware options validate the JWT and insert the user's UUID into request extensions, making it available to your axum handlers.

```rust
use axum::{Router, routing::get, middleware, Extension};
use auth::{middleware::auth_standard, models::SbAuthenticator};
use uuid::Uuid;

async fn protected_handler(
    Extension(user_id): Extension<Uuid>,
) -> String {
    format!("Hello, user {}!", user_id)
}

let authenticator = SbAuthenticator::default();
let app = Router::new()
    .route("/protected", get(protected_handler))
    .route_layer(middleware::from_fn_with_state(
        authenticator.clone(),
        auth_standard  // or auth_strict for backend verification
    ))
```

### Architecture

The crate is built around two core traits that define the authentication interface.
`Authenticator` defines the operations an authentication backend must support: sending OTPs, verifying them, managing sessions, and validating tokens. 
`AuthSession` represents an authenticated session containing access and refresh tokens along with expiration information.

The included `SbAuthenticator` implements these traits for Supabase, but you can create custom backends by implementing the same interface. 
This trait-based design allows the library to work with any authentication provider while maintaining type safety and a consistent API.

#### Core Traits

```rust
#[async_trait]
pub trait Authenticator: Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    type Session: AuthSession + Send + Sync + 'static;

    async fn send_otp(&self, contact: &str) -> Result<(), Self::Error>;
    async fn verify_otp(&self, contact: &str, token: &str) -> Result<Self::Session, Self::Error>;
    async fn logout(&self, bearer_token: &str) -> Result<(), Self::Error>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<Self::Session, Self::Error>;
    async fn verify_token(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error>;
    async fn verify_token_strict(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error>;
}

pub trait AuthSession {
    fn access_token(&self) -> &str;
    fn refresh_token(&self) -> &str;
    fn expires_at(&self) -> u64;
}
```

#### Module Organization

The crate is organized into modules: 
- `dto` contains request and response structures;
- `handlers` implements the HTTP endpoint logic;
- `error` defines the crate's error type for expressing JWT-related errors;
- `middleware` contains the two authentication middleware options; and 
- `models` defines the core traits along with the Supabase implementation.

### License

Please see the workspace root for license information.
