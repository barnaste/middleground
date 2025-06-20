/// An implementation of the AuthManager trait using supabase_auth.
use supabase_auth::models::AuthClient;

pub struct SbManager {
    client: AuthClient,
}
