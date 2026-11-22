//! Email validation module (internal).

use crate::error::ValidationError;

/// Email type with validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Email(String);

impl Email {
    /// Create a new email with validation.
    pub(crate) fn new(email: String) -> Result<Self, ValidationError> {
        if !email.contains('@') || !email.contains('.') {
            return Err(ValidationError::InvalidEmail { email });
        }

        Ok(Self(email))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}
