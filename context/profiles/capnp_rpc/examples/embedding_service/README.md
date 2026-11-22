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

`pkg/rs/src/embedding_service.rs`:
```rust
use capnp::capability::Promise;
use crate::gen::embedding_capnp::{embedding_service, Embedding, Event};

pub struct EmbeddingServiceImpl {
    store: EmbeddingStore,
}

impl embedding_service::Server for EmbeddingServiceImpl {
    async fn store(
        &mut self,
        params: embedding_service::StoreParams,
        mut results: embedding_service::StoreResults,
    ) -> Result<(), capnp::Error> {
        let embedding = params.get()?.get_embedding()?;
        let id = self.store.add(embedding)?;

        results.get().set_id(&id);
        results.get().set_stored_at(current_timestamp());
        Ok(())
    }

    async fn search(
        &mut self,
        params: embedding_service::SearchParams,
        mut results: embedding_service::SearchResults,
    ) -> Promise<(), capnp::Error> {
        let query = params.get()?.get_query()?;
        let k = params.get()?.get_k();

        Promise::from_future(async move {
            let results_stream = self.store.search(query, k).await;

            while let Some(batch) = results_stream.next().await {
                let mut results_builder = results.get();
                let mut results_list = results_builder.reborrow().init_results(batch.len() as u32);

                for (i, embedding) in batch.iter().enumerate() {
                    // Fill results list
                }
            }

            Ok(())
        })
    }
}
```

## WebSocket Server

`pkg/rs/src/ws_server.rs`:
```rust
use capnp_rpc::{RpcSystem, twoparty::VatNetwork, rpc_twoparty_capnp::Side};
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn handle_ws_connection(stream: TcpStream) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (read_half, write_half) = ws_stream.split();

    let network = VatNetwork::new(
        WSReader::new(read_half),
        WSWriter::new(write_half),
        Side::Server,
        Default::default(),
    );

    let bootstrap: EmbeddingServiceImpl = EmbeddingServiceImpl::new();
    let rpc_system = RpcSystem::new(Box::new(network), Some(bootstrap.client));

    rpc_system.run().await?;
    Ok(())
}
```

## WebSocket Client (Rust)

`pkg/rs/src/ws_client.rs`:
```rust
use capnp_rpc::auto_reconnect;

async fn create_client(url: &str) -> anyhow::Result<EmbeddingServiceClient> {
    let (client, _) = auto_reconnect(|| async {
        let (ws, _) = tokio_tungstenite::connect_async(url).await?;
        let (read, write) = ws.split();

        let network = VatNetwork::new(
            WSReader::new(read),
            WSWriter::new(write),
            Side::Client,
            Default::default(),
        );

        let rpc_system = RpcSystem::new(Box::new(network), None);
        let client = rpc_system.bootstrap(Side::Server);

        tokio::spawn(rpc_system);
        Ok(client)
    });

    Ok(client)
}

async fn use_client(client: &EmbeddingServiceClient) -> anyhow::Result<()> {
    let mut request = client.store_request();
    request.get().set_embedding(/* ... */)?;

    let response = request.send().promise.await?;
    let id = response.get()?.get_id()?;

    println!("Stored embedding with id: {}", id);
    Ok(())
}
```

## Performance

- Zero-copy: Embeddings map directly from buffer
- SIMD-friendly: Fixed-size arrays for constant dimensions
- Pipelining: Chain `store().search()` in single RTT
- Bandwidth: Packed encoding saves 10-30%

## Related

- Zero-copy serialization: the `capnp` profile, `schema` package, `reference/zero_copy.md`
- WebSocket tunneling: the `capnp_rpc` profile, `rpc` package, `rules/websocket_tunneling.md`
- Reconnection: the `capnp_rpc` profile, `rpc` package, `reference/reconnection.md`
