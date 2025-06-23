use entity::user::User;

// The initial rating assigned to a new user.
const INITIAL_SCORE: f32 = 300.0;

// After a conversation terminates, each participant assigns the other a score 
// between -MAX_SCORE (bad experience) and MAX_SCORE (good experience).
const MAX_SCORE: f32 = 5.0;

// The number of times a *newly-created* user can receive the *worst possible* score (-MAX_SCORE) from
// *newly-created* partners before they reach a rating of 0 (and are considered potential bad actors).
// Serves as a balancing factor in the update_user_rating() formula.
const K: f32 = 3.0;

fn process_message(msg: &str) {

}

// Return the initial rating assigned to a new user.
fn init_user_rating() -> f32 {
    INITIAL_SCORE
}

// Update the user's rating based on the score given to the user by their partner,
// weighted by the partner's own rating.
fn update_user_rating(user: &mut User, partner: User, score: f32) {
    user.rating += partner.rating * score / MAX_SCORE / K;
}

fn report() {

}