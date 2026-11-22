# Fixture-Driven Verification

Conformance and behavior verification is fixture-driven: committed fixture data behind a black-box harness, independent oracles, stable case-ID closure with re-runnable censuses, a blessability taxonomy with never-blessable safety classes, adversarial negative controls, and vacuity defenses that keep verification honest. Conformance suites for parsers, compilers, emitters, and indexers are committed data plus a harness, not code inside a product crate. This reference defines the corpus model, the oracles that judge it, and the honesty rules that keep a green run meaningful. External-suite precedents referenced for shape, not as mandates: JSONTestSuite's `y_`/`n_`/`i_` classification, KDL's `_fail` files and NOT-AUTHORITIVE markers, toml-test's `invalid/` tree, and the WebAssembly spec's `assert_invalid` directives all converge on the same mechanics.

## The Fixture Corpus Model

One schema governs every fixture and is validated at load — a fixture that does not validate against its schema is a harness error, not a skipped case. The tree shape:

```
tests/fixtures/
├── schema/            # JSON Schema draft-07 pattern; every fixture validates against it
├── <area>/            # per-area scenario files; scenario-ID prefix equals the directory
│   ├── data/          # byte-exact inputs the scenarios consume
│   └── invalid/       # negative controls, each declaring its rejection
```

- Scenario files are YAML (or JSON); `data/` holds the raw bytes a scenario feeds the implementation. The scenario-ID prefix equals its directory so a misfiled fixture is visible in a listing.
- Fixtures are committed data consumed by the harness and by any implementation-coupled tests. They are data, not test code: no `#[cfg(test)]` module, no crate `tests/*.rs` target lives in the fixture tree — the `tests/` directory name collides with cargo's integration-test convention only for data files. A repository whose test policy scans `tests/` paths should exempt fixture data (YAML, bytes, JSON) by path, not by suppression markers.
- Two-lane location when a design corpus exists: the design-repo copy is authoritative for the design; the implementation tree carries a synchronized copy. Divergence between lanes is a sync bug, detected by comparing case-ID sets.

## Conformance Identity and Closure

- Every scenario cites stable conformance case IDs (`<FAMILY>-<NN>` style) traced to spec sections. Case IDs are minted once and never renumbered or reused — the same stability rule as diagnostic machine codes.
- Closure is bidirectional and mechanical: every case ID cited by the spec-derived list has at least one scenario, and every scenario cites at least one case ID. Publish the expected counts; the census is re-run arithmetic ("re-run these checks and diff the numbers"), never judgment.
- One behavior per scenario: a scenario asserting two independent behaviors fails bisectably only half the time. Where a spec section carries several behaviors, mint several scenarios.
- Every fixture is classified three ways — must-accept, must-reject, implementation-defined (the JSONTestSuite `y_`/`n_`/`i_` shape). Implementation-defined cases assert stability (same input, same output across runs), not a specific verdict.

## Independent Oracles

An expected value is evidence only if it came from somewhere other than the code under test:

- **Spec text.** Expected values derived by hand from the specification, recorded in the scenario with the section reference. This is the default oracle.
- **Differential / N-version.** A second implementation, independently derived from the spec, produces the expected output. Two implementations sharing a spec misreading fail together — correlated failure is the known caveat, so differential agreement supplements spec-derived values; it never replaces them.
- **Specified relations.** Round-trip (`print(parse(x)) == x`), canonical-print stability, chunk-split invariance. A relation needs no ground truth, only a stated invariant between runs — the metamorphic shape in the property-testing reference.

The input-never-snapshot rule: a data file used as input anywhere never serves as a snapshot source anywhere. A suite that blesses its own inputs' outputs has encoded the implementation's current behavior as truth — the trap is encoded as a negative control: an input file whose bytes equal a snapshot file's bytes must be rejected by the corpus lint.

## Blessability Taxonomy

Each check has exactly one oracle class, declared in the scenario: exit status, stream/disk snapshot, predicate, structured output, absence, no-change, containment.

- **Never blessable** — exit codes, absence, no-change, and containment results. These are safety boundaries: a run that regenerates them instead of asserting them can convert a containment failure into a green pass, so no bless path exists for them, at all.
- **Blessable with review** — snapshot-class oracles only. A bless run rewrites only declared blessable, single-oracle data files; never fixture YAML, never inputs; the rewrites are listed for review in the same output as the verdicts. Re-blessing is a decision someone signs, not a flag habit.
- **The bless fence.** The harness refuses to bless under overwrite-mode environments (`TRYCMD`, `SNAPSHOTS`-style variables can fake passes by rewriting expectations in place). Blessing runs inside the harness's own protocol or not at all.
- **Reconciliation with golden-output doctrine.** Byte-pinned goldens for consumed CLI output remain blessable-with-review (the tooling-contract doctrine of the engineering module's repositories profile). The "never serialized byte goldens" ban in the persistence-envelopes reference governs archive bytes, whose layout legitimately changes with crate versions and format features — that ban is untouched; this reference governs how snapshot goldens may be produced, blessed, and fenced, not which bytes may be pinned.

## Negative Controls and Adversarial Coverage

- Every `invalid/` fixture declares its rejection in machine-readable form (`expect-invalid: <reason>`). The harness proves every file rejected as declared — "rejected for the declared reason" where the error carries a code, "rejected" where it does not. A file that passes when it should fail is a suite failure, never a warning.
- Adversarial fixtures are edge-biased toward danger: multi-byte character boundaries, nesting near every declared limit, a leading BOM, CRLF and mixed line endings, duplicate keys, invalid UTF-8, zero-length inputs, maximum-length tokens. The generator-design guidance in the property-testing reference applies to fixture authoring too.
- External conformance suites are pinned by version — commit hash plus content hash — and imported fixtures keep their upstream case IDs alongside local ones so upstream updates diff cleanly.
- A conformance suite is a living interpretation of the spec: upstream errata and issue-tracker verdicts (KDL's NOT-AUTHORITIVE posture) feed back into case classifications, and the errata loop is part of the suite's maintenance contract, not an afterthought.

## Vacuity Defenses and Honesty

- A filter selecting zero scenarios is a failure, not a pass. Every run reports registered versus selected counts; a no-match control (a filter that cannot match anything, expected to error) proves the run itself is non-vacuous.
- Detection meta-fixtures: scenarios with inverted expectations (`verdict: fail`) that seed known failures into a stub binary. The harness is proven to detect them — green against the meta-fixtures before any product-feature work proceeds. Harness-before-product sequencing: the harness is a deliverable with its own acceptance, and it lands first.
- Under an effective-uid-0 harness, permission-dependent assertions record `indeterminate` — evidence without a verdict — with the marks, captures, and uid retained. A silent pass is forbidden: CI containers and agent sandboxes routinely run as root, where permission-bit rejections cannot occur.
- Oracles are instrumented: byte counters, file counters, record counters — measurable claims, not wall-clock impressions. Partial or cancelled runs label their results incomplete; a run that died mid-suite asserts nothing about the scenarios it never reached.
- Semantic-equivalence and byte-reproducibility are separate claims and are reported separately. The general semantic-vs-invariant discipline (which claims are semantic, which are byte invariants, and how each is evidenced) lives in the agentic module's verification doctrine — do not restate it here; this reference only requires the two claims never be merged into one verdict line.

## Harness Discipline

- The harness consults the product only through its binary and declared seams — no implicit coupling to crate internals, no test-only builds, no `cfg(test)` behavior switches, no test modes in the product. The binary is built by the normal build path and injected explicitly.
- Seams carry production defaults: the restrictive default applies when a seam is absent, never a permissive one.
- The harness owns its environment: the environment variable names it sets for the product are its own namespace, and no fixture may set them — a fixture that carries harness-owned variables is a corpus-lint error.

## Placement Among Test Surfaces

- The harness is a black-box test program: it runs the built binary as a subprocess and is selected deliberately by whoever needs its evidence. Fence its scratch environment (temporary directories, environment variables, tool resolution) so a run cannot touch the developer's real state.
- Implementation-coupled properties during active work (chunk-split invariance over generated inputs, span-shift metamorphic relations) belong in implementation-coupled tests, wherever the repository's test policy places those.
- Negative-control doctrine in general — what makes a control honest, why positive-only evidence fails — lives in the agentic module's verification doctrine; this reference instantiates it for conformance fixtures.

## Related

- Runner mechanics and isolation principles for the tests around the harness: [Test Runners](../../property/reference/test_runners.md)
- Property and fuzz mechanics for the specified-relation oracles: [Property, Metamorphic, and Fuzz Verification](../../property/reference/property_testing.md)
- The parser-side limits the adversarial fixtures attack: Input Hardening (in the `data_integrity` profile)
