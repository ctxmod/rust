# Property, Metamorphic, and Fuzz Verification

Property and fuzz checks generate inputs instead of enumerating them. Property, metamorphic, and fuzz verification covers property kinds, metamorphic relations without an oracle, edge-biased shrinkable generators, fixed seeds and committed regression files, bounded case counts, and cargo-fuzz as deliberate exploratory evidence. Bounded property runs (fixed seeds, capped case counts) fit wherever the repository runs its tests; open-ended fuzzing and long property campaigns are deliberate verification — run them when someone chooses to, never from a hook, pre-commit, pre-push, or per-commit CI gate. The checks live wherever the repository's test policy places tests: in-crate, in a crate's `tests/` targets, or in a separate test-only or harness crate. Crate facts were verified against proptest 1.11.0, quickcheck 1.1.0, cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13, and arbitrary 1.4.2 (2026-10-04); bolero against 0.13.7 (2026-11-22). Current lines at 2026-11-22: proptest 1.12.0, arbitrary 1.5.0.

## Property Kinds

| Kind | Shape | Example |
|---|---|---|
| Round-trip | `print(parse(x)) == x` | Unedited input reprints byte-identically |
| Inverse pair | `decode(encode(v)) == v` | Escape and unescape |
| Idempotence | `f(f(x)) == f(x)` | Formatting and normalization |
| Invariant | A predicate holds on every output | Every span is on a char boundary and within `0..=len` |
| Model / oracle | `fast(x) == reference(x)` | An optimized parser agrees with a naive one |

Assert the strongest property that is still true. A round-trip property that holds only for "valid" input needs the generator to produce valid input; do not filter random bytes down to that set.

## Metamorphic Relations

When no oracle exists, transform the input in a way whose effect on the output is predictable, then check that relation:

- Append unrelated content after the document: every existing span stays the same, and spans in the appended region shift by exactly the prefix length.
- Prepend content of known byte length `d`: every span shifts by `d`.
- Reorder independent entries: the extracted set is equal, and only order-sensitive fields change.
- Duplicate an entry: the result is the same plus exactly one duplicate-key diagnostic.

A metamorphic check needs no ground truth, only a stated relation between two runs.

## Generator Design

- Generators must shrink. A failing case reduces to a minimal reproducer that someone can read. Prefer composed strategies over a hand-rolled RNG loop that cannot shrink.
- Bias toward edge cases, because uniform random input rarely reaches them: empty input, multibyte UTF-8 (2-, 3-, and 4-byte scalars at token edges), CRLF and mixed line endings, a leading BOM, deep nesting near any recursion limit, duplicate keys, and maximum-length tokens.
- Generate structure, not bytes, for properties about valid input. Keep a separate raw-byte generator for robustness invariants such as "never panics" and "returns a typed error".

## Determinism

A property run must be reproducible, or its failure is anecdote.

- Fix the seed for reproducible runs: `PROPTEST_RNG_SEED`, or `Config::rng_seed = RngSeed::Fixed(n)`. Without one, proptest generates a random seed by default. `TestRunner::deterministic()` gives the standard deterministic RNG.
- Record the failing seed with the failure report.
- proptest persists failures by default (`FileFailurePersistence::SourceParallel("proptest-regressions")`). Commit those regression files as fixtures so that a known failure replays first on every later run. Do not set `PROPTEST_DISABLE_FAILURE_PERSISTENCE` on a deliberate run whose findings you intend to keep.
- Bound the case count explicitly with `PROPTEST_CASES` or `Config::with_cases`. State the count in any claim: "1,024 cases, seed N" is evidence, and "property passed" alone is not.

Use `prop_assert!` rather than `assert!` inside properties, so a failure is reported as a test failure and goes through shrinking instead of panicking.

## Fuzzing

Fuzzing uses cargo-fuzz with libfuzzer-sys and arbitrary:

- Put fuzz targets in a separate fuzz crate, never in the production crate. Each target is `#![no_main]` with `fuzz_target!`.
- Derive structured input with `arbitrary = { version = "1", features = ["derive"] }`. The derive sits behind the `derive` feature.
- cargo-fuzz requires a nightly toolchain because it uses unstable `-Z` compiler flags for sanitizers. Pin that toolchain to the fuzz crate only. Supported platforms for this profile are Linux and macOS.
- Minimize the corpus with `cargo fuzz cmin`, and each crashing input with `cargo fuzz tmin`.
- Promote every crash to a deterministic regression case: commit the minimized input as a fixture, together with an ordinary check that replays it on the stable toolchain.

bolero is the alternative when one harness should serve both roles: a `bolero::check!()` test runs as an ordinary `#[test]` that replays the committed corpus under `cargo test`, and `cargo bolero test <target>` drives the same target under a coverage-guided engine (libFuzzer by default; AFL, honggfuzz, and Kani are selectable with `--engine`). It runs on stable with `--sanitizer NONE`; sanitizers still need nightly. Choose it over cargo-fuzz when the fuzz targets should live beside the property tests instead of in a separate crate; the regression-promotion rule below applies unchanged.

Fuzzing is exploratory evidence, not a pass/fail gate. A fuzz run that finds nothing proves nothing on its own. Never claim "fuzzed clean" without stating the target, the duration, the corpus size, and the toolchain.

## Concurrency Testing

Concurrency checks tier by what they can prove; pick the tier that matches the claim:

| Tool | What it does | What a pass proves |
|---|---|---|
| loom | Exhaustive model checking of small concurrency kernels under a deliberately unfair scheduler | Sound for the checked schedules — but by default loom downgrades sequentially-consistent accesses to Acquire/Release, so a sound-looking pass does not cover every memory-model corner |
| shuttle | Scalable randomized schedule exploration (PCT) for larger systems | Nothing — under randomization, a pass proves nothing; failures are the payload, and they reproduce deterministically |
| miri | Interprets MIR and detects undefined behavior, one execution per run | Orthogonal to scheduling: UB-freedom on the executed path only, no schedule coverage |

The split is the endorsed shape: loom for sound-but-small exhaustive kernels, shuttle where the system outgrows exhaustive checking, miri for UB — and none of them replaces the product-side discipline (schedule-independent reductions, the worker-count sweep) in the determinism reference.

The citation form is mandatory for every property claim, concurrency checks included: state the seed and the case count ("1,024 cases, seed N" is evidence; "property passed" alone is not), and for loom, the schedule bound explored.

## Sketch

This is a sketch, not a drop-in implementation:

```rust
proptest! {
    #![proptest_config(Config::with_cases(1024))]
    #[test]
    fn append_preserves_spans(doc in doc_strategy(), tail in tail_strategy()) {
        let a = extract(&doc);
        let b = extract(&format!("{doc}{tail}"));
        prop_assert_eq!(a.spans(), b.spans_before(doc.len()));
    }
}
```

## Related

- Runner mechanics (rstest, cargo-nextest) and isolation principles for these checks: [Test Runners](test_runners.md)
- The span, round-trip, and char-boundary invariants that properties most often assert: Source Spans and Permissive Extraction (in the `data_integrity` profile)

## Sources

- https://github.com/camshaft/bolero — `bolero::check!`, `cargo bolero test` (bolero 0.13.7, 2026-10-03; accessed 2026-11-22)
- https://github.com/camshaft/bolero/blob/master/book/src/features/unified-interface.md, corpus-replay.md, rust-stable.md — engines and `--engine`, corpus replay under `cargo test`, stable runs with `--sanitizer NONE` (accessed 2026-11-22)
- https://crates.io/crates/proptest and https://crates.io/crates/arbitrary — current versions 1.12.0 (2026-10-10) and 1.5.0 (2026-10-06) (accessed 2026-11-22)
