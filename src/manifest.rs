//! Table -> Manifest (typed Cargo.toml).
//!
//! This layer never looks at text. It takes the generic `Table` the parser
//! produced and checks it has the shape Cargo expects. That's the same job
//! serde does for you in real code; doing it by hand shows you why serde exists.

use std::collections::BTreeMap;
use std::fmt;

use crate::error::ParseError;
use crate::parser;
use crate::value::{Table, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub package: Package,
    pub dependencies: BTreeMap<String, Dependency>,
    // STEP 7 (you): dev_dependencies, features, ...
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: Option<String>,
    pub authors: Vec<String>,
}

/// Cargo allows two spellings:
///   serde = "1.0"                                   -> Simple
///   serde = { version = "1.0", features = [...] }   -> Detailed
/// An enum is the natural "one of these shapes" type.
#[derive(Debug, Clone, PartialEq)]
pub enum Dependency {
    Simple(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        features: Vec<String>,
        optional: bool,
    },
}

// ---- Errors ---------------------------------------------------------------
// An enum this time (compare with ParseError's struct). Callers can match on it.

#[derive(Debug, Clone, PartialEq)]
pub enum ManifestError {
    /// The TOML itself was bad.
    Parse(ParseError),
    /// A required key is absent. Holds a dotted path like "package.name".
    Missing(String),
    /// Key exists but has the wrong type.
    WrongType {
        key: String,
        expected: &'static str,
        found: &'static str,
    },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::Parse(e) => write!(f, "invalid TOML: {e}"),
            ManifestError::Missing(key) => write!(f, "missing required key `{key}`"),
            ManifestError::WrongType {
                key,
                expected,
                found,
            } => {
                write!(f, "`{key}` should be a {expected}, found {found}")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

// This impl is what lets `parser::parse(input)?` work inside a fn returning
// Result<_, ManifestError>. `?` calls From::from on the error for you.
impl From<ParseError> for ManifestError {
    fn from(e: ParseError) -> Self {
        ManifestError::Parse(e)
    }
}

// ---- Parsing --------------------------------------------------------------

impl Manifest {
    /// REFACTOR: later, replace this with `impl std::str::FromStr for Manifest`
    /// so callers can write `input.parse::<Manifest>()`.
    pub fn parse(input: &str) -> Result<Self, ManifestError> {
        let root = parser::parse(input)?;

        let package_table = required_table(&root, "package")?;
        let package = Package::from_table(package_table)?;

        let mut dependencies = BTreeMap::new();
        // [dependencies] is optional, so `if let` not `?`.
        if let Some(dep_table) = optional_table(&root, "dependencies")? {
            for (name, value) in dep_table {
                let dep = Dependency::from_value(name, value)?;
                dependencies.insert(name.clone(), dep);
            }
        }

        Ok(Manifest {
            package,
            dependencies,
        })
    }
}

impl Package {
    fn from_table(t: &Table) -> Result<Self, ManifestError> {
        Ok(Package {
            name: required_str(t, "package", "name")?,
            version: required_str(t, "package", "version")?,
            edition: optional_str(t, "package", "edition")?,
            // STEP 7 (you): read `authors = ["a", "b"]` into Vec<String>.
            // Needs step 4 (arrays) first. Write an `optional_str_array` helper
            // next to the others below.
            authors: Vec::new(),
        })
    }
}

impl Dependency {
    fn from_value(name: &str, value: &Value) -> Result<Self, ManifestError> {
        match value {
            Value::String(v) => Ok(Dependency::Simple(v.clone())),
            Value::Table(_t) => {
                // STEP 7 (you): build Dependency::Detailed from the table.
                // Every field is optional; `optional` defaults to false.
                // The helpers below take a `section` for error messages —
                // what should it be here? (format!("dependencies.{name}"))
                todo!("step 7: detailed dependency `{name}`")
            }
            other => Err(ManifestError::WrongType {
                key: format!("dependencies.{name}"),
                expected: "string or table",
                found: other.type_name(),
            }),
        }
    }
}

// ---- Small helpers ---------------------------------------------------------
// Notice the repetition below. That's fine in v1. In the refactor you might
// collapse these with a trait or a generic. Don't do it until you've written
// the 4th near-copy and feel the pain — premature abstraction is worse.

fn required_table<'a>(root: &'a Table, key: &str) -> Result<&'a Table, ManifestError> {
    optional_table(root, key)?.ok_or_else(|| ManifestError::Missing(key.to_string()))
}

fn optional_table<'a>(root: &'a Table, key: &str) -> Result<Option<&'a Table>, ManifestError> {
    match root.get(key) {
        None => Ok(None),
        Some(Value::Table(t)) => Ok(Some(t)),
        Some(other) => Err(ManifestError::WrongType {
            key: key.to_string(),
            expected: "table",
            found: other.type_name(),
        }),
    }
}

fn required_str(t: &Table, section: &str, key: &str) -> Result<String, ManifestError> {
    optional_str(t, section, key)?.ok_or_else(|| ManifestError::Missing(format!("{section}.{key}")))
}

fn optional_str(t: &Table, section: &str, key: &str) -> Result<Option<String>, ManifestError> {
    match t.get(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(ManifestError::WrongType {
            key: format!("{section}.{key}"),
            expected: "string",
            found: other.type_name(),
        }),
    }
}
