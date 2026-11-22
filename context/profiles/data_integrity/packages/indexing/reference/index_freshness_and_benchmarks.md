# Index Freshness and Benchmarks

An incremental index is correct only if every reuse is justified by evidence that its inputs have not changed. A benchmark is evidence only if someone else could reproduce the conditions it ran under. Crate and std facts were verified against blake3 1.8.7, criterion 0.8.2, divan 0.1.21, and std 1.99.0 (2026-10-04).

## Lazy Layers

Build a derived layer, such as a symbol table, a reverse-reference map, or a full-text postings list, only when a query needs it. Each layer is a cache over its inputs, never a source of truth: deleting any layer must leave the system correct, only slower.

## Input Fingerprints

Every derived layer records the fingerprints of the inputs it was built from:

- **Content hash.** This is authoritative. Equal hashes mean equal inputs.
- **Metadata tuple `(size, mtime, inode)`.** This is a cheap pre-check. If the tuple matches, the input is presumed unchanged. If it differs, recompute the content hash before declaring a change, because touch, checkout, and copy operations alter metadata without altering content.

The metadata pre-check must tolerate its known weaknesses, so treat each of these as a requirement:

- Filesystem mtime granularity varies. A write that lands within the same granularity window as the recorded stamp must not be missed.
- Clocks can step backwards. Compare for inequality, never "newer than".
- Inodes are reused after deletion. The tuple is only a hint and never replaces the content hash.

### Fingerprint Taxonomy

Name the kind of every persisted hash alongside the algorithm identifier — the kind tells a reader what question the hash answers, and the stamp tells them how it was computed:

| Kind | Answers |
|---|---|
| Source | Did the input bytes change? |
| Semantic | Did the derived meaning change? |
| Provenance | Which producer, toolchain, and flags made this? |
| Recipe | Did the build steps change? |
| Artifact | Is this the exact output recorded? |

The strength gate follows the kind: a fast non-cryptographic hash may skip re-extraction only for performance-only consumers; every safety-relevant flow first confirms the cryptographic hash or compares the bytes. The canonical-data reference generalizes the stamp to algorithm + hash domain.

## Stable Hashing

std's `DefaultHasher` documentation says: "The internal algorithm is not specified, and so it and its hashes should not be relied upon over releases." Never persist a `DefaultHasher` output. A toolchain upgrade can silently invalidate every stored fingerprint, or worse, make unrelated inputs appear to match.

Use a specified, pinned hash for anything that is persisted, such as `blake3::hash(input: &[u8]) -> Hash`. Record the hash algorithm identifier in the layer's envelope next to the fingerprints. A reader that finds an unknown or different algorithm identifier treats the layer as stale and rebuilds it.

## Coverage Is Not Absence

A scan reports what it saw, not what exists. A partial scan must never be read as deletion. Scans are partial in these cases:

- the scan was filtered (by glob, scope, or tag);
- the scan was interrupted (by cancellation, timeout, or a crash mid-walk);
- a subtree returned permission-denied or another I/O error;
- the path lies outside the indexed coverage.

Tombstone an entry only on positive evidence of absence within a scope that was fully covered on that pass. A coverage error is a distinct outcome from an empty result. Return an explicit "not covered" or "incomplete" state, never an empty list that a caller would read as "nothing matches".

## Staleness and Invalidation

- Staleness checks are cheap and run before every reuse. Compare recorded fingerprints with current metadata, and hash only on mismatch.
- Invalidation cascades. When a base layer changes, every layer derived from it is stale too, so track layer dependencies explicitly rather than relying on rebuild order.
- A full rebuild is always a correct fallback. On a corrupt envelope, an unknown algorithm identifier, or an unexplained mismatch, rebuild rather than repair in place.

### The Invalidation Matrix

The kind → trigger → outcome mapping is a maintained design artifact, not tribal knowledge: for each fingerprint kind (the taxonomy above), write down which triggers invalidate (source byte change, recipe change, producer-version change, explicit flush) and which outcome each produces (typed error, rebuild, or ignore). Conservative over-invalidation is correct; false reuse is a defect. The outcome set aligns with the persistence-envelopes corruption posture — cold / truncated / foreign-version / stale each resolve to a typed outcome or a rebuild — so a reader of either document resolves the same states.

## Benchmarks

Benchmarks live in Cargo `benches/` targets declared with `harness = false`, because the built-in `#[bench]` attribute is nightly-only:

- **criterion** uses `criterion_group!` and `criterion_main!`.
- **divan** uses a `main` that calls `divan::main()`, with `#[divan::bench(args = [...])]` on each benchmark.

Use `std::hint::black_box`, which is stable since Rust 1.66, to keep the optimizer from deleting measured work. `criterion::black_box` is deprecated. std documents `black_box` as "best-effort" only, so a surprisingly fast result still needs inspection.

Benchmarks are deliberate measurement. They never run in a hook, a pre-commit or pre-push check, or a CI gate.

Measure each layer's cost separately, on fixed generated corpora whose size you record (file count, total bytes, symbol count):

| Scenario | Measures |
|---|---|
| Cold build | All layers from an empty cache |
| Warm reuse | Staleness checks only; no layer rebuilt |
| Incremental | Rebuild after changing N files (state N) |

Report the environment with every result: CPU model, system load, and toolchain version. Compare results only under matched conditions. A ratio claim such as "incremental is 40x faster than cold" needs repeated runs and must state their count and spread. A single run is anecdote.

## Benches and the Test Policy

Benches are measurement, not tests, and a repository whose test policy scans test targets may leave `benches/` unscanned for that reason. That exemption only holds if benches stay honest. A bench must not contain assertions that act as pass/fail checks. If something needs to be asserted, it belongs in a test, a black-box test program, or a property check.

## Sketch

This is a sketch, not a drop-in implementation:

```rust
struct LayerEnvelope {
    hash_algo: &'static str,         // e.g. "blake3"
    inputs: Vec<(PathBuf, Meta, [u8; 32])>,
}

enum Lookup<T> { Hit(T), NotCovered, Incomplete { reason: String } }
```

## Related

- The envelope that carries version and algorithm identifiers for persisted layers: [Persistence Envelopes for Zero-Copy Archives](../../persistence/reference/persistence_envelopes.md)
- Runner mechanics and isolation principles for the checks around the benches: [Test Runners](../../../../verification/packages/property/reference/test_runners.md)
- Property checks for index invariants, such as rebuild-equals-incremental: [Property, Metamorphic, and Fuzz Verification](../../../../verification/packages/property/reference/property_testing.md)
