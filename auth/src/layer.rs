// session() -> gives us our session layer
// require_auth() -> gives us our auth-filter layer

use serde::{Deserialize, Serialize};
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore, cookie::time::Duration};

#[derive(Serialize, Deserialize)]
pub enum AuthStatus {
    Unauthenticated,
    Pending(i32),
    Authenticated,
}

pub fn session_manager<S: SessionStore>(store: S) -> SessionManagerLayer<S> {
    let expiry = Expiry::OnInactivity(Duration::weeks(1));

    SessionManagerLayer::new(store)
        .with_name("sessionId")
        .with_expiry(expiry)
        .with_always_save(true)
}

pub fn require_auth() {}
