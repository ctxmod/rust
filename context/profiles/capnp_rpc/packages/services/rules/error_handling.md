# Error Handling

Cap'n Proto uses explicit `Result`-based error handling with `capnp::Error` and `ErrorKind` variants. Use `?` for propagation. Handle `Disconnected` and `Overloaded` explicitly in RPC contexts.

## Error Types

- `Failed`: General failures
- `Disconnected`: Connection drops
- `Overloaded`: Backpressure
- `Unimplemented`: Missing method
- `NotInSchema`: Enum/union values outside schema

## Message Reading

```rust
let x = struct_reader.get_list_field()?.get(0)?;  // Propagates decode failures
```

Getters return `Result<T, capnp::Error>`. Use `?` to propagate.

## RPC Server Methods

```rust
async fn method(&mut self, params: Params, mut results: Results) -> Result<(), capnp::Error> {
    let x = params.get()?.get_x();
    results.get().set_y(x);
    Ok(())
}
```

## Client-Side Pipelining

Use `promise.and_then()` with `pry!` macro for early returns:

```rust
let promise = client.method()?;
promise.and_then(|response| {
    pry!(response.get()?.get_value());
    Ok(())
})
```

## Distributed Systems

- Wrap RPC in `RpcSystem`, handle `Disconnected`/`Overloaded`
- Use `Promise::all()` for concurrent calls
- For schema evolution, use sentinel defaults or unions to detect absences explicitly
- Implement `auto_reconnect` for variable connectivity

## Reconnection Pattern

```rust
use capnp_rpc::auto_reconnect;

let (client, set_target) = auto_reconnect(|| async {
    let ws = tokio_tungstenite::connect_async("wss://server").await?;
    let network = VatNetwork::new(/* ... */);
    let rpc_system = RpcSystem::new(network, bootstrap);
    Ok(rpc_system.client)
});
```

## Related

[Reconnection Patterns](../reference/reconnection.md)
