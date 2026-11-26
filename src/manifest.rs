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
    pub dev_dependencies: BTreeMap<String, Dependency>,
    pub features: BTreeMap<String, Vec<String>>,
    // STEP 7 (you): dev_dependencies, features, ...
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
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

        let dependencies = read_deps(&root, "dependencies")?;
        let dev_dependencies = read_deps(&root, "dev-dependencies")?;
        let features = read_features(&root)?;

        Ok(Manifest {
            package,
            dependencies,
            dev_dependencies,
            features,
        })
    }
}

fn read_features(root: &Table) -> Result<BTreeMap<String, Vec<String>>, ManifestError> {
    let mut feat = BTreeMap::new();
    if let Some(f) = optional_table(root, "features")? {
        for (name, value) in f {
            let list = str_array(value, &format!("features.{name}"))?;
            feat.insert(name.clone(), list);
        }
    }
    Ok(feat)
}

fn read_deps(root: &Table, key: &str) -> Result<BTreeMap<String, Dependency>, ManifestError> {
    let mut map = BTreeMap::new();
    if let Some(deps) = optional_table(root, key)? {
        for (name, value) in deps {
            let dep = Dependency::from_value(name, value)?;
            map.insert(name.clone(), dep);
        }
    }
    Ok(map)
}

impl Package {
    fn from_table(t: &Table) -> Result<Self, ManifestError> {
        Ok(Package {
            name: required_str(t, "package", "name")?,
            version: required_str(t, "package", "version")?,
            edition: optional_str(t, "package", "edition")?,
            authors: optional_str_array(t, "package", "authors")?,
            description: optional_str(t, "package", "description")?,
        })
    }
}

impl Dependency {
    fn from_value(name: &str, value: &Value) -> Result<Self, ManifestError> {
        let section = format!("dependencies.{name}");
        match value {
            Value::String(v) => Ok(Dependency::Simple(v.clone())),
            Value::Table(t) => Ok(Dependency::Detailed {
                version: optional_str(t, &section, "version")?,
                path: optional_str(t, &section, "path")?,
                features: optional_str_array(t, &section, "features")?,
                optional: optional_bool(t, &section, "optional")?.unwrap_or(false),
            }),
            other => Err(ManifestError::WrongType {
                key: section,
                expected: "string or table",
                found: other.type_name(),
            }),
        }
    }
}

// ---- Small helpers ---------------------------------------------------------
//
// - The `optional_*` ones look up a key, accept it if it's missing, and check
//   its type if it's there.
// - The `required_*` ones wrap an optional one and turn "missing" into an
//   error.
//
// Notice the repetition below. That's fine in v1. In the refactor you might
// collapse these with a trait or a generic. Don't do it until you've written
// the 4th near-copy and feel the pain — premature abstraction is worse.
fn str_array(val: &Value, key: &str) -> Result<Vec<String>, ManifestError> {
    match val {
        Value::Array(items) => {
            let mut storage = Vec::new();
            for item in items {
                match item.as_str() {
                    Some(s) => storage.push(s.to_string()),
                    None => {
                        return Err(ManifestError::WrongType {
                            key: key.to_string(),
                            expected: "string",
                            found: item.type_name(),
                        });
                    }
                }
            }
            Ok(storage)
        }
        other => Err(ManifestError::WrongType {
            key: key.to_string(),
            expected: "array of strings",
            found: other.type_name(),
        }),
    }
}

fn optional_str_array(t: &Table, section: &str, key: &str) -> Result<Vec<String>, ManifestError> {
    match t.get(key) {
        None => Ok(Vec::new()),
        Some(v) => str_array(v, &format!("{section}.{key}")),
    }
}

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

/// `t` is whichever table you're looking in right now.
/// `key` is what to look up in `t`, which is either there or isn't.
/// `section` is the name of that particular table: its path from root like `"package"`
/// or `"dependencies.clap"`.
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

fn optional_bool(t: &Table, section: &str, key: &str) -> Result<Option<bool>, ManifestError> {
    match t.get(key) {
        None => Ok(None),
        Some(Value::Boolean(b)) => Ok(Some(*b)),
        Some(other) => Err(ManifestError::WrongType {
            key: format!("{section}.{key}"),
            expected: "boolean",
            found: other.type_name(),
        }),
    }
}
