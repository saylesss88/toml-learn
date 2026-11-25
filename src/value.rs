//! The shape of TOML data.
//!
//! Before writing any parsing code, ask: "what do I want to HAVE when I'm done?"
//! The answer is this enum. A TOML document is just a Table at the root.

use std::collections::BTreeMap;

/// A table maps keys to values. BTreeMap (not HashMap) so iteration order is
/// sorted and test output / Debug prints are stable.
pub type Table = BTreeMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Table(Table),
    // TOML also has dates/times. Skipped on purpose: add them last, if ever.
}

impl Value {
    /// Human-readable name of the variant, used in error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Table(_) => "table",
        }
    }

    // ---- Accessors -------------------------------------------------------
    // Pattern: "give me the inner thing IF it's that variant, else None".
    // Note the return types: we hand out borrows (&str, &Table), not clones.

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }

    /// STEP 2 (you): same pattern as as_integer.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(i) => Some(*i),
            _ => None,
        }
    }

    /// STEP 4 (you): what should the return type be? Borrow or own?
    /// Look at as_table for the answer.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
        // todo!("step 4: hint — &Vec<Value> coerces to &[Value]")
    }

    /// Look up a key if this value is a table.
    /// The `?` on an Option returns None early. Read this one slowly.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_table()?.get(key)
    }
}
