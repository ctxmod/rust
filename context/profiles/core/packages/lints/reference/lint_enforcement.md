# Lint Enforcement

Rust lint enforcement uses Clippy as the unified linter with selective pedantic configuration, cargo-deny for dependency licensing and advisories, and cargo-machete for unused dependency detection. The commands and CI blocks below enforce the same zero-warning policies whether they run directly or behind a repository's own check wrapper; when a repository routes lints through such a wrapper, its gate is the wrapper and the raw cargo shape is what the wrapper runs.

## Lint Inheritance

Declare lints once in the root manifest's `[workspace.lints]` and opt in per member with `[lints] workspace = true`. Inheritance is NOT implicit: cargo warns via `missing_lints_inheritance` when a member has no `[lints]` table at all, and via a companion warn when a workspace defines a lints table no member consumes. A conservative landed baseline:

```toml
# Cargo.toml (workspace root)
[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
```

```toml
# every member manifest, e.g. tools/<tool>/Cargo.toml
[lints]
workspace = true

[package]
# ...
autotests = false
```

The root table carries the same group at the same level a clippy wrapper might arm on the command line for undeclared crates (`-W clippy::all`), so for this warn-level baseline the finding set is unchanged; nothing is elevated to deny, and no pedantic/nursery groups are armed. A wrapper should pass that flag only when a linted crate has no `[lints]` table: a command-line level outranks every manifest level, so passing it to declaring crates would override their `allow` and `deny` entries. In-source `#![allow]`/`#![deny]` attributes outrank both. Members may also set `autotests = false`: no auto-discovered `tests/` targets (a deterministic implicit target set; reversible by deleting the key). It does not disable the implicit lib test harness — whatever test-placement policy a repository keeps is enforced by its own checks, never by cargo target discovery.

## Clippy Configuration

Clippy serves as the unified linting standard, replacing meta-linter approaches. Configure Clippy with `clippy::all` at deny level and selective `clippy::pedantic` lints. The `clippy::restriction` category requires cherry-picking due to internal contradictions.

### Base Configuration

Create `.clippy.toml` or configure in `Cargo.toml`:

```toml
# Cargo.toml
[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }

# Restriction lints (cherry-picked)
unwrap_used = "deny"
expect_used = "warn"
todo = "deny"
unimplemented = "deny"
panic = "deny"
```

### Selective Pedantic Enablement

Enable individual pedantic lints after team evaluation. Avoid blanket pedantic deny due to false positive rates.

```toml
[lints.clippy]
# Recommended pedantic lints with low false positive rates
enum_glob_use = "warn"
explicit_into_iter_loop = "warn"
explicit_iter_loop = "warn"
filter_map_next = "warn"
flat_map_option = "warn"
implicit_clone = "warn"
inconsistent_struct_constructor = "warn"
map_flatten = "warn"
map_unwrap_or = "warn"
match_bool = "warn"
match_same_arms = "warn"
missing_errors_doc = "allow"  # Often noisy for internal code
missing_panics_doc = "allow"  # Often noisy for internal code
redundant_else = "warn"
semicolon_if_nothing_returned = "warn"
uninlined_format_args = "warn"
```

### CI Enforcement

Enforce Clippy in CI with `cargo clippy -- -D warnings`. All warnings become errors.

```yaml
# .github/workflows/lint.yml
- name: Run Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

### Formatting

Standalone rustfmt defaults to the 2015 style edition when no config is found, so the repo root carries `rustfmt.toml` pinning `style_edition = "2024"` — matching the crates' edition 2024 and rust's own posture. No other rules are set: rustfmt defaults are the style contract. Gate formatting with `cargo fmt --all --check` beside the lint stages; `cargo fmt` derives `--edition` from `Cargo.toml` for its own invocations, and the pin covers every other rustfmt entry point (editors, standalone runs).

### Invocation and Escalation

The workspace invocation is `cargo clippy --workspace --all-targets -- -D warnings` (or the repository's wrapper around it). Triage a finding narrowest-first, escalating only when the narrower scope is clean: per-crate check (`cargo check -p <crate>`) → per-crate clippy (`cargo clippy -p <crate> --all-targets`) → workspace lint → any repository-level file scans.

### Rationale References

Clippy lints enforce principles documented in workspace organization and error handling principles.

[Workspace Organization](../../workspace/reference/workspace_organization.md)
[Error Handling](../../errors/reference/error_handling.md)

## cargo-deny

cargo-deny validates dependencies for licensing compliance, security advisories, and banned crates. Configure via `deny.toml` and run in CI.

### Installation

```bash
cargo install cargo-deny
```

### Configuration

Create `deny.toml` at workspace root:

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
]
copyleft = "warn"
default = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### CI Integration

```yaml
# .github/workflows/audit.yml
- name: Check dependencies
  run: cargo deny check
```

### Advisory Database

cargo-deny queries the Rust Security Advisory Database. Update regularly:

```bash
cargo deny check --update-advisory-db
```

## cargo-machete

cargo-machete detects unused dependencies in Cargo.toml files. Remove false positives via inline comments. Run in CI to prevent dependency drift.

### Installation

```bash
cargo install cargo-machete
```

### Usage

```bash
# Check all crates in workspace
cargo machete

# Fix automatically (review changes before committing)
cargo machete --fix
```

### False Positive Handling

Mark dependencies as used with inline comments:

```toml
[dependencies]
# used-by: build.rs
prost-build = "0.12"

# used-by: tests/
tokio-test = "0.4"
```

### CI Integration

```yaml
# .github/workflows/lint.yml
- name: Check unused dependencies
  run: cargo machete
```

## Step-by-Step Validation Workflow

### Local Development

1. **Run Clippy before commits**:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

2. **Check dependencies periodically**:
   ```bash
   cargo deny check
   cargo machete
   ```

3. **Fix unused dependencies**:
   ```bash
   cargo machete --fix
   git diff  # Review changes
   git commit -am "chore: remove unused dependencies"
   ```

### CI Pipeline

1. **Clippy check**:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

2. **Security audit**:
   ```bash
   cargo deny check
   ```

3. **Unused dependency detection**:
   ```bash
   cargo machete
   ```

### Addressing Failures

**Clippy warning**:
1. Read lint message and documentation (`cargo clippy --explain LINT_NAME`)
2. Fix code or add allow annotation with rationale
3. Re-run Clippy locally before pushing

**cargo-deny advisory**:
1. Review advisory details at <https://rustsec.org>
2. Update dependency to patched version if available
3. If no patch exists, evaluate severity and add exception with justification

**cargo-machete failure**:
1. Verify dependency is truly unused (check build scripts, tests, conditional compilation)
2. Remove from Cargo.toml or add `# used-by:` comment with justification
3. Re-run cargo-machete to verify

## Exception Handling

### Clippy Allow Annotations

Suppress specific lints with allow attributes when justified:

```rust
#[allow(clippy::unwrap_used)]
fn internal_helper() {
    // Unwrap acceptable in test helpers with invariant guarantees
}
```

Document rationale in code comments or commit messages.

### cargo-deny Exceptions

Allow specific advisories or licenses in `deny.toml`:

```toml
[advisories]
ignore = [
    "RUSTSEC-2024-0001",  # Low severity, no patch available, tracked in #123
]

[licenses]
allow = ["GPL-3.0"]  # Internal tool, GPL compatible
```

Document justification in comments.

### cargo-machete False Positives

Mark dependencies as used when static analysis cannot detect usage:

```toml
[build-dependencies]
# used-by: build.rs for code generation
prost-build = "0.12"
```

## Related

[Workspace Organization](../../workspace/reference/workspace_organization.md)
[Error Handling](../../errors/reference/error_handling.md)
The rust `core` profile's `layered_workspace` example carries a complete virtual-manifest workspace.
