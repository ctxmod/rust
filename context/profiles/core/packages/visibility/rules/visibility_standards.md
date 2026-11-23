---
paths:
  - "*.rs"
---

# Visibility Standards

All types private by default. Add `pub` only when external code must access the item, keep fields private behind constructors rather than exposing internal representation, and flatten the public surface through explicit `pub use` re-exports at the module root — internal modules stay private. Apply `#[non_exhaustive]` to public enums and structs so future variants and fields are additive, not breaking.

Exceptions: test helpers may use `pub(crate)` for visibility within the crate; FFI boundaries require `pub` for linkage but may document internal status through naming conventions.

- the private-by-default and `#[non_exhaustive]` worked examples, the module re-export pattern, and the test-utility and FFI exceptions — [Visibility Standards](../reference/visibility.md)
- the workspace layering that bounds which crates can see an API surface — [Workspace Organization](../../workspace/reference/workspace_organization.md)
- error enums carrying `#[non_exhaustive]` for API evolution — [Error Handling](../../errors/reference/error_handling.md)
