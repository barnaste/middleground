use entity::user::User;
use entity::conversation::Conversation;
use std::cmp;

/// The initial rating assigned to a new user.
const INITIAL_RATING: f64 = 300.0;

/// The minimum duration (in seconds) of a conversation before it is considered 'maximized',
/// i.e., its formulaic weight in update_user_rating() is maximized.
const CONVERSATION_DURATION_THRESHOLD: f64 = 60.0 * 60.0 * 24.0;

/// The number of times a *newly created* user can receive the *worst possible* score (-1) from *newly
/// created* partners, assuming *maximized* conversations, before they reach a rating of 0, i.e., are
/// considered potential bad actors. Serves as a balancing factor in the update_user_rating() formula.
const K: f64 = 3.0;

/// Return the initial rating assigned to a new user.
pub fn init_user_rating() -> f64 {
    INITIAL_RATING
}

/// Update the user's rating based on the score given to the user by their partner,
/// weighted by the partner's own rating and the duration of the conversation.
/// score is a float between -1 (bad experience) and 1 (good experience).
pub fn update_user_rating(user: &mut User, partner: User, score: f64, conversation: Conversation) {
    // the duration of the conversation
    let duration = conversation.ended_at.signed_duration_since(conversation.created_at).as_seconds_f64();

    // the fraction between 0 and 1 accounting for the duration of the conversation
    // (a compare function is necessary since floats do not implement TotalOrd)
    let floored_fraction = cmp::min_by(duration / CONVERSATION_DURATION_THRESHOLD, 1.0, |a, b| a.total_cmp(b));

    // the rating formula
    user.rating += partner.rating * score / K * floored_fraction;
}