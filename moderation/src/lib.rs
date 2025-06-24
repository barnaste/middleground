//! A moderation system for users participating in a conversation. Includes basic message filtering, a rating system determined by
//! user-assigned scores, and a strike system driven by sentiment analysis.
//! 
//! Outline of the moderation flow:
//! - Two users enter a conversation
//!     - A user sends a message, before which process_message() is called
//!         - If process_message() detects obviously offensive language, the user is warned
//!     - A user ends the conversation, and each user is given the option to either report their partner for bad behaviour (providing a
//!       reason for logs), or assign them a score (-1 to 1). (Similarly, if both users are inactive for a prolonged period of time, the
//!       conversation ends automatically.)
//!         - Rating system updates occur through update_user_rating(), weighed by assigned scores, current user ratings, and conversation duration
//!         - Strike system updates occur through update_user_strikes() as follows:
//!             - If a user was reported, sentiment analysis is conducted on the whole conversation through analyze_conversation()
//!                 - If analyze_conversation() returns a sufficiently negative score *for either user*, they receive a strike and timeout
//!             - If a user has (or just hit) 0 rating, sentiment analysis is conducted on the whole conversation through analyze_conversation()
//!                 - If analyze_conversation() returns a sufficiently negative score *for the 0-score user*, they receive a strike and timeout
//!             - If a user has accumulated enough strikes, they receive a permanent (or lengthy) ban

pub mod rating_system;
pub mod strike_system;

/// Scan a message for obviously offensive language. Returns a vector of slices of the message
/// detected as offensive, or an empty vector if none are found.
pub fn process_message<'a>(msg: &'a str) -> Vec<&'a str> {
    Vec::<&str>::new()
}