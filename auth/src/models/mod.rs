//! Authentication models and traits.

mod authenticator;
pub mod sb_authenticator;

pub use authenticator::{Authenticator, AuthSession};
pub use sb_authenticator::SbAuthenticator;
