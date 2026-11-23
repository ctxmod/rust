# Data Integrity

Rust code that persists derived state, parses authored text, resolves paths under a root, or maintains an incremental index follows four disciplines. Load archives through checked access behind a versioned envelope header, and publish them atomically. Spans are UTF-8 byte offsets; edits splice the original bytes; duplicate keys and unknown fields surface as diagnostics. Containment compares canonical paths component-wise, and the symlink policy is declared per file class. A partial scan never means deletion, and a coverage error never means empty.

- checked access, alignment, buffer lifetimes, envelope stamps, corruption posture, and atomic generation publication for zero-copy archives — [Persistence Envelopes for Zero-Copy Archives](../reference/persistence_envelopes.md)
- `unsafe` exists only inside declared validated-view modules; every raw-byte-to-reference boundary validates alignment, width, endianness, enum validity, offsets, and UTF-8 before typed access, with a SAFETY comment per block — [Persistence Envelopes for Zero-Copy Archives](../reference/persistence_envelopes.md)
- byte-offset spans, char-boundary discipline, span-preserving edits, and permissive extraction (duplicate keys, unknown fields, missing vs invalid) — [Source Spans and Permissive Extraction](../../parsing/reference/source_spans.md)
- canonical roots, component-wise containment, no-follow inspection, TOCTOU-aware writes, and Linux/macOS path aliases and case sensitivity — [Path Containment and Symlink Safety](../../parsing/reference/path_containment.md)
- lazy layers, stable input fingerprints, partial-scan and coverage semantics, and `benches/` measurement discipline — [Index Freshness and Benchmarks](../../indexing/reference/index_freshness_and_benchmarks.md)
