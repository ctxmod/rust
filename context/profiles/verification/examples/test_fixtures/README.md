# Test Fixtures Example

Demonstrates Rust testing mechanics — rstest fixtures, parametrized tests, and cargo-nextest configuration. The library crate holds only production code and a doctest; the tests live in the crate's `tests/` target and pull rstest in as a dev-dependency, so nothing test-only reaches the library's consumers. Where a repository's test policy keeps tests out of production crates altogether, move `tests/` and the `[dev-dependencies]` table into a separate test-only crate that depends on the library — never behind a feature flag. Doctests are documentation first: they validate the usage examples in doc comments.

## Mechanics Demonstrated

1. **rstest fixtures**: Reusable test setup and teardown
2. **Parametrized tests**: Test multiple cases with single test function
3. **cargo-nextest**: Fast, parallel test runner with better output

## Crate Structure

```
test_fixtures/
├── Cargo.toml                  # Workspace manifest (edition 2024, resolver 3)
├── .config/nextest.toml        # Nextest configuration (the path nextest reads)
└── pkg/rs/test-fixtures/
    ├── Cargo.toml              # Crate manifest (rstest as a dev-dependency)
    ├── src/
    │   ├── lib.rs              # Library root with re-exports
    │   └── calculator.rs       # Calculator module (doctest-annotated)
    └── tests/
        └── calculator.rs       # rstest fixtures and #[case] parametrization
```

## rstest Features

### Fixtures

Define reusable test context:

```rust
#[fixture]
fn calculator() -> Calculator {
    Calculator::new()
}

#[rstest]
fn fresh_calculator_has_no_history(calculator: Calculator) {
    assert!(calculator.history().is_empty());
}
```

### Parametrized Tests

Test multiple cases in one function:

```rust
#[rstest]
#[case(2, 3, 5)]
#[case(0, 0, 0)]
#[case(-1, 1, 0)]
fn addition(mut calculator: Calculator, #[case] a: i64, #[case] b: i64, #[case] expected: i64) {
    assert_eq!(calculator.add(a, b).unwrap(), expected);
}
```

## cargo-nextest Benefits

- **Speed**: Runs tests in parallel by default
- **Better output**: Clean, actionable failure messages
- **Retry support**: Automatically retry flaky tests
- **Partitioning**: Split test runs across jobs

## Running

```bash
# Run the tests and the doctest with plain cargo
cargo test

# Install nextest
cargo install cargo-nextest

# Run tests with nextest (doctests are not run by nextest: add cargo test --doc)
cargo nextest run

# Run specific test
cargo nextest run addition

# Run with verbose output
cargo nextest run --no-capture

# Generate JUnit report
cargo nextest run --profile ci
```

## References

- Test Runners: the rust `verification` profile, `property` package, `reference/test_runners.md`
- Error Handling: the rust `core` profile, `errors` package, `reference/error_handling.md`
- Workspace Organization: the rust `core` profile, `workspace` package, `reference/workspace_organization.md`
