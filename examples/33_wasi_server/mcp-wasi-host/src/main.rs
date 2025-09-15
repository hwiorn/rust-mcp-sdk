use anyhow::Result;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::bindings::wasi::http::types::{
    Headers, IncomingBody, IncomingRequest, Method, OutgoingBody, OutgoingResponse, Scheme,
};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

// Define the host state, including the WASI context and the HTTP context.
struct MyState {
    table: Table,
    wasi: WasiCtx,
    http: WasiHttpCtx,
}

impl WasiView for MyState {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for MyState {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

async fn handle_request(
    req: Request<Incoming>,
    engine: Engine,
    component: Component,
) -> Result<Response<Full<Bytes>>> {
    let mut store = Store::new(
        &engine,
        MyState {
            table: Table::new(),
            wasi: WasiCtxBuilder::new().inherit_stdout().build(),
            http: WasiHttpCtx::new(),
        },
    );

    let (parts, body) = req.into_parts();
    let body_bytes = http_body_util::BodyExt::collect(body).await?.to_bytes();

    let headers = Headers::from_list(
        &parts
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect::<Vec<_>>(),
    )?;

    let req = IncomingRequest::new(
        &Method::Post,
        &"/".to_string(),
        &Scheme::Http,
        &"localhost".to_string(),
        headers,
    );
    let body_stream = req.consume()?;
    body_stream.write(&body_bytes)?;
    body_stream.finish(None)?;

    let out = OutgoingResponse::new(Headers::new());

    let handler = WasiHttpImpl::new(req, out);
    let mut linker = Linker::new(&engine);
    wasmtime_wasi_http::add_to_linker(&mut linker)?;

    let (instance, _) = linker
        .instantiate_async(&mut store, &component, &handler)
        .await?;

    let response = instance.exports().wasi_http_incoming_handler();
    response.handle(req, out)?;

    let resp = handler.into_response().await?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;

    // Path to the compiled WASM module.
    let wasm_path = "../mcp-wasi-server/target/wasm32-wasi/release/mcp_wasi_server.wasm";
    let component = Component::from_file(&engine, wasm_path)?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let engine = engine.clone();
        let component = component.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(move |req| {
                    handle_request(req, engine.clone(), component.clone())
                }))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
