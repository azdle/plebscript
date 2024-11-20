use plebscript::{run_lua, ScriptResponseBody};

#[test]
fn return_string() {
    let src = br#"return "Hello, world!""#;
    let request = Default::default();

    let response = run_lua(src, request).unwrap();

    assert_eq!(response.body.unwrap(), "Hello, world!");
    assert_eq!(response.status_code, None);
    assert_eq!(response.headers, None);
}

#[test]
fn return_table() {
    let src = br#"return { color = "purple", count = 37 }"#;
    let request = Default::default();

    let response = run_lua(src, request).unwrap();

    assert!(matches!(
        response.body.unwrap(),
        ScriptResponseBody::Table(_)
    ));
    assert_eq!(response.status_code, None);
    assert_eq!(response.headers, None);
}

#[test]
fn return_number() {
    let src = br#"return 204"#;
    let request = Default::default();

    let response = run_lua(src, request).unwrap();

    assert_eq!(response.body, None);
    assert_eq!(response.status_code, Some(204));
    assert_eq!(response.headers, None);
}

#[test]
fn return_string_table() {
    let src = br#"return "<h1>Hello, world!</h1>", { ['Content-Type'] = "text/html" }"#;
    let request = Default::default();

    let response = run_lua(src, request).unwrap();

    assert!(response.body.is_some());
    assert_eq!(response.status_code, None);
    assert_eq!(
        response.headers.unwrap(),
        [("Content-Type".into(), "text/html".into())].into()
    );
}

#[test]
fn return_table_table() {
    let src = br#"return "#;
    let request = Default::default();

    let response = run_lua(src, request).unwrap();

    assert_eq!(response.body, None);
    assert_eq!(response.status_code, None);
    assert_eq!(response.headers, None);
}

#[test]
fn return_number_string() {
    let src = br#"return 418, "I'm a teapot""#;
    let request = Default::default();

    let response = run_lua(src, request).unwrap();

    assert_eq!(response.body.unwrap(), "I'm a teapot");
    assert_eq!(response.status_code, Some(418));
    assert_eq!(response.headers, None);
}
