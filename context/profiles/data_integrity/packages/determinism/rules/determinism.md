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

# Determinism

Outputs are independent of scheduling: identical inputs produce identical semantic outputs regardless of worker count or completion order; byte outputs are identical under a pinned writer profile.

- Parallel reductions are provably schedule-independent: an associative operation with a true identity, or reduction over total-ordered keys whose tie-break is decided after semantic ranking, never by arrival order.
- Floating-point accumulation is non-deterministic under parallel reduction and never carries identity or exact quantities.
- Prove it with the worker-count sweep — output identity across 1..N workers — as a standing metamorphic relation; loom, shuttle, and miri cover scheduling-sensitive internals when they exist.
- Iteration order over persisted data is chosen, not inherited: sorted vectors instead of hash-map iteration, explicit byte order for emitted sequences and diagnostics, no volatile fields in persisted payloads.
- Parallelism is an explicit parameter of the code that owns it; no hidden global thread pool.
- Property and fuzz runs are reproducible — fixed seed, committed regressions, stated seed and case count — or their failures are anecdotes.
- the schedule-independence contract, the worker-count sweep mechanics, chosen-order discipline, and reproducible-evidence requirements — [Determinism](../reference/determinism.md)
