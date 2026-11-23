# Lint Enforcement

Rust lint enforcement uses Clippy as the unified linter with selective pedantic configuration, cargo-deny for dependency licensing and advisories, and Cargo's own lints (unused dependencies since Cargo 1.100) with cargo-machete as the complement for unused-dependency detection. The commands and CI blocks below enforce the same zero-warning policies whether they run directly or behind a repository's own check wrapper; when a repository routes lints through such a wrapper, its gate is the wrapper and the raw cargo shape is what the wrapper runs.

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
map_unwrap_or = "warn"
match_bool = "warn"
match_same_arms = "warn"
missing_errors_doc = "allow"  # Often noisy for internal code
missing_panics_doc = "allow"  # Often noisy for internal code
redundant_else = "warn"
semicolon_if_nothing_returned = "warn"
uninlined_format_args = "warn"
```

(`map_flatten`, sometimes listed here, is a `complexity` lint already armed by `clippy::all`.)

### Cargo Lints

Cargo has its own lint table, inherited the same way: `[workspace.lints.cargo]` in the root and `[lints] workspace = true` in members. Since Cargo 1.100 the warn-by-default `unused_dependencies` lint reports `[dependencies]` entries the crate never uses (it does not cover `dev-dependencies`), and `unused_workspace_dependencies` reports `[workspace.dependencies]` entries no member consumes; `missing_lints_inheritance` is the warn-by-default lint behind the inheritance rule above. Deny them where the dependency set is policy:

```toml
[workspace.lints.cargo]
unused_dependencies = "deny"
unused_workspace_dependencies = "warn"
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
# Vulnerability and notice advisories always error; the old
# vulnerability/notice/unlicensed/copyleft/default keys are rejected.
yanked = "deny"
unmaintained = "workspace"   # all | workspace (default) | transitive | none
ignore = [
    { id = "RUSTSEC-2024-0001", reason = "low severity, no patch available, tracked in #123" },
]

[licenses]
# Every license not listed is denied.
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
]

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

cargo-deny fetches the RustSec advisory database (into `$CARGO_HOME/advisory-dbs` by default) on each run. When fetching is disabled for an offline run, `maximum-db-staleness` (default `P90D`) bounds how old the local copy may be before the check fails rather than passing on stale data.

## cargo-machete

cargo-machete detects unused dependencies by searching crate sources for each dependency's import name. With Cargo's `unused_dependencies` lint (1.100+) covering `[dependencies]` from compiler information, cargo-machete remains useful for `dev-dependencies` and `build-dependencies`, and for workspaces whose toolchain floor predates 1.100. Run it in CI to prevent dependency drift.

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

cargo-machete reads an ignore list from manifest metadata — comments are not a suppression mechanism:

```toml
[package.metadata.cargo-machete]
ignored = ["prost"]   # used only by build.rs-generated code

# or once for the whole workspace, in the root manifest
[workspace.metadata.cargo-machete]
ignored = ["prost"]
```

For a crate whose import name differs from its package name, prefer the `renamed` table over `ignored`, so a later genuinely-unused state is still detected; `--with-metadata` resolves names through `cargo metadata` at the cost of possibly rewriting `Cargo.lock`. Only `build.rs` itself is searched for build-dependencies, not modules it loads.

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
2. Remove from Cargo.toml, or add it to `package.metadata.cargo-machete.ignored` with a comment naming the use
3. Re-run cargo-machete to verify

## Exception Handling

### Lint Suppression with `#[expect]`

Suppress a specific lint with `#[expect(..., reason = "...")]` (Rust 1.81+) rather than `#[allow]`: `expect` itself warns (`unfulfilled_lint_expectations`) once the suppressed lint no longer fires, so stale suppressions surface instead of accumulating, and the `reason` string is printed in the compiler message, keeping the rationale with the code.

```rust
#[expect(clippy::unwrap_used, reason = "index validated by the caller's bounds check")]
fn internal_helper() {
    // ...
}
```

Keep `#[allow]` for suppressions that are legitimately conditional (a lint that fires under one `cfg` or feature set only), still with a `reason`. Two restriction lints make this mechanical: `clippy::allow_attributes_without_reason` requires a `reason` on every `allow`, and `clippy::allow_attributes` rejects `#[allow]` in favor of `#[expect]`.

### cargo-deny Exceptions

Allow specific advisories or licenses in `deny.toml`:

```toml
[advisories]
ignore = [
    { id = "RUSTSEC-2024-0001", reason = "low severity, no patch available, tracked in #123", expiry = "2027-03-31" },
]

[licenses]
exceptions = [
    { allow = ["GPL-3.0"], crate = "internal-tool" },  # scoped to one crate, not workspace-wide
]
```

Carry the justification in `reason` (it is reported with the finding); an `expiry` turns a forgotten exception back into a failure.

### cargo-machete False Positives

List dependencies static search cannot see in `[package.metadata.cargo-machete] ignored`, with a comment naming where the use lives (see False Positive Handling above).

## Related

[Workspace Organization](../../workspace/reference/workspace_organization.md)
[Error Handling](../../errors/reference/error_handling.md)
The rust `core` profile's `layered_workspace` example carries a complete virtual-manifest workspace.

## Sources

- https://doc.rust-lang.org/cargo/reference/lints.html — Cargo lints: `unused_dependencies` (style, warn), `unused_workspace_dependencies`, `missing_lints_inheritance` (suspicious, warn) (accessed 2026-11-22)
- https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/ — unused dependency warnings controlled by `[lints.cargo]`, not covering dev-dependencies (Rust 1.100.0, 2026-11-12; accessed 2026-11-22)
- https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/ — the `expect` lint level and lint reasons, `clippy::allow_attributes` and `allow_attributes_without_reason` (Rust 1.81.0; accessed 2026-11-22)
- https://clippy.rust-lang.org/stable/index.html — lint groups (`map_flatten` is complexity; the selective list above is pedantic; the two allow-attribute lints are restriction) (accessed 2026-11-22)
- https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html — version-2 advisories schema, removed `vulnerability`/`notice` keys, `unmaintained` scopes, `ignore` with `reason`/`expiry`, `maximum-db-staleness` (cargo-deny 0.22.0 current; accessed 2026-11-22)
- https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html — removed `unlicensed`/`copyleft`/`default`/`deny` keys, `allow` and `exceptions` (accessed 2026-11-22)
- https://github.com/bnjbvr/cargo-machete — `package.metadata.cargo-machete.ignored`, `workspace.metadata`, `renamed`, `--with-metadata`, build.rs search scope (cargo-machete 0.10.1; accessed 2026-11-22)
