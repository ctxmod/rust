---
paths:
  - "*.rs"
  - "*/*.rs"
  - "*/*/*.rs"
  - "*/*/*/*.rs"
  - "*/*/*/*/*.rs"
  - "*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*/*.rs"
  - "*/*/*/*/*/*/*/*/*/*.rs"
---

# Module Organization

Organize modules by domain features (user, auth, payment) rather than technical layers (handlers, models, services). Each feature module contains all its related code — types, business logic, handlers, storage. Prefer flat structures: limit nesting to 2-3 levels, `src/user/auth.rs` over `src/handlers/user/auth/login.rs`. Use `pub use` re-exports at the feature-module root to flatten the public API.

Exceptions: shared infrastructure (logging, metrics, config) may organize by technical category when used across features; generated types may isolate in a separate module tree (`src/gen/`); platform-conditional code may use platform subdirectories (`src/platform/linux/`, `src/platform/macos/`).

- the feature-based and flat-vs-deep directory trees, the module re-export pattern, worked examples, and the shared-utility, generated-code, and platform exceptions (Module Organization section) — [Workspace Organization](../reference/workspace_organization.md)
- the error types a feature module owns and re-exports — [Error Handling](../../errors/reference/error_handling.md)
