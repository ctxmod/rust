---
paths:
  - "*.capnp"
  - "*/*.capnp"
  - "*/*/*.capnp"
  - "*/*/*/*.capnp"
  - "*/*/*/*/*.capnp"
  - "*/*/*/*/*/*.capnp"
---

# Schema Evolution

Cap'n Proto schemas evolve through ordinal field numbering, ensuring backward and forward compatibility. Never reuse or skip field numbers. Add new fields at the end with higher ordinals.

## Rationale

Cap'n Proto's zero-copy design relies on stable field layouts. Ordinal numbers define field positions in the serialized format. Renumbering breaks binary compatibility—old readers misinterpret new data and vice versa. Adding fields at the end preserves existing layouts, enabling seamless version mixing.

## Safe Changes

- Adding new fields/methods at end (higher numbers)
- Renaming symbols (names are not part of wire format)
- Moving types with explicit IDs (`@0x...`)
- Rearranging source code (keeping numbers fixed)

## Unsafe Changes

- Renumbering fields
- Altering types or defaults
- Removing fields (mark obsolete instead)
- Reusing field numbers

## Examples

**Correct** -- adding a field:
```capnp
@0xa1b2c3d4e5f67890;

struct User {
    id @0 :UInt64;
    name @1 :Text;
    email @2 :Text;        # New field added at end
}
```

**Incorrect** -- inserting between existing fields:
```capnp
struct User {
    id @0 :UInt64;
    active @1 :Bool;       # ERROR: inserted before @1
    name @1 :Text;         # ERROR: reused @1
}
```

**Correct** -- marking obsolete:
```capnp
struct User {
    id @0 :UInt64;
    deprecated @1 :Text;   # Mark obsolete, don't remove
    name @2 :Text;
}
```

## Tooling

Generate unique file IDs with `capnp id`. Test evolution with mixed-version readers/writers.
