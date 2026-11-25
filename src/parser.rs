//! Text -> Table.
//!
//! v1 STRATEGY: line-by-line. Each non-empty line is either
//!   - a table header:  [name] or [a.b]
//!   - a key/value:     key = value
//!
//! This is how most people would write it first, and it gets surprisingly far.
//! It WILL break on multi-line arrays and on '#' or ',' inside strings.
//! Hitting those walls is the lesson: they motivate the refactor to a
//! character-by-character scanner (see GUIDE.md, phase 2).

use crate::error::ParseError;
use crate::value::{Table, Value};

/// Parse a whole TOML document into its root table.
pub fn parse(input: &str) -> Result<Table, ParseError> {
    let mut root = Table::new();
    // Which table are we currently inside? [] = root, ["package"], ["a","b"]...
    let mut current_path: Vec<String> = Vec::new();

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1; // humans count from 1
        let line = strip_comment(raw_line).trim();

        if line.is_empty() {
            continue;
        }

        // [[array.of.tables]] -- out of scope for v1, fail loudly not silently.
        if line.starts_with("[[") {
            return Err(ParseError::new(
                line_no,
                "arrays of tables `[[..]]` are not supported yet",
            ));
        }

        // [table] header
        if let Some(rest) = line.strip_prefix('[') {
            let header = rest
                .strip_suffix(']')
                .ok_or_else(|| ParseError::new(line_no, "unclosed table header"))?;
            current_path = split_key(header);
            if current_path.iter().any(|k| k.is_empty()) {
                return Err(ParseError::new(line_no, "empty key in table header"));
            }
            // Create it now so empty tables like `[features]` still exist.
            get_or_create_table(&mut root, &current_path, line_no)?;
            continue;
        }

        // key = value
        let (key, value_text) = line.split_once('=').ok_or_else(|| {
            ParseError::new(line_no, format!("expected `key = value`, got `{line}`"))
        })?;

        let key = key.trim();
        if key.is_empty() {
            return Err(ParseError::new(line_no, "missing key before `=`"));
        }
        // STEP 6 (you): dotted keys. `a.b = 1` should mean table a, key b.
        // Right now "a.b" is stored as one literal key. Hint: split_key + the
        // helper below already do 90% of the work.

        let value = parse_value(value_text.trim(), line_no)?;
        let table = get_or_create_table(&mut root, &current_path, line_no)?;

        if table.contains_key(key) {
            return Err(ParseError::new(line_no, format!("duplicate key `{key}`")));
        }
        table.insert(key.to_string(), value);
    }

    Ok(root)
}

/// Remove a trailing `# comment`.
///
/// KNOWN BUG (STEP 1, yours): `name = "a#b"` gets cut at the '#'.
/// You need to know whether you're inside quotes when you see '#'.
/// Hint: `line.char_indices()` + a `bool` you flip on '"'.
fn strip_comment(line: &str) -> &str {
    let mut inside_quotes = false;
    for (i, c) in line.char_indices() {
        if c == '"' {
            inside_quotes = !inside_quotes;
        } else if c == '#' && !inside_quotes {
            return &line[..i];
        }
    }
    line
}

/// "a . b" -> ["a", "b"]
fn split_key(s: &str) -> Vec<String> {
    s.split('.').map(|part| part.trim().to_string()).collect()
}

/// Walk down `path` from `root`, creating empty tables as needed, and return
/// a mutable reference to the innermost one.
///
/// This is the trickiest borrow in v1. Read it twice. The trick: `current` is
/// *reassigned* to a borrow of something inside itself each loop iteration.
fn get_or_create_table<'a>(
    root: &'a mut Table,
    path: &[String],
    line_no: usize,
) -> Result<&'a mut Table, ParseError> {
    let mut current = root;
    for key in path {
        let entry = current
            .entry(key.clone())
            .or_insert_with(|| Value::Table(Table::new()));
        current = match entry {
            Value::Table(t) => t,
            other => {
                return Err(ParseError::new(
                    line_no,
                    format!("`{key}` is already a {}, not a table", other.type_name()),
                ))
            }
        };
    }
    Ok(current)
}

/// Text on the right of `=` -> Value.
///
/// Order matters: check the unambiguous prefixes ('"', '[', '{') first,
/// then try numbers last.
fn parse_value(s: &str, line_no: usize) -> Result<Value, ParseError> {
    if s.is_empty() {
        return Err(ParseError::new(line_no, "missing value after `=`"));
    }

    // Basic string. STEP 3 (you): escapes like \" and \n, plus 'literal' strings.
    if let Some(rest) = s.strip_prefix('"') {
        let inner = rest
            .strip_suffix('"')
            .ok_or_else(|| ParseError::new(line_no, "unterminated string"))?;
        return Ok(Value::String(inner.to_string()));
    }

    match s {
        "true" => return Ok(Value::Boolean(true)),
        "false" => return Ok(Value::Boolean(false)),
        _ => {}
    }

    if s.starts_with('[') {
        return parse_array(s, line_no);
    }
    if s.starts_with('{') {
        return parse_inline_table(s, line_no);
    }

    // TOML allows 1_000_000. `replace` allocates; fine for v1.
    let digits = s.replace('_', "");
    if let Ok(i) = digits.parse::<i64>() {
        return Ok(Value::Integer(i));
    }

    // STEP 2 (you): floats. 3.14, -0.5, 1e6, 6.02e23
    // Hint: same move as integers, different target type. Mind the order.
    if let Ok(f) = digits.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    Err(ParseError::new(
        line_no,
        format!("can't understand value `{s}`"),
    ))
}

/// STEP 4 (you): `[1, 2, 3]`, `["a", "b"]`, `[]`, `[ [1,2], [3] ]`
///
/// Plan before code:
///   1. strip the outer [ ]
///   2. split the inside on commas -- but ONLY top-level commas
///      (not ones inside "strings" or nested [brackets])
///   3. parse_value each piece (recursion!)
///   4. collect into Value::Array
///
/// Step 2 is the real problem. Write it as its own fn and test it alone.
/// You'll reuse it in step 5.
fn parse_array(s: &str, line_no: usize) -> Result<Value, ParseError> {
    let _ = (s, line_no);
    todo!("step 4: parse a single-line array")
}

/// STEP 5 (you): `{ version = "1", features = ["derive"] }`
/// Same splitter as arrays, then each piece is `key = value`.
fn parse_inline_table(s: &str, line_no: usize) -> Result<Value, ParseError> {
    let _ = (s, line_no);
    todo!("step 5: parse an inline table")
}

// Unit tests for private helpers live next to them. Integration tests
// (public API only) live in tests/.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comment_basic() {
        assert_eq!(strip_comment("a = 1 # hi"), "a = 1 ");
        assert_eq!(strip_comment("# whole line"), "");
        assert_eq!(strip_comment("no comment"), "no comment");
    }

    #[test]
    fn strip_comment_ignores_hash_in_string() {
        assert_eq!(strip_comment(r##"url = "a#b" # c"##), r##"url = "a#b" "##);
    }

    #[test]
    fn split_key_trims() {
        assert_eq!(split_key("a . b"), vec!["a", "b"]);
    }
}
