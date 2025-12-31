# Auth

A flexible JWT-based authentication library for Axum web applications centred around OTP (One-Time Password) verification.

This crate provides a complete authentication solution with OTP-based passwordless login, JWT token management (access and refresh tokens), and pluggable authentication backends. It includes a production-ready Supabase integration and two middleware options for route protection.

## Quick Start

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

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
auth = { path = "path/to/auth" }
```

When using the Supabase backend, configure the following environment variables:

```bash
SUPABASE_URL=your_project_url
SUPABASE_API_KEY=your_api_key
SUPABASE_JWT_SECRET=your_jwt_secret
```

## Authentication Flow

The `router()` function provides four endpoints that handle the complete authentication lifecycle. To authenticate, users first request an OTP at `/send-otp` with their email, then verify it at `/verify-otp` to receive access and refresh tokens. These tokens can be refreshed at `/refresh` or invalidated at `/logout`.

**Sending an OTP:**
```bash
POST /auth/send-otp
Content-Type: application/json

{"contact": "user@example.com"}
```

**Verifying the OTP:**
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

**Refreshing tokens:**
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

**Logging out:**
```bash
POST /auth/logout
Authorization: Bearer <access_token>
```

## Protecting Routes

The crate provides two middleware options for protecting routes. Standard middleware performs fast local JWT validation using HMAC signature verification. It is ideal for most use cases, particularly those that do not mutate state. Strict middleware additionally validates tokens against the authentication backend to ensure the session is still active, providing stronger security at the cost of performance. Use this for sensitive operations like administrative functions.

Both middleware options validate the JWT and insert the user's UUID into request extensions, making it available to your handlers.

```rust
use axum::{Router, routing::get, middleware, Extension};
use auth::middleware::auth_standard;
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
    .with_state(authenticator);
```

## Architecture

The crate is built around two core traits that define the authentication interface. `Authenticator` defines the operations an authentication backend must support: sending OTPs, verifying them, managing sessions, and validating tokens. `AuthSession` represents an authenticated session containing access and refresh tokens along with expiration information.

The included `SbAuthenticator` implements these traits for Supabase, but you can create custom backends by implementing the same interface. This trait-based design allows the library to work with any authentication provider while maintaining type safety and a consistent API.

### Core Traits

```rust
#[async_trait]
pub trait Authenticator: Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    type Session: AuthSession + Send + Sync + 'static;

    fn jwt_secret(&self) -> &str;
    async fn send_otp(&self, contact: &str) -> Result<(), Self::Error>;
    async fn verify_otp(&self, contact: &str, token: &str) -> Result<Self::Session, Self::Error>;
    async fn logout(&self, bearer_token: &str) -> Result<(), Self::Error>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<Self::Session, Self::Error>;
    async fn verify_token(&self, access_token: &str) -> Result<uuid::Uuid, Self::Error>;
}

pub trait AuthSession {
    fn access_token(&self) -> &str;
    fn refresh_token(&self) -> &str;
    fn expires_at(&self) -> u64;
}
```

### Module Organization

The crate is organized into modules: `dto` contains request and response structures, `handlers` implements the HTTP endpoint logic, `jwt` provides token extraction and validation utilities, `middleware` contains the two authentication middleware options, and `models` defines the core traits along with the Supabase implementation.

## JWT Utilities

The `jwt` module provides utilities for working with JWTs directly. You can extract tokens from Authorization headers and validate them using HMAC signature verification. The validation checks both the signature and expiration time, returning decoded claims containing the user ID and expiration timestamp.

```rust
use auth::jwt::{extract_jwt_from_headers, validate_jwt_hmac, Claims};

let token = extract_jwt_from_headers(&headers)?;
let claims: Claims = validate_jwt_hmac(&token, jwt_secret)?;
println!("User ID: {}", claims.sub);  // UUID string
println!("Expires at: {}", claims.exp);  // Unix timestamp
```

## Error Handling

The crate uses type-safe errors throughout. The `AuthError` enum covers JWT-related errors like missing or invalid headers and invalid tokens. HTTP handlers map these to appropriate status codes: 400 for malformed requests, 401 for authentication failures, and 200 for success.

```rust
pub enum AuthError {
    MissingAuthHeader,
    InvalidAuthHeader,
    InvalidToken(String),
}
```

## Testing

Run the test suite with `cargo test`. The crate includes comprehensive tests covering JWT extraction, validation with correct and incorrect secrets, expiration handling, tamper detection, and invalid format handling.

## Key Dependencies

The crate builds on `axum` for the web framework, `jsonwebtoken` for JWT operations, `supabase-auth` for Supabase integration, and `uuid` for user identification. All authentication operations are async using `tokio` and `async-trait`.

## License

See the workspace root for license information.
