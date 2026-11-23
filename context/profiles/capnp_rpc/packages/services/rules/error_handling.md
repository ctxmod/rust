# Error Handling

Cap'n Proto uses explicit `Result`-based error handling with `capnp::Error`, whose public `kind: ErrorKind` drives handling. Use `?` for propagation. Handle `Disconnected` and `Overloaded` explicitly in RPC contexts.

## Error Kinds

The four RPC-level kinds mirror the Cap'n Proto exception types; construct them with `capnp::Error::failed(..)`, `overloaded(..)`, `disconnected(..)`, and `unimplemented(..)`:

- `Failed`: General failures — not retryable without a change
- `Overloaded`: Temporary lack of resources — retry later with backoff, never immediately
- `Disconnected`: The connection was lost — reconnect, then retry idempotent calls
- `Unimplemented`: The method or capability is not implemented by the peer

`ErrorKind` also carries many decode-specific variants for malformed messages; an enum or union value outside the reader's schema surfaces as `NotInSchema` directly from a generated enum getter or union `which()`, and as `EnumValueOrUnionDiscriminantNotPresent(NotInSchema)` once converted into `capnp::Error`. `ErrorKind` is `#[non_exhaustive]`: match the four RPC kinds and treat everything else as `Failed`. Since capnp 0.28 the message text is read through `.extra()` and errors can carry typed `details`.

## Message Reading

```rust
let x = struct_reader.get_list_field()?.get(0)?;  // Propagates decode failures
```

Getters return `Result<T, capnp::Error>`. Use `?` to propagate.

## RPC Server Methods

Generated `Server` traits (capnpc 0.23+) take `self: Rc<Self>` and are `async fn`:

```rust
impl foo::Server for FooImpl {
    async fn method(
        self: Rc<Self>,
        params: foo::MethodParams,
        mut results: foo::MethodResults,
    ) -> Result<(), capnp::Error> {
        let x = params.get()?.get_x();
        results.get().set_y(x);
        Ok(())
    }
}
```

State that a method mutates lives behind `Cell`/`RefCell`; never hold a `RefCell` borrow across an `.await`, because another call on the same object can run in between.

## Client Calls and Pipelining

A call is a request builder: `client.method_request()`, fill `request.get()`, then `send()`. The result has a `promise` to await and a `pipeline` for calling methods on capabilities the response will contain before it arrives:

```rust
let response = client.method_request().send().promise.await?;
let value = response.get()?.get_value();

// Pipelined: call a method on a returned capability in the same round trip
let obj = factory.create_request().send().pipeline.get_obj();
let text = obj.method_request().send().promise.await?;
```

`pry!` is for the legacy `Promise`-returning style only, where `?` is unavailable; in `async` code use `?`.

## Distributed Systems

- Wrap RPC in `RpcSystem`, handle `Disconnected`/`Overloaded`
- The RPC system is single-threaded (`Rc`-based, `!Send`): run it and every client under a `tokio::task::LocalSet` with `spawn_local`, never `tokio::spawn`
- Issue concurrent calls by sending several requests before awaiting, and join the promises (`futures::future::try_join_all`)
- For schema evolution, use sentinel defaults or unions to detect absences explicitly
- Implement `auto_reconnect` for variable connectivity

## Reconnection Pattern

`auto_reconnect` takes a synchronous `FnMut() -> capnp::Result<C>`; wrap the asynchronous connect in `capnp_rpc::new_future_client` so calls queue until the new connection is up:

```rust
use capnp_rpc::{auto_reconnect, new_future_client};

let (client, set_target) = auto_reconnect(|| Ok(new_future_client(connect())))?;
```

The full pattern, with `connect()` and heartbeat detection, is in the reconnection reference.

## Related

[Reconnection Patterns](../reference/reconnection.md)
