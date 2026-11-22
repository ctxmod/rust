//! Library crate demonstrating Rust patterns.
//!
//! This example shows:
//! - thiserror for library error enums
//! - Visibility controls (pub, pub(crate), private)
//! - Feature-based module organization
//! - Non-exhaustive enums for API evolution

mod email;
mod error;
mod user;

// Re-export public API
pub use error::{UserError, ValidationError};
pub use user::{CreateUserRequest, User, UserId};
