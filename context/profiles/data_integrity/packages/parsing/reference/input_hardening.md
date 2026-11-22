# Input Hardening

Externally-sourced bytes — files, network payloads, embedded archives — are adversarial by default. Parser security failures share one shape: the limit that would have prevented the failure was enforced after the allocation that exhausted the budget, or not enforced at all. This reference carries the evidence, the checklist, and the type-level disciplines; the rule mandates them.

## Why Limits-Before-Growth

The advisory record is the argument; each of these failed below an application-layer cap:

- **serde-json-wasm RUSTSEC-2024-0012** — no depth check; nesting depth is unbounded and stack exhaustion is attacker-controlled. The fix is a depth counter inside the parser.
- **quick-xml RUSTSEC-2026-0195** — `NsReader` appended a namespace binding (32 bytes) for every declaration on every start tag with no cap, so a single crafted start tag forced large heap allocations while the application's byte counter saw only the input size. Amplification lived entirely inside the parser.
- **russh RUSTSEC-2026-0154** — attacker-sized frames were buffered before validation. The length field was believed, then the bytes were read.
- **postgres-protocol RUSTSEC-2026-0179** — the client ran a server-supplied SCRAM iteration count with no upper bound, pinning a worker thread for minutes per connection; an untrusted scalar drove unbounded work because no bound existed at the consumption site.

serde_json's `recursion_limit` design shows the intended shape — a default depth bound inside the parser — and its feature-gated `unbounded_depth` opt-out shows the cost of removing it is explicit and named. Application-layer byte caps cannot see in-parser amplification, which is exactly why limits live at the allocation site.

## The Pre-Growth Limit Checklist

Every parser entry point declares, and enforces before growth:

- **Depth** — nesting, recursion, container depth. Enforced with a counter at descent, not by stack-overflow detection.
- **Token and decoded size** — the largest single scalar, string, or decoded expansion the parser will materialize.
- **Element counts** — array lengths, map sizes, comment depth: each counted against a declared maximum before the container allocates.
- **Typed resource diagnostics on breach** — a limit breach is a typed error variant carrying the limit and the observed value; never an abort, never a silent truncation. The error-handling reference's taxonomy owns the variant shape.
- **Budget profiles** — bounds beyond a single parse (process memory budget, RSS ceiling, spill-to-disk growth path) are explicit, named parameters with defaults, not ambient globals. Owned-buffer growth, mapped-memory size, and RSS are accounted separately — claiming one when you measured another is a false budget.
- **Spill-to-disk** is the bounded growth path for streams that legitimately exceed memory: the bound moves from RAM to disk, it does not disappear.

## Parse, Don't Validate

Constructor success must carry the proof, not assume it:

- Fallible smart constructors (`Decimal::from_scientific`, `Depth::new(n)`) return `Result` into a refined type; every downstream use is statically guaranteed valid. Validation that returns `()` or a bool loses the proof at the next call boundary.
- The enforcement ladder for domains the type system cannot fully capture: static types where possible → dynamic enforcement at the boundary → `debug_assert!` for internal invariants → `_unchecked` accessors as opt-in, documented, and confined to callers with a proven-valid argument. `_unchecked` is never the default spelling a newcomer would reach for.
- `unwrap` on untrusted bytes is a denial-of-service primitive: the input selects whether the process dies. Panics never respond to externally-sourced bytes; `expect` is permitted only where the argument is a type-level-proven invariant, and the message names the invariant.
- The honest counterpoint: a panic for a genuine internal bug (an invariant violation the types already rule out) is correct — the carve-out is proven invariants, not convenience. Bytes that came from outside never qualify.

## Chunk-Boundary Invariance

Streaming parsers must not let the transport's chunking become semantics:

- nom's `Incomplete` contract states the shape: whether the data arrives entirely or in chunks, the result should be the same. A parser whose output differs by chunk split has let the reader's buffer size into the document model.
- The failure mode is real and observable: values split across boundaries, events duplicated or dropped at resumes, spans shifted by bytes consumed twice or skipped.
- The property shape: `parse(whole) == events(parse(prefix) + parse(suffix))` for every split point — values, events, spans, and diagnostics all identical. Prove it as a property over arbitrary split points (the property-testing reference owns the mechanics); never assume it from a fixed-chunk test.
- Span-shift metamorphic relations pair with it: prepending `d` bytes shifts every span by exactly `d`; splitting at `k` leaves every post-split span shifted by a computable constant.

## Unsafe Isolation and Validated Views

Raw bytes become typed views only through a validated boundary. The full discipline — `unsafe` confined to declared validated-view modules, SAFETY comments per block, the alignment/width/endianness/discriminant/offset/UTF-8 checklist before typed access — lives in the persistence-envelopes reference's Unsafe Isolation section, which owns this subject; cross-reference it rather than restating it. `clippy::undocumented_unsafe_blocks` (in the selective-pedantic warn set) is the enforcement hook, and `clippy::arithmetic_side_effects` may be scoped to parsing modules where overflow is an attack surface.

## Fuzzing and Adversarial Fixtures

- Every parser entry point gets a structure-aware fuzz target: `arbitrary`-derived input types that generate syntactically plausible documents, not raw byte soup. The fuzz crate, nightly pinning, corpus minimization, and crash-to-regression promotion are the property-testing reference's to own mechanically.
- The merged corpus is a permanent regression suite: minimized crashing inputs are committed as fixtures and replayed deterministically on the stable toolchain.
- The blanket property every parser states: arbitrary bytes never panic and always produce either a typed result or a typed diagnostic, within the declared limits.
- Committed adversarial fixtures are edge-biased toward danger — boundaries, limits, invalid UTF-8 — and live under the fixture-verification reference's corpus model, where they double as negative controls with declared rejections.

## Related

- The corruption posture and unsafe isolation for binary views: [Persistence Envelopes for Zero-Copy Archives](../../persistence/reference/persistence_envelopes.md)
- Typed error variants for limit breaches and parse failures: [Error Handling](../../../../core/packages/errors/reference/error_handling.md)
- Fuzz-target mechanics and chunk-split property execution: [Property, Metamorphic, and Fuzz Verification](../../../../verification/packages/property/reference/property_testing.md)
