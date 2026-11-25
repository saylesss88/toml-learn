//! toml-learn: a hand-written TOML parser, built in stages.
//!
//! Layout (read in this order):
//!   value.rs    -> the SHAPE of parsed data (start here, everything flows from it)
//!   error.rs    -> what can go wrong
//!   parser.rs   -> text  -> Table   (generic TOML)
//!   manifest.rs -> Table -> Manifest (Cargo-specific, typed)
//!
//! Two layers on purpose: the parser knows nothing about Cargo, and the
//! manifest code knows nothing about text. That split is the single most
//! important design decision in this crate.

pub mod error;
pub mod manifest;
pub mod parser;
pub mod value;

// Re-exports so users write `toml_learn::parse` instead of `toml_learn::parser::parse`.
pub use error::ParseError;
pub use manifest::{Dependency, Manifest, ManifestError, Package};
pub use parser::parse;
pub use value::{Table, Value};
