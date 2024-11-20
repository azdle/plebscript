use anyhow::Result;
use std::collections::HashSet;
use viceroy_lib::{ExecuteCtx, ProfilingStrategy};

#[tokio::test]
async fn get_root() -> Result<()> {
    use http::request::Request;
    use hyper::body::Body;

    let adapt_core_wasm = false;
    let ctx = ExecuteCtx::new(
        "./bin/main.wasm",
        ProfilingStrategy::None,
        HashSet::new(),
        None,
        Default::default(),
        adapt_core_wasm,
    )?;

    let req = Request::get("/").body(Body::empty()).unwrap();
    let local = "127.0.0.1:80".parse().unwrap();
    let remote = "127.0.0.1:0".parse().unwrap();
    let _resp = ctx.handle_request(req, local, remote).await?;

    Ok(())
}
