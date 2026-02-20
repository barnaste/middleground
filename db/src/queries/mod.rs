//! Database query modules.
//!
//! This module organizes database queries by domain. Each domain (user, source, etc.)
//! has its own submodule containing related queries.
//!
//! All query functions follow consistent patterns:
//! - Accept a `&DbPool` as the first parameter
//! - Return a `Result<T, DbError>` for error handling
//! - Use `sqlx::query_as!` for type-safe queries where possible

pub mod conversations;
pub mod messages;
pub mod users;
