use fastly::http::StatusCode;
use fastly::{Error, Request, Response};
use include_dir::{include_dir, Dir};
use plebscript::{run_lua, ScriptRequest, ScriptResponseBody};

static SRC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../www/src");
// TODO: Piccolo doesn't support `require` yet.
// static LIB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../www/lib");
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../www/static");

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

    let Some(src) = SRC_DIR.get_file(&path) else {
        println!("{path}, not found in script dir:\n{SRC_DIR:?}");

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

    let sresp = match run_lua(src.contents(), sreq) {
        Ok(sresp) => sresp,
        Err(e) => {
            return Ok(Response::new()
                .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain(&format!("{:?}", e)));
        }
    };

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
}
