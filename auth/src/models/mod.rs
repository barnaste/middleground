// TODO: where do you REALLY need pub modifiers...
mod authenticator;
pub mod sb_authenticator;

pub use authenticator::Authenticator;
pub use authenticator::AuthSession;
