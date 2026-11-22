//! Business logic with domain types.
//!
//! This crate represents Layer 2 - business logic using domain types.

use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::info;
use types::{User, UserError, UserId};
use utils::Timer;

/// User repository for business operations.
pub struct UserRepository {
    users: HashMap<UserId, User>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Find a user by ID.
    pub fn find(&self, id: &UserId) -> Result<&User> {
        let _timer = Timer::new("find_user");

        self.users
            .get(id)
            .ok_or_else(|| UserError::NotFound {
                user_id: id.clone(),
            })
            .context("failed to find user")
    }

    /// Save a user.
    pub fn save(&mut self, user: User) -> Result<()> {
        let _timer = Timer::new("save_user");

        let id = user.id.clone();
        self.users.insert(id, user);
        info!("user saved successfully");
        Ok(())
    }

    /// List all users.
    pub fn list(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}
