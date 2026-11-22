//! Binary application demonstrating workspace usage.
//!
//! This crate represents Layer 3 - application consuming all layers.

use anyhow::Result;
use domain::UserRepository;
use tracing::info;
use types::{User, UserId};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("starting workspace example");

    // Create repository
    let mut repo = UserRepository::new();

    // Create and save users
    let alice = User::new(
        UserId::new("alice-123"),
        "alice@example.com".to_string(),
        "Alice Anderson".to_string(),
    )?;

    let bob = User::new(
        UserId::new("bob-456"),
        "bob@example.com".to_string(),
        "Bob Brown".to_string(),
    )?;

    repo.save(alice)?;
    repo.save(bob)?;

    // List all users
    let users = repo.list();
    info!("found {} users", users.len());

    for user in users {
        info!("user: {} ({})", user.name, user.email);
    }

    // Find specific user
    let found = repo.find(&UserId::new("alice-123"))?;
    info!("found alice: {}", found.name);

    info!("workspace example complete");
    Ok(())
}
