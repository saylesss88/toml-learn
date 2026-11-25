//! Things v1 already does. These should pass on day one.
//! If you break one while working on a step, you'll know immediately.

use toml_learn::{parse, Dependency, Manifest, ManifestError, Value};

#[test]
fn key_values_and_types() {
    let t = parse("a = \"hi\"\nb = 42\nc = true\nd = 1_000").unwrap();
    assert_eq!(t["a"], Value::String("hi".into()));
    assert_eq!(t["b"].as_integer(), Some(42));
    assert_eq!(t["c"].as_bool(), Some(true));
    assert_eq!(t["d"].as_integer(), Some(1000));
}

#[test]
fn comments_and_blank_lines() {
    let t = parse("# top\n\nx = 1 # trailing\n").unwrap();
    assert_eq!(t.len(), 1);
    assert_eq!(t["x"].as_integer(), Some(1));
}

#[test]
fn tables_and_nested_headers() {
    let t = parse("[owner]\nname = \"Tom\"\n[server.dev]\nport = 8080").unwrap();
    assert_eq!(t["owner"].get("name").and_then(Value::as_str), Some("Tom"));
    let port = t["server"].get("dev").and_then(|d| d.get("port"));
    assert_eq!(port.and_then(Value::as_integer), Some(8080));
}

#[test]
fn empty_table_exists() {
    let t = parse("[features]").unwrap();
    assert!(t["features"].as_table().unwrap().is_empty());
}

#[test]
fn duplicate_key_is_error() {
    let err = parse("a = 1\na = 2").unwrap_err();
    assert_eq!(err.line, 2);
    assert!(err.message.contains("duplicate"), "{err}");
}

#[test]
fn garbage_is_error_with_line_number() {
    let err = parse("ok = 1\nthis is not toml").unwrap_err();
    assert_eq!(err.line, 2);
}

#[test]
fn table_over_value_is_error() {
    let err = parse("a = 1\n[a]").unwrap_err();
    assert!(err.message.contains("not a table"), "{err}");
}

#[test]
fn simple_manifest() {
    let src = r#"
[package]
name = "demo"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
"#;
    let m = Manifest::parse(src).unwrap();
    assert_eq!(m.package.name, "demo");
    assert_eq!(m.package.edition.as_deref(), Some("2021"));
    assert_eq!(m.dependencies["anyhow"], Dependency::Simple("1".into()));
}

#[test]
fn manifest_missing_name() {
    let err = Manifest::parse("[package]\nversion = \"1\"").unwrap_err();
    assert_eq!(err, ManifestError::Missing("package.name".into()));
}

#[test]
fn manifest_wrong_type() {
    let err = Manifest::parse("[package]\nname = 5\nversion = \"1\"").unwrap_err();
    assert!(matches!(err, ManifestError::WrongType { expected: "string", .. }));
}
