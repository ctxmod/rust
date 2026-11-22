# Visibility Standards

All types private by default. Use `pub` only for public API surface. Apply `#[non_exhaustive]` to public types for API evolution safety.

## Rationale

Private-by-default minimizes API surface, reducing breaking change risk and enabling internal refactoring without semver constraints. Explicit `pub` markers document intentional API exposure. The `#[non_exhaustive]` attribute enables additive API changes without breaking changes.

## Private-by-Default

Default visibility is private. Add `pub` only when external code must access the item:

```rust
// Internal implementation - no pub
struct InternalCache {
    entries: HashMap<String, Entry>,
}

// Public API - explicit pub
pub struct UserService {
    cache: InternalCache,  // Private field
}

impl UserService {
    pub fn new() -> Self { /* ... */ }  // Public constructor

    fn validate(&self) -> bool { /* ... */ }  // Private helper
}
```

## Non-Exhaustive Attribute

Apply `#[non_exhaustive]` to public enums and structs:

```rust
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ErrorCode {
    NotFound,
    InvalidInput,
    // Future variants can be added without breaking changes
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    pub environment: String,
    // Future fields can be added without breaking changes
}
```

## Module Visibility

Organize modules with explicit re-exports:

```rust
// src/lib.rs
mod internal;
mod types;

pub use types::{Config, Service};  // Explicit public API
// internal module remains private
```

## Examples

**Correct**:
```rust
#[non_exhaustive]
pub struct ApiResponse {
    pub status: u16,
    body: String,  // Private field
}

impl ApiResponse {
    pub fn new(status: u16, body: String) -> Self {
        Self { status, body }
    }
}
```

**Incorrect**:
```rust
pub struct ApiResponse {
    pub status: u16,
    pub body: String,  // Exposes internal representation
}

pub enum ErrorCode {
    NotFound,
}  // Missing #[non_exhaustive] prevents adding variants
```

## Exceptions

**Internal test utilities**: Test helpers may use `pub(crate)` for visibility within the crate without exposing to external crates.

**FFI boundaries**: External function interfaces require `pub` for linkage but may document as internal through naming conventions (`pub extern "C" fn internal_hook()`).

## Related

[Workspace Organization](../../workspace/reference/workspace_organization.md)
[Error Handling](../../errors/reference/error_handling.md)
