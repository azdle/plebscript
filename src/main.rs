use std::collections::BTreeMap;

use fastly::http::StatusCode;
use fastly::{Error, Request, Response};
use include_dir::{include_dir, Dir};
use piccolo::{Closure, Executor, Fuel, Lua, Table, Value, Variadic};
use serde::{Deserialize, Serialize};

static SRC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/www/src");
// TODO: Piccolo doesn't support `require` yet.
// static LIB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/www/lib");
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/www/static");
static WEBSCRIPT_LIB: &[u8] = include_bytes!("webscript.lua");

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    println!(
        "FASTLY_SERVICE_VERSION: {}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap_or_else(|_| String::new())
    );

    let mut path = req.get_path().to_string();

    // `include_dir` doesn't use a leading `/`
    path.remove(0);

    // first, check for static file
    if let Some(file) = STATIC_DIR.get_file(&path) {
        return Ok(Response::from_body(file.contents()));
    }

    // find matching script file
    if path.ends_with('/') || path.is_empty() {
        path.push_str("init.lua");
    } else {
        path.push_str(".lua");
    };

    println!("script path: {path}");
    println!("SCRIPT_DIR: {SRC_DIR:?}");

    let Some(src) = SRC_DIR.get_file(&path) else {
        return Ok(
            Response::from_status(StatusCode::NOT_FOUND).with_body_text_plain("404 Not Found")
        );
    };

    let sreq = ScriptRequest {
        // TODO: form
        query: dbg!(req.get_query()).unwrap_or_default(),
        querystring: req.get_query_str().map(ToString::to_string),
        // TODO: files
        method: req.get_method_str().to_string(),
        remote_addr: req
            .get_client_ip_addr()
            .expect("client request has client IP")
            .to_string(),
        scheme: req.get_url().scheme().to_string(),
        path: req.get_url().path().to_string(),
        port: req
            .get_url()
            .port()
            .unwrap_or(if req.get_url().scheme() == "https" {
                443
            } else {
                80
            }),
        headers: req
            .get_headers()
            .map(|(n, v)| {
                (
                    n.to_string(),
                    v.to_str()
                        .expect("convert header value to String")
                        .to_string(),
                )
            })
            .collect(),
        body: req.into_body_str_lossy(),
        ..Default::default()
    };

    Ok(run_lua(src.contents(), sreq).unwrap())
}

#[derive(Serialize, Default)]
struct ScriptRequest {
    /// A table consisting of form data posted to your script. This field is only present if the request has a Content-Type of multipart/form-data or application/x-www-form-urlencode and the body is successfully parsed as form-encoded.
    form: BTreeMap<String, String>,
    /// A table consisting of query string parameters from the request's URL. For example, the query string ?color=red&number;=3 will result in a query table of {color="red", number="3"}.
    query: BTreeMap<String, String>,
    /// The raw query string from the URL. Using the previous example, the querystring value would be "color=red&number;=3".
    querystring: Option<String>,
    /// A table consisting of files included in a form post. For each included file, the key is the name of the form's input field, and the value is a table consisting of type (the content-type of the uploaded file), filename (original file name), and content (the raw contents of the file).
    files: Option<BTreeMap<String, String>>,
    /// The content of the request, after it's been decoded as needed (e.g. decompressed as specified by a Content-Encoding header).
    body: String,
    /// The HTTP request method. (E.g., GET, POST, ...)
    method: String,
    /// The request's origin IP address.
    remote_addr: String,
    /// The URL scheme. (E.g., http or https.)
    scheme: String,
    /// The URL's port. (E.g., 80 or 443.)
    port: u16,
    /// The URL's path. (E.g., for the URL http://example.webscript.io/foo/bar, the path is /foo/bar.)
    path: String,
    /// A table of the HTTP request headers. Keys are "train-cased," like Content-Type.
    headers: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct ScriptResponse {
    status_code: Option<u16>,
    headers: Option<BTreeMap<String, String>>,
    body: Option<ScriptResponseBody>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ScriptResponseBody {
    String(String),
    Table(serde_json::Value),
}

fn run_lua(src: &[u8], req: ScriptRequest) -> Result<Response, Error> {
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

        // build response
        let mut resp = Response::new();

        // response status
        if let Some(status_code) = sresp.status_code {
            resp.set_status(status_code);
        }

        // response headers
        if let Some(headers) = sresp.headers {
            for (header, value) in headers {
                resp.set_header(header, value)
            }
        }

        // response body
        match dbg!(sresp.body) {
            Some(ScriptResponseBody::String(body)) => {
                resp.set_body(body);
            }
            Some(ScriptResponseBody::Table(tbl_body)) => {
                resp.set_body_json(&tbl_body).expect("set json body");
            }
            None => (),
        }

        Ok(resp)
    })?)
}
