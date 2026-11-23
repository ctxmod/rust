# Embedding Service Example

This example demonstrates a complete Cap'n Proto RPC service for embedding vectors with metadata, including WebSocket tunneling for browser clients.

The code is illustrative, not a buildable crate: it shows the shape of the schema, the server, and the WebSocket transport, with store and search logic elided. Paths follow a polyglot monorepo layout (`pkg/capnp/` for schemas, `pkg/rs/` for Rust). Two schema constructs need adjusting before the schema compiles with current `capnp`: Cap'n Proto has no `Float16` primitive (carry half-precision values as `List(UInt16)` bit patterns), and its `-> stream` methods are client-to-server flow control that return no results — model server-to-client streaming as a callback capability the client passes in (for example `search @1 (query :Embedding, k :UInt32, sink :ResultSink) -> ()` with `interface ResultSink { push @0 (results :List(Embedding)) -> stream; done @1 () -> (); }`).

## Schema Definition

`pkg/capnp/embedding.capnp`:
```capnp
@0xa1b2c3d4e5f67890;

struct Embedding {
    content @0 :Data;           # Raw embedded content (bytes)
    contentHash @1 :Data;       # SHA256 hash (32 bytes)
    attributes @2 :Text;        # JSON/attributes string
    dims @3 :UInt32;            # Vector dimension

    union {
        float32 @4 :List(Float32);
        float16 @5 :List(Float16);
        int8 @6 :List(Int8);
    }

    model @7 :Text;
    modelVersion @8 :UInt32;
    date @9 :UInt64;            # Unix timestamp
}

interface EmbeddingService {
    # Unary: store embedding
    store @0 (embedding :Embedding) -> (id :Text, storedAt :UInt64);

    # Server streaming: search similar embeddings
    search @1 (query :Embedding, k :UInt32)
        -> stream (results :List(Embedding));

    # Server streaming: event stream
    streamEvents @2 (sessionId :Text, lastSeq :UInt64)
        -> stream (event :Event);
}

struct Event {
    seq @0 :UInt64;
    embedding @1 :Embedding;
    timestamp @2 :UInt64;
}
```

## Generated Rust Types

After running `cargo build`, generated types appear in `pkg/rs/gen/embedding_capnp.rs`.

## Server Implementation

`pkg/rs/src/embedding_service.rs` — generated `Server` traits (capnp 0.28) are `async fn` taking `self: Rc<Self>`, so mutable state sits behind `RefCell`, and `search` streams batches into the client-supplied `ResultSink` capability (the schema adjustment noted above):
```rust
use std::cell::RefCell;
use std::rc::Rc;

use crate::gen::embedding_capnp::embedding_service;

pub struct EmbeddingServiceImpl {
    store: RefCell<EmbeddingStore>,
}

impl embedding_service::Server for EmbeddingServiceImpl {
    async fn store(
        self: Rc<Self>,
        params: embedding_service::StoreParams,
        mut results: embedding_service::StoreResults,
    ) -> Result<(), capnp::Error> {
        let embedding = params.get()?.get_embedding()?;
        let id = self.store.borrow_mut().add(embedding)?;

        let mut r = results.get();
        r.set_id(&id);
        r.set_stored_at(current_timestamp());
        Ok(())
    }

    async fn search(
        self: Rc<Self>,
        params: embedding_service::SearchParams,
        _results: embedding_service::SearchResults,
    ) -> Result<(), capnp::Error> {
        let p = params.get()?;
        let (query, k, sink) = (p.get_query()?, p.get_k(), p.get_sink()?);

        // Collect under the borrow, then release it: no RefCell borrow across .await.
        let batches = self.store.borrow().search(query, k);
        for batch in batches {
            let mut push = sink.push_request();
            fill_results(push.get().init_results(batch.len() as u32), &batch);
            push.send().await?; // `-> stream` call: flow-controlled by the RPC system
        }
        sink.done_request().send().promise.await?;
        Ok(())
    }
}
```

## WebSocket Server

`pkg/rs/src/ws_server.rs` — Cap'n Proto RPC is `Rc`-based and `!Send`, so the accept loop runs inside a `tokio::task::LocalSet` and each connection's `RpcSystem` is driven with `spawn_local`:
```rust
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::gen::embedding_capnp::embedding_service;

async fn serve(listener: TcpListener) -> anyhow::Result<()> {
    let service: embedding_service::Client =
        capnp_rpc::new_client(EmbeddingServiceImpl::new());

    loop {
        let (stream, _) = listener.accept().await?;
        let ws_stream = accept_async(stream).await?;
        let (write_half, read_half) = ws_stream.split();

        let network = VatNetwork::new(
            WSReader::new(read_half),
            WSWriter::new(write_half),
            Side::Server,
            Default::default(),
        );
        let rpc_system = RpcSystem::new(Box::new(network), Some(service.clone().client));
        tokio::task::spawn_local(rpc_system);
    }
}

// main: tokio::task::LocalSet::new().run_until(serve(listener)).await
```

## WebSocket Client (Rust)

`pkg/rs/src/ws_client.rs` — `auto_reconnect` takes a synchronous closure; `new_future_client` lets it hand back a client while the asynchronous connect is still in flight:
```rust
use capnp_rpc::{auto_reconnect, new_future_client, rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};

use crate::gen::embedding_capnp::embedding_service;

async fn connect(url: String) -> capnp::Result<embedding_service::Client> {
    let (ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| capnp::Error::disconnected(e.to_string()))?;
    let (write, read) = ws.split();

    let network = VatNetwork::new(WSReader::new(read), WSWriter::new(write), Side::Client, Default::default());
    let mut rpc_system = RpcSystem::new(Box::new(network), None);
    let client: embedding_service::Client = rpc_system.bootstrap(Side::Server);

    tokio::task::spawn_local(rpc_system); // on the caller's LocalSet
    Ok(client)
}

fn create_client(url: &str) -> capnp::Result<embedding_service::Client> {
    let url = url.to_owned();
    let (client, _set_target) = auto_reconnect(move || Ok(new_future_client(connect(url.clone()))))?;
    Ok(client)
}

async fn use_client(client: &embedding_service::Client) -> anyhow::Result<()> {
    let mut request = client.store_request();
    request.get().set_embedding(/* ... */)?;

    let response = request.send().promise.await?;
    let id = response.get()?.get_id()?.to_str()?;

    println!("Stored embedding with id: {id}");
    Ok(())
}
```

## Performance

- Zero-copy: Embeddings map directly from buffer
- SIMD-friendly: Fixed-size arrays for constant dimensions
- Pipelining: calls on capabilities returned by an earlier call ride the same round trip (data results such as `id` must still be awaited)
- Bandwidth: Packed encoding saves 10-30%

## Related

- Zero-copy serialization: the `capnp` profile, `schema` package, `reference/zero_copy.md`
- WebSocket tunneling: the `capnp_rpc` profile, `services` package, `rules/websocket_tunneling.md`
- Reconnection: the `capnp_rpc` profile, `services` package, `reference/reconnection.md`
- Server trait and client API shapes follow the capnp-rpc 0.28 `hello-world` and `calculator` examples in the capnproto-rust repository
