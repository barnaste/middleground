//! Authentication models and traits.

mod authenticator;
pub mod sb_authenticator;

pub use authenticator::{AuthSession, Authenticator};
pub use sb_authenticator::SbAuthenticator;
