//! Error types for the library.

use thiserror::Error;

/// Validation errors for user input.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidationError {
    #[error("invalid email format: {email}")]
    InvalidEmail { email: String },

    #[error("name too short: {length} characters (minimum: {minimum})")]
    NameTooShort { length: usize, minimum: usize },

    #[error("name too long: {length} characters (maximum: {maximum})")]
    NameTooLong { length: usize, maximum: usize },
}

/// User domain errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UserError {
    #[error("validation failed: {source}")]
    Validation {
        #[source]
        source: ValidationError,
    },

    #[error("user {user_id} not found")]
    NotFound { user_id: String },

    #[error("database error: {message}")]
    Database { message: String },
}

// Protocol mappings for error conversion
impl From<ValidationError> for UserError {
    fn from(err: ValidationError) -> Self {
        UserError::Validation { source: err }
    }
}
