# Input Hardening

Code that consumes externally-sourced bytes — files, network, embedded payloads — treats every input as adversarial. Parser security failures come from limits bolted on after growth, not before.

- Resource limits (nesting depth, token size, comment depth, decoded-string size) are enforced inside the parser at the allocation site, before any buffer grows; application-layer byte caps cannot see in-parser amplification.
- Constructor success is never structural validation: fallible smart constructors build typed domains that carry the proof; `_unchecked` siblings are opt-in, documented, and confined to proven-valid callers.
- Panics never respond to externally-sourced bytes; every parse or decode failure is a typed `Result` error. `expect` is permitted only where the argument is a type-level-proven invariant, named in the message.
- Chunk-boundary invariance: identical values, events, spans, and diagnostics for every split of the same input — proven as a property over arbitrary split points, never assumed.
- Bounds beyond the parser (process budgets, spill-to-disk growth paths) follow the budget-profile discipline; exceeding a bound is a typed resource diagnostic, never unbounded growth.
- Adversarial coverage: fuzz every parser entry point with structure-aware targets and a persisted corpus; committed fixtures are edge-biased toward danger — boundaries, limits, invalid UTF-8.
- the RUSTSEC evidence, the pre-growth limit checklist, parse-don't-validate, chunk-boundary invariance, unsafe confinement, and fuzz-target design — [Input Hardening](../reference/input_hardening.md)
- how the fuzzing and chunk-boundary properties are executed — [Property, Metamorphic, and Fuzz Verification](../../../../verification/packages/property/reference/property_testing.md)
