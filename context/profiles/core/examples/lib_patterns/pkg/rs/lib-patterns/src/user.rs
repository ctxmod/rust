//! User domain module.

use crate::email::Email;
use crate::error::{UserError, ValidationError};
use serde::{Deserialize, Serialize};

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

/// User domain type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
}

/// Request to create a new user.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub id: String,
    pub email: String,
    pub name: String,
}

impl User {
    /// Create a new user with validation.
    pub fn create(req: CreateUserRequest) -> Result<Self, UserError> {
        // Validate name length
        const MIN_NAME_LENGTH: usize = 3;
        const MAX_NAME_LENGTH: usize = 100;

        if req.name.len() < MIN_NAME_LENGTH {
            return Err(ValidationError::NameTooShort {
                length: req.name.len(),
                minimum: MIN_NAME_LENGTH,
            }
            .into());
        }

        if req.name.len() > MAX_NAME_LENGTH {
            return Err(ValidationError::NameTooLong {
                length: req.name.len(),
                maximum: MAX_NAME_LENGTH,
            }
            .into());
        }

        // Validate email (internal type)
        let email = Email::new(req.email)?;

        Ok(Self {
            id: UserId::new(req.id),
            email: email.as_str().to_string(),
            name: req.name,
        })
    }
}
