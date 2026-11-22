# Test Runners

The runner mechanics for Rust tests: rstest fixtures and parameterization, cargo-nextest execution, doctests, and the isolation principles that keep runs trustworthy. Where tests live — in-crate `#[cfg(test)]` modules (the ecosystem idiom), crate `tests/` targets, a separate test-only crate, or black-box programs that run the built binary — is the repository's test policy; these mechanics apply under any placement.

**Doctests are documentation first.** Doc-comment examples validate API usage as prose and run under `cargo test --doc`; they complement tests of behavior rather than replace them.

**Test-only APIs go in a sibling crate, never behind a feature flag.** Feature-gated test-only APIs leak into production builds through feature unification (the tokio#4035 lesson); when a durable test-only surface is needed at workspace scale, make it a separate workspace member.

## rstest Fixtures

rstest is optional mechanics, adopted by adding it to the test crate's `[dev-dependencies]` (or `[dependencies]` of a dedicated test-only crate): it provides fixture injection via `#[fixture]` and parameterization via `#[case]`, eliminating shared mutable state that causes ordering dependencies and flakiness.

### Basic Fixtures

Define fixtures with `#[fixture]` attribute. Inject into tests via function parameters.

```rust
use rstest::*;
use my_crate::Calculator;

#[fixture]
fn calculator() -> Calculator {
    Calculator::new()
}

#[rstest]
fn test_fresh_calculator(calculator: Calculator) {
    assert!(calculator.history().is_empty());
}
```

### Fixtures with State

Fixtures can build complex state. Each test receives an independent instance.

```rust
#[fixture]
fn calculator_with_history() -> Calculator {
    let mut calc = Calculator::new();
    calc.add(1, 2).unwrap();
    calc.multiply(3, 4).unwrap();
    calc
}

#[rstest]
fn test_calculator_with_history(calculator_with_history: Calculator) {
    let history = calculator_with_history.history();
    assert_eq!(history.len(), 2);
}
```

### Async Fixtures

Async fixtures work with tokio::test. Mark the fixture parameter with `#[future]`.

```rust
#[fixture]
async fn async_calculator() -> Calculator {
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    Calculator::new()
}

#[rstest]
#[tokio::test]
async fn test_async_fixture(#[future] async_calculator: Calculator) {
    let calc = async_calculator.await;
    assert!(calc.history().is_empty());
}
```

## Parameterized Tests

Parameterized tests run the same logic across multiple inputs via `#[case(...)]` attributes. This replaces hand-written test tables with declarative test cases.

```rust
#[rstest]
#[case(2, 3, 5)]
#[case(0, 0, 0)]
#[case(-1, 1, 0)]
#[case(100, -50, 50)]
fn test_addition_cases(#[case] a: i64, #[case] b: i64, #[case] expected: i64) {
    let mut calc = Calculator::new();
    assert_eq!(calc.add(a, b).unwrap(), expected);
}
```

Error-case parameterization follows the same shape: one `#[case]` per row, and a `matches!` assertion on the expected error variant.

## cargo-nextest Execution

cargo-nextest accelerates test execution via process-level parallelization — each test runs in isolation, improving reliability and speed. Any test crate may adopt it (`cargo nextest run`); where it is not installed, plain `cargo test` is the runner and the verdict semantics are identical. Configuration lives in `.config/nextest.toml` under the workspace root — the only repository path nextest reads (`--config-file` overrides it); a root-level `.nextest.toml` is silently ignored, so its profiles never take effect. Profiles inherit from `default` (and, since nextest 0.9.115, from a named parent), and per-user UI preferences belong in `~/.config/nextest/config.toml`, not the repository.

### Compatibility Notes

- rstest `#[once]` fixtures are process-wide statics: under nextest every test runs in its own process, so a `#[once]` fixture is rebuilt per test and shares nothing, and it is never dropped (so it cannot own anything that needs cleanup) — never use it to carry state between tests, and do not count on it to amortize expensive setup under nextest
- Use `#[fixture]` without `#[once]` for per-test isolated state
- Each test receives fresh fixture instances in separate processes
- Archive-and-run (`cargo nextest archive`, then `cargo nextest run --archive-file`) and partitioning (`--partition slice:1/4`, or `hash:1/4` for buckets stable as tests change; `count:` is deprecated) split one build across CI machines without rebuilding

Doctests are not run by nextest: run `cargo test --doc` alongside it when doctests matter.

## Test Isolation Principles

These apply to every test surface, from in-crate unit tests to black-box test programs; they are what keeps test runs trustworthy.

### Prohibit Global Mutable State

Global mutable state causes ordering dependencies. Tests fail when run in different sequences.

**Incorrect**:

```rust
static mut COUNTER: i32 = 0;  // Unsafe, causes flaky tests
```

**Correct**:

```rust
#[fixture]
fn counter() -> i32 {
    0
}
```

### Mocks vs Real Dependencies

- **Tests that reach into implementation** (unit and implementation-coupled tests): mocks acceptable for external services and flaky dependencies; real dependencies preferred for persistence-layer and critical-path questions
- **Black-box tests of the built program**: real dependencies always — the point is evidence about the real program

## Validation Ladder

Triage a Rust finding narrowest-first, escalating only when the narrower scope is clean (the rust-analyzer script ladder, mirrored here): per-crate check (`cargo check -p <crate>`) → per-crate clippy (`cargo clippy -p <crate> --all-targets -- -D warnings`) → workspace lint (`cargo clippy --workspace --all-targets -- -D warnings`) → any repository-level file scans (for example a test-placement policy scanner).

[Workspace Organization](../../../../core/packages/workspace/reference/workspace_organization.md)

[Error Handling](../../../../core/packages/errors/reference/error_handling.md)

The rust `verification` profile's `test_fixtures` example applies these mechanics to a small crate.

## Sources

- https://nexte.st/docs/configuration/ — repository config at `.config/nextest.toml` relative to the workspace root, `--config-file`, the configuration hierarchy, user config, profile inheritance since 0.9.115 (cargo-nextest 0.9.152 current, 2026-11-13; accessed 2026-11-22)
- https://nexte.st/docs/ci-features/archiving/ and https://nexte.st/docs/ci-features/partitioning/ — archive-and-run and test partitioning (accessed 2026-11-22)
- https://docs.rs/rstest/latest/rstest/attr.fixture.html — `#[once]` fixtures return a static reference that is never dropped; not allowed on async or generic fixtures (rstest 0.29.0; accessed 2026-11-22)
