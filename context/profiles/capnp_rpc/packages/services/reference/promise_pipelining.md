# Promise Pipelining

Promise pipelining allows chaining RPC calls in a single network round trip by sending dependent requests upfront, collapsing multi-step operations. Client receives promise immediately on RPC call and can chain subsequent calls without waiting.

## Mechanics

**Traditional approach** (multiple RTTs):
```
Client -> Server: foo()
Client <- Server: result1
Client -> Server: bar(result1)
Client <- Server: result2
Total: 2 RTTs
```

**Pipelined approach** (single RTT):
```
Client -> Server: foo()
Client -> Server: bar(foo_result)
Client -> Server: baz(bar_result)
Client <- Server: final_result
Total: 1 RTT
```

## Example

```rust
// Traditional (sequential)
let result1 = client.foo().await?;
let result2 = client.bar(result1).await?;
let final = client.baz(result2).await?;

// Pipelined (single RTT)
let promise = client.foo()?;
let promise2 = promise.bar()?;
let final = promise2.baz().await?;  // Only one await at the end
```

## Latency Benefits

- Collapses multi-step operations
- 200-500ms savings per chain on high-latency mobile/edge
- 4x+ faster than gRPC over high-latency links
- Prevents N+1 query problems

## Schema Design Impact

Interfaces define capabilities passed by reference:

```capnp
interface Factory {
    create @0 () -> (obj :Object);
}

interface Object {
    method @0 () -> (result :Text);
}
```

Client chains:
```rust
let obj = factory.create()?;
let result = obj.method().await?;  // Single RTT
```

## Capability-Based Security

Object references serve as capabilities:
- Grant call permissions
- New objects accessible only by creator
- Passing reference delegates access securely
- IDs are connection-specific (prevent forgery)

## Related

[WebSocket Tunneling](../rules/websocket_tunneling.md)
