//! Your roadmap, as tests. Each is #[ignore]d.
//!
//! Workflow for each step:
//!   1. Delete the `#[ignore]` line on ONE test.
//!   2. cargo test <name>          -> watch it fail. Read the failure.
//!   3. Make it pass. Nothing more.
//!   4. cargo test                 -> everything still green?
//!   5. Commit. Next.
//!
//! See all remaining: cargo test -- --ignored

use toml_learn::{Dependency, Manifest, Value, parse};

// ---- STEP 1: '#' inside strings (also see unit test in parser.rs) ---------
#[test]
// #[ignore = "step 1"]
fn step1_hash_inside_string() {
    let t = parse(r##"url = "https://x.com/#frag" # real comment"##).unwrap();
    assert_eq!(t["url"].as_str(), Some("https://x.com/#frag"));
}

// ---- STEP 2: floats ------------------------------------------------------
#[test]
// #[ignore = "step 2"]
fn step2_floats() {
    let t = parse("a = 3.14\nb = -0.5\nc = 1e6\nd = 7").unwrap();
    assert_eq!(t["a"].as_float(), Some(3.14));
    assert_eq!(t["b"].as_float(), Some(-0.5));
    assert_eq!(t["c"].as_float(), Some(1e6));
    assert_eq!(t["d"].as_integer(), Some(7), "integers must stay integers");
}

// ---- STEP 3: strings done properly ---------------------------------------
#[test]
// #[ignore = "step 3"]
fn step3_escapes() {
    let t = parse(r#"s = "say \"hi\"\n\\""#).unwrap();
    assert_eq!(t["s"].as_str(), Some("say \"hi\"\n\\"));
    println!("{t:?}");
}

#[test]
// #[ignore = "step 3"]
fn step3_literal_strings() {
    // Single quotes: no escapes at all. Great for Windows paths & regex.
    let t = parse(r#"p = 'C:\Users\tom'"#).unwrap();
    assert_eq!(t["p"].as_str(), Some(r"C:\Users\tom"));
}

// ---- STEP 4: single-line arrays ------------------------------------------
#[test]
#[ignore = "step 4"]
fn step4_arrays() {
    let t = parse(
        r#"a = [1, 2, 3]
b = ["x", "y"]
c = []
d = [[1, 2], [3]]
e = ["has, comma", "ok"]"#,
    )
    .unwrap();
    assert_eq!(t["a"].as_array().map(|a| a.len()), Some(3));
    assert_eq!(t["b"].as_array().unwrap()[1].as_str(), Some("y"));
    assert_eq!(t["c"].as_array(), Some(&[][..]));
    assert_eq!(t["d"].as_array().unwrap()[0].as_array().unwrap().len(), 2);
    assert_eq!(t["e"].as_array().unwrap()[0].as_str(), Some("has, comma"));
}

#[test]
#[ignore = "step 4"]
fn step4_trailing_comma_allowed() {
    let t = parse("a = [1, 2,]").unwrap();
    assert_eq!(t["a"].as_array().unwrap().len(), 2);
}

// ---- STEP 5: inline tables -----------------------------------------------
#[test]
#[ignore = "step 5"]
fn step5_inline_table() {
    let t = parse(r#"dep = { version = "1.0", features = ["derive", "std"] }"#).unwrap();
    let dep = &t["dep"];
    assert_eq!(dep.get("version").and_then(Value::as_str), Some("1.0"));
    assert_eq!(
        dep.get("features")
            .and_then(Value::as_array)
            .map(|a| a.len()),
        Some(2)
    );
}

// ---- STEP 6: dotted keys -------------------------------------------------
#[test]
#[ignore = "step 6"]
fn step6_dotted_keys() {
    let t = parse("a.b.c = 1\na.d = 2").unwrap();
    assert_eq!(
        t["a"]
            .get("b")
            .and_then(|b| b.get("c"))
            .and_then(Value::as_integer),
        Some(1)
    );
    assert_eq!(t["a"].get("d").and_then(Value::as_integer), Some(2));
}

// ---- STEP 7: richer manifest ---------------------------------------------
#[test]
#[ignore = "step 7"]
fn step7_authors() {
    let m =
        Manifest::parse("[package]\nname = \"x\"\nversion = \"1\"\nauthors = [\"Tom\"]").unwrap();
    assert_eq!(m.package.authors, vec!["Tom".to_string()]);
}

#[test]
#[ignore = "step 7"]
fn step7_detailed_dependency() {
    let src = r#"
[package]
name = "x"
version = "1"
[dependencies]
clap = { version = "4", features = ["derive"] }
local = { path = "../local", optional = true }
"#;
    let m = Manifest::parse(src).unwrap();
    assert_eq!(
        m.dependencies["clap"],
        Dependency::Detailed {
            version: Some("4".into()),
            path: None,
            features: vec!["derive".into()],
            optional: false,
        }
    );
    assert!(matches!(
        m.dependencies["local"],
        Dependency::Detailed { optional: true, .. }
    ));
}

// ---- STEP 8: the wall (needs the phase-2 refactor) -----------------------
#[test]
#[ignore = "step 8 / phase 2"]
fn step8_real_cargo_toml() {
    let src = include_str!("../fixtures/real_cargo.toml");
    let m = Manifest::parse(src).unwrap();
    assert_eq!(m.package.name, "jj-example");
    assert_eq!(m.dependencies.len(), 3);
}
