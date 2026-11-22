# Workspace Cargo.toml Example

Demonstrates Cargo workspace organization with virtual manifest and layer-based crate separation.

## Pattern Demonstrated

This example shows a 4-crate workspace organized by dependency layers:

```
layered_workspace/
├── Cargo.toml          # Virtual workspace manifest
├── pkg/rs/
│   ├── utils/          # Layer 0: No internal dependencies
│   ├── types/          # Layer 1: Domain types (depends on utils)
│   ├── domain/         # Layer 2: Business logic (depends on types, utils)
│   └── app/            # Layer 3: Binary application (depends on all)
```

## Workspace Principles

1. **Virtual manifest**: Root `Cargo.toml` declares workspace members without `[package]` section
2. **Layer-based separation**: Strict DAG structure enforced at compile time (circular dependencies fail)
3. **Workspace dependencies**: Unified third-party versions in `[workspace.dependencies]`
4. **Feature-based modules**: Within crates, organize by domain features

## Running the Example

```bash
# Build entire workspace
cargo build

# Run binary crate
cargo run --bin app

# Check dependency graph
cargo tree --workspace
```

The members live under `pkg/rs/`, the layout of a polyglot monorepo that keeps each language's packages under `pkg/<language>/`; a Rust-only repository commonly uses `crates/`. The example carries no tests, so it reads the same under any test-placement policy; the rust `verification` profile's test-fixtures example shows the runner mechanics.

## References

- Workspace Organization: the rust `core` profile, `workspace` package, `reference/workspace_organization.md`
- Lint Enforcement: the rust `core` profile, `lints` package, `reference/lint_enforcement.md`
