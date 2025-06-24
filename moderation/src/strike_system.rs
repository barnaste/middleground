use entity::user::User;
use entity::conversation::Conversation;

/// Update a user's strikes if they were either in a reported conversation or have a score of zero;
/// if so, a sentiment analyzer is called that may determine there is a sufficiently high chance
/// the user is a bad actor, in which case they are given a strike and a timeout.
pub fn update_user_strikes() {

}

/// Perform sentiment analysis on a conversation for a particular user, returning
/// the expected score (between -1 and 1) for the user to assign to their partner.
pub fn analyze_conversation(conversation: Conversation, user: User) -> f64 {
    0.0
}