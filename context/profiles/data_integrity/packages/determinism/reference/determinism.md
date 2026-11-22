# Determinism

Identical inputs produce identical semantic outputs regardless of worker count or completion order; byte outputs are additionally identical under a pinned writer profile. Schedule independence is a design property of the product, not a test-harness setting; this reference carries the contract, the proof, and the order discipline.

## The Schedule-Independence Contract

- The contract has two clauses. Semantic: the set and content of outputs (events, diagnostics, emitted records) is identical for every schedule. Byte: the serialized output is identical under a pinned writer profile — same toolchain, same flags, canonical encoding (the canonical-data reference owns that profile).
- Parallel reductions are the hazard. Rayon's documented contract is the reference case: parallel iterators match sequential results except for side-effect ordering, and `reduce` makes no promise about combination order. A reduction is schedule-independent only if the operation is associative with a true identity — or if the reduction runs over total-ordered keys whose tie-break is decided after semantic ranking, never by arrival order.
- Floating-point addition is the canonical counterexample: non-associative, so any parallel fold over floats is schedule-dependent by construction. Float accumulation never carries identity values or exact quantities; exact sums move to the integer or decimal domain (the canonical-data reference owns the numeric-domain rule).
- Stable tie ordering is decided after semantic ranking: rank on meaning first, then break ties on a total order (key, then offset, then a declared constant) — the order in which workers happened to finish is never an input to the decision.

## The Worker-Count Sweep

The standing proof is a metamorphic relation: run the work under 1..N workers and require output identity at every count.

- The sweep is cheap and high-yield: `RAYON_NUM_THREADS=1..N` (or the equivalent pool parameter), diff the outputs. It catches arrival-order tie-breaks, lost reductions, and order-sensitive accumulation in minutes.
- What it does not prove: it samples schedules, it does not exhaust them, and it does not cover interleavings inside a single operation. Sound-but-small exhaustive checking of scheduling-sensitive internals belongs to loom; scalable randomized exploration to shuttle; UB detection to miri — the tiering and its pass-semantics live in the property-testing reference's concurrency section.
- Run the sweep as a deliberate check (a black-box test program or an implementation-coupled test), not a hook; a green sweep is stated with the worker counts it covered.

## Chosen Order Everywhere

Iteration order over persisted or emitted data is a decision, never an inheritance:

- Sorted vectors instead of hash-map iteration: `std`'s `HashMap` uses a randomized hasher precisely so its iteration order varies between runs — any output that walks one has absorbed that variance.
- Explicit byte order for emitted sequences and diagnostics: sort by (semantic rank, key, offset) before serialization. Diagnostics are emitted once, after the work stabilizes, in that byte order — not streamed as discovered.
- No volatile fields in persisted payloads: timestamps, environment captures, host identifiers, and durations are metadata recorded beside the payload, never inside it — a payload whose bytes depend on when it was written cannot be compared, cached, or rebuilt-and-hashed.
- Where a closure algorithm converges to a fixed point, order-independence follows from the fixed point — state that argument explicitly when relying on it, and keep the convergence check deterministic anyway.

## Reproducible Evidence

- Every property and fuzz claim states its seed and case count — "1,024 cases, seed N" is evidence; "property passed" is an anecdote. Fixed seeds, committed regression files, bounded case counts: the property-testing reference owns the mechanics.
- Where a run carries a timestamp, it is a fixed, recorded timestamp, not a fresh clock read.
- Expected values contain no concrete scratch paths — a golden that embeds this machine's temporary directories is evidence about the machine, not the program.
- Parallelism is an explicit parameter of the code that owns it: a pool or worker count passed by the caller, visible in the signature or configuration. No hidden global thread pool — a program whose parallelism cannot be set to one cannot be swept, and code that cannot be swept cannot claim schedule independence.

## Related

- Property, metamorphic, and concurrency-check mechanics including the loom/shuttle/miri tiering: [Property, Metamorphic, and Fuzz Verification](../../../../verification/packages/property/reference/property_testing.md)
- The canonical-encoding and writer-profile half of byte identity: [Canonical Data](../../persistence/reference/canonical_data.md)
- Fingerprint stability for persisted derived layers: [Index Freshness and Benchmarks](../../indexing/reference/index_freshness_and_benchmarks.md)
