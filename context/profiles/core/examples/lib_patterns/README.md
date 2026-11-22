# Library Crate Patterns Example

Demonstrates Rust library patterns: thiserror for error enums, visibility controls, and module organization.

## Patterns Demonstrated

This example shows library crate best practices:

1. **thiserror for error enums**: Public error types with `#[derive(thiserror::Error)]`
2. **Visibility controls**: `pub` for public API, `pub(crate)` for internal visibility
3. **Module organization**: Feature-based modules with flat structure
4. **Non-exhaustive enums**: API evolution with `#[non_exhaustive]`

## Library Structure

```
lib_patterns/
├── Cargo.toml              # Workspace manifest
└── pkg/rs/lib-patterns/
    ├── Cargo.toml          # Crate manifest
    └── src/
        ├── lib.rs          # Library root with re-exports
        ├── user.rs         # User domain module
        ├── email.rs        # Email validation module (internal)
        └── error.rs        # Error types module
```

The crates live under `pkg/rs/`, the layout of a polyglot monorepo; a Rust-only repository would use `crates/` or put the crate at the root. The example carries no tests, so it reads the same under any test-placement policy; behavior contracts are documented with doctests. The rust `verification` profile's test-fixtures example shows the runner mechanics.

## Key Principles

### Error Handling

- Use `thiserror` for library error enums
- Implement `From` for protocol mappings (gRPC, HTTP)
- Add `#[non_exhaustive]` for API evolution

### Visibility

- `pub`: Public API items
- `pub(crate)`: Internal crate visibility
- Private: Implementation details within modules

### Module Organization

- Feature-based modules (user, email, error)
- Flat structure (2 levels max)
- Re-export public API from lib.rs

## Running the Example

```bash
# Build library
cargo build

# Check public API
cargo doc --open
```

## References

- Error Handling: the rust `core` profile, `errors` package, `reference/error_handling.md`
- Visibility: the rust `core` profile, `visibility` package, `reference/visibility.md`
- Workspace Organization: the rust `core` profile, `workspace` package, `reference/workspace_organization.md`
