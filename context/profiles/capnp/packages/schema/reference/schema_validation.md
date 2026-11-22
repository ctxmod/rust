# Schema Validation

Validate Cap'n Proto schemas against schema evolution rules, type validity, and file-ID uniqueness — and, when the schemas are generated from a higher-level operation contract, against that contract — to ensure consistency and compatibility.

## Validation Categories

### 1. Contract Compliance (generated schemas only)

When `.capnp` interfaces are generated from operation specifications (a service catalog declaring each operation's `rpc.service`, `rpc.method`, and interaction `pattern`), verify the generated interfaces match them:

- Service names match `rpc.service` field
- Method names match `rpc.method` field
- Streaming modes match operation `pattern`
- Message types contain all operation parameters/outputs

**Tool**: a validator beside the generator, for example:

```go
func validateContract(op Operation, schema Schema) error {
    if schema.Service != op.RPC.Service {
        return fmt.Errorf("service name mismatch: %s != %s", schema.Service, op.RPC.Service)
    }

    if schema.Method != op.RPC.Method {
        return fmt.Errorf("method name mismatch: %s != %s", schema.Method, op.RPC.Method)
    }

    expectedStreaming := patternToStreaming(op.Pattern)
    if schema.Streaming != expectedStreaming {
        return fmt.Errorf("streaming mode mismatch: %s != %s", schema.Streaming, expectedStreaming)
    }

    return validateMessageTypes(op, schema)
}
```

### 2. Evolution Compatibility

Check schema changes follow backward/forward compatibility rules:

- Field ordinals are consecutive
- No field number reuse
- New fields added at end
- No type changes to existing fields
- Explicit IDs for moved types

**Tool**: CI runs on schema PRs. The `capnp` compiler validates one version of a schema at a time and has no built-in old-versus-new comparison, so compatibility against the base branch needs a checker that compares the two parsed schemas (ordinals, types, and IDs per struct and interface):

```bash
# Every schema must still compile on its own
capnp compile -o- new.capnp > /dev/null
# Compare against the base branch with the repository's evolution checker
git show origin/main:schemas/user.capnp > old.capnp
evolution-check old.capnp new.capnp
```

### 3. Type Validity

Ensure all types resolve to valid Cap'n Proto primitives or defined structs:

- Primitive types are valid (`Text`, `Data`, `Int64`, etc.)
- List element types are valid
- Struct references exist in schema
- No circular references

### 4. Streaming Mode Consistency (generated schemas only)

Verify streaming modes align with operation patterns:

| Pattern | Allowed Streaming |
|---------|------------------|
| `get` | `unary` |
| `list` | `unary` |
| `stream` | `server_stream` |
| `watch` | `server_stream` |
| `bidi` | `bidi` |

### 5. File ID Uniqueness

Ensure all `.capnp` files have unique 64-bit IDs:

```bash
find schemas -name "*.capnp" -exec grep -h "^@0x" {} + | sort | uniq -d
```

No duplicates should appear.

## CI Integration

```yaml
# .github/workflows/validate-capnp.yaml
name: Validate Cap'n Proto Schemas

on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - name: Install capnp
        run: sudo apt-get install -y capnproto

      - name: Validate schemas
        run: |
          find schemas -name '*.capnp' -print0 |
            xargs -0 -n1 sh -c 'capnp compile -I schemas -o- "$0" > /dev/null'

      - name: Validate contract compliance
        run: ./scripts/validate-contracts   # only when schemas are generated

      - name: Check evolution compatibility
        run: ./scripts/evolution-check origin/main HEAD
```

## Error Categories

| Category | Severity | Fix |
|----------|----------|-----|
| Contract mismatch | Error | Update generator or operation definition |
| Evolution violation | Error | Follow safe change rules |
| Invalid type | Error | Use valid Cap'n Proto type |
| Streaming inconsistency | Error | Align pattern with streaming mode |
| Duplicate file ID | Error | Regenerate with `capnp id` |

## Related

[Schema Evolution](../rules/schema_evolution.md)
