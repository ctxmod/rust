//! Domain types with utils.
//!
//! This crate represents Layer 1 - domain types used across business logic.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// User identifier type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User domain type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
}

/// User validation errors.
#[derive(Debug, Error)]
pub enum UserError {
    #[error("invalid email format: {email}")]
    InvalidEmail { email: String },

    #[error("user {user_id} not found")]
    NotFound { user_id: UserId },
}

impl User {
    /// Create a new user with validation.
    pub fn new(id: UserId, email: String, name: String) -> Result<Self, UserError> {
        if !email.contains('@') {
            return Err(UserError::InvalidEmail { email });
        }

        Ok(Self { id, email, name })
    }
}
