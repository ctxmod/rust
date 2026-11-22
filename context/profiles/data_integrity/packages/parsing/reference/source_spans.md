# Source Spans and Permissive Extraction

Tools that report on, or rewrite, human-authored text (config files, manifests, frontmatter, documents) must point at the exact original bytes and leave everything they do not touch unchanged. Crate and std facts were verified against std 1.99.0, serde_yaml 0.9.34, serde_norway 0.9.42, yaml-rust2 0.13.0, saphyr 0.1.0, and serde_json 1.0.151 (2026-10-04).

## Spans Are Byte Offsets

- A span is a `Range<usize>` of byte offsets into the original `&str`. Never store char or grapheme counts: they cannot index the string and drift on multi-byte text.
- Compute line and column on demand, and only for display. State the column unit explicitly in the diagnostic contract: bytes, chars (Unicode scalar values), or UTF-16 code units for editor protocols. Never round-trip display coordinates back into offsets.
- Every span is relative to a declared origin. When a sub-document is parsed out of a larger one (a body after a frontmatter block, an embedded snippet), record the base offset with the span and rebase on report. Never re-slice silently so that offsets quietly become relative to a fragment.

## Char-Boundary Discipline

- Check offsets before slicing. `str::is_char_boundary` treats `0` and `len()` as boundaries and returns `false` past the end. `str::get(range)` is the non-panicking alternative: it returns `None` wherever the equivalent indexing would panic.
- Slicing `&s[a..b]` panics if either end is not a char boundary, if `a > b`, or if `b > len`. `String::replace_range` panics on a bounded end that is not a char boundary. Validate every externally supplied offset before use.
- `str::floor_char_boundary` and `ceil_char_boundary` are stable since Rust 1.91.0. Use them to snap a display-derived offset to a valid boundary deliberately, never to hide an offset bug.

## Span-Preserving Edits

- Apply edits by splicing replacement text into the original bytes. Do not re-serialize the parsed tree: serializers normalize quoting, ordering, comments, and whitespace.
- Collect edits as `(span, replacement)` pairs, reject overlaps, then apply them in descending start order so earlier offsets stay valid. Every region outside an edit stays byte-identical.
- Re-parse after editing to re-derive spans. Spans from before the edit are stale, so never patch them arithmetically and reuse them.
- Round-trip invariant: parse followed by print is the identity on unedited input. Byte-order marks and line endings (CRLF, LF, or mixed) are preserved exactly, never normalized. A span-producing reader may strip a BOM for parsing only if it records the BOM's length in the origin.

## Permissive Extraction

Strict mode rejects malformed input at the first problem. Permissive mode extracts everything it can and reports every problem. Diagnostics are data, not early-return errors: the extractor returns the value together with a `Vec<Diagnostic>`, and each diagnostic carries a span.

### Duplicate Keys

Report a duplicate key as a diagnostic carrying both spans (first occurrence and duplicate). Parsers disagree on duplicates, so never rely on a parser's default:

- serde_yaml and serde_norway (`Value`/`Mapping`) and yaml-rust2 reject duplicate mapping keys with an error. serde_yaml is deprecated and unmaintained, and serde_yml is a deprecated shim, so do not choose either for new work.
- saphyr's loader and serde_json's `Value`/`Map` keep the last duplicate silently. This was observed in their source, not their docs, at the versions above.

Detect duplicates explicitly, either through an event- or marker-level parser (saphyr's parser exposes markers with byte `index` and spans with an exclusive `end`) or with your own scan over keys.

### Unknown Fields

Unknown fields survive extraction. Capture them with a `#[serde(flatten)]` catch-all map, or keep the raw source span so they can be re-emitted by splicing. serde ignores unknown fields by default for self-describing formats, which drops them silently. Use `#[serde(deny_unknown_fields)]` only in strict mode; it cannot be combined with `flatten`, so strict and permissive modes need separate types or a post-pass.

### Missing, Invalid, and Null

Keep three states distinct, because collapsing them to `Option<T>` loses the author's intent:

| State | Meaning | Diagnostic span |
|---|---|---|
| Missing | Key absent | Enclosing mapping |
| Present-null | Key present with an explicit null | The null value |
| Invalid | Key present, value fails to convert | The offending value |

Model the states with a dedicated enum (or `Option<Result<..>>` plus a null variant). Each non-success state carries its span.

### Explicit Knowledge States

The state set is exhaustive and each state is distinct: unknown, explicitly-absent, and conflicting are separate typed values, never collapsed. A present key is never reported missing; an absent key is never reported invalid. Where the domain carries more causes, extend the three-state table above to the five-state framing — Missing / Null / Invalid / Unavailable / Present — with every state representable; Unavailable covers "the source that would answer exists but cannot be consulted right now", which collapsing into Missing turns into a false absence.

- **Omission never implies a default.** An omitted field is represented and surfaced as absent; it never silently takes a default value. No implicit status, equivalence, confidence, or language: a consumer that needs a default applies its own, visibly, on top of an explicit absent.
- **Application policies state their decisions explicitly.** The output says which policy ran and why, rather than presenting a universal "safe default" — two consumers with different policies must be able to read the same extraction and diverge on top of it.
- **Condition evaluation is three-valued** — true / false / unknown — with the reason attached to the unknown. A boolean cannot carry "the answer exists but could not be determined"; a tri-state with a reason preserves it for the caller instead of forcing a guess at extraction time.

## Sketch

This is a sketch, not a drop-in implementation:

```rust
enum Field<T> { Missing, Null(Range<usize>), Invalid(Range<usize>, String), Present(T, Range<usize>) }

fn apply(src: &str, mut edits: Vec<(Range<usize>, String)>) -> Result<String, EditError> {
    edits.sort_by(|a, b| b.0.start.cmp(&a.0.start));      // descending
    let mut out = src.to_owned();
    let mut floor = usize::MAX;
    for (r, text) in edits {
        if r.end > floor || src.get(r.clone()).is_none() {  // overlap or non-boundary
            return Err(EditError::BadSpan(r));
        }
        out.replace_range(r.clone(), &text);
        floor = r.start;
    }
    Ok(out)                                                // caller re-parses
}
```

## Verification

Run round-trip identity checks on fixtures that include multi-byte characters, a BOM, CRLF and mixed line endings, comments, and a frontmatter-offset body. Seed duplicate keys, unknown fields, and all three missing/null/invalid states, then assert the diagnostic set and its spans. Property-test edits by applying random non-overlapping spans and asserting that the bytes outside them are unchanged.

## Related

- Error enums for strict-mode failures and the diagnostic/error split: [Error Handling](../../../../core/packages/errors/reference/error_handling.md)
- Untrusted-input posture and typed outcomes for persisted artifacts: [Persistence Envelopes for Zero-Copy Archives](../../persistence/reference/persistence_envelopes.md)
