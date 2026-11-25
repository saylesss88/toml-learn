//! Your playground. Parse any file and pretty-print the result.
//!
//!   cargo run --example dump -- fixtures/simple.toml
//!   cargo run --example dump -- Cargo.toml --manifest
//!
//! When stuck, make a tiny .toml file that shows the problem and run this on it.

use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("usage: dump <file.toml> [--manifest]");
        process::exit(2);
    };
    let as_manifest = args.iter().any(|a| a == "--manifest");

    let text = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("can't read {path}: {e}");
        process::exit(1);
    });

    let result = if as_manifest {
        toml_learn::Manifest::parse(&text).map(|m| format!("{m:#?}")).map_err(|e| e.to_string())
    } else {
        toml_learn::parse(&text).map(|t| format!("{t:#?}")).map_err(|e| e.to_string())
    };

    match result {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}
