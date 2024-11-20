use std::collections::BTreeMap;

use anyhow::Result;
use piccolo::{Closure, Executor, Fuel, Lua, Table, Value, Variadic};
use serde::{Deserialize, Serialize};

static WEBSCRIPT_LIB: &[u8] = include_bytes!("webscript.lua");

#[derive(Serialize, Default)]
pub struct ScriptRequest {
    /// A table consisting of form data posted to your script. This field is only present if the request has a Content-Type of multipart/form-data or application/x-www-form-urlencode and the body is successfully parsed as form-encoded.
    pub form: BTreeMap<String, String>,
    /// A table consisting of query string parameters from the request's URL. For example, the query string ?color=red&number;=3 will result in a query table of {color="red", number="3"}.
    pub query: BTreeMap<String, String>,
    /// The raw query string from the URL. Using the previous example, the querystring value would be "color=red&number;=3".
    pub querystring: Option<String>,
    /// A table consisting of files included in a form post. For each included file, the key is the name of the form's input field, and the value is a table consisting of type (the content-type of the uploaded file), filename (original file name), and content (the raw contents of the file).
    pub files: Option<BTreeMap<String, String>>,
    /// The content of the request, after it's been decoded as needed (e.g. decompressed as specified by a Content-Encoding header).
    pub body: String,
    /// The HTTP request method. (E.g., GET, POST, ...)
    pub method: String,
    /// The request's origin IP address.
    pub remote_addr: String,
    /// The URL scheme. (E.g., http or https.)
    pub scheme: String,
    /// The URL's port. (E.g., 80 or 443.)
    pub port: u16,
    /// The URL's path. (E.g., for the URL http://example.webscript.io/foo/bar, the path is /foo/bar.)
    pub path: String,
    /// A table of the HTTP request headers. Keys are "train-cased," like Content-Type.
    pub headers: BTreeMap<String, String>,
}

#[derive(Deserialize)]
pub struct ScriptResponse {
    pub status_code: Option<u16>,
    pub headers: Option<BTreeMap<String, String>>,
    pub body: Option<ScriptResponseBody>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ScriptResponseBody {
    String(String),
    Table(serde_json::Value),
}

pub fn run_lua(src: &[u8], req: ScriptRequest) -> Result<ScriptResponse> {
    let name = "user script";
    let mut lua = Lua::core();

    Ok(lua.try_enter(|ctx| {
        // populate `request` table
        let req_value = piccolo_util::serde::ser::to_value(ctx, &req).expect("valuize request");
        ctx.set_global("request", req_value);

        // load webscript lib
        let webscript =
            Closure::load(ctx, Some("webscript.lua"), WEBSCRIPT_LIB).expect("load webscript lib");

        let exe = Executor::start(ctx, webscript.into(), ());

        let mut fuel = Fuel::with(8192);
        while !exe.step(ctx, &mut fuel).expect("step WS") {
            fuel.refill(8192, 8192);
        }

        let ws: Table = exe
            .take_result(ctx)
            .expect("failed to load webscript lib")
            .expect("take ws lib");

        let Ok(Value::Table(ws_env)) = ws.get(ctx, "env") else {
            panic!("WS.env value wasn't a table");
        };

        let Ok(Value::Function(process_response)) = ws.get(ctx, "process_response") else {
            panic!("`process_response` wasn't a function")
        };

        // run user script in webscript envi
        // TODO: this is still getting the global env for some reason
        let script = Closure::load_with_env(ctx, Some(name), src, ws_env).expect("load");

        let exe = Executor::start(ctx, script.into(), ());

        let mut fuel = Fuel::with(8192);
        while !exe.step(ctx, &mut fuel).expect("step script") {
            fuel.refill(8192, 8192);
        }

        let res: Variadic<Vec<Value>> = exe
            .take_result(ctx)
            .expect("failed to run to completion")
            .expect("take intermideate result");

        println!("res: {res:?}");

        // process webscript-style response, convert to structured response
        let exe = Executor::start(ctx, process_response, res);

        let mut fuel = Fuel::with(8192);
        while !exe.step(ctx, &mut fuel).expect("step PR") {
            fuel.refill(8192, 8192);
        }

        let resp_tbl = exe
            .take_result(ctx)
            .expect("failed to run to completion")
            .expect("take result table");

        let sresp: ScriptResponse = piccolo_util::serde::de::from_value(resp_tbl).unwrap();

        Ok(sresp)
    })?)
}
