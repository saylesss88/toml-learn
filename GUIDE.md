# toml-learn: the guide

## 0. First 10 minutes

```sh
cargo test                                   # 10 passing basics, 11 ignored steps
cargo run --example dump -- fixtures/simple.toml
cargo test -- --ignored                      # your to-do list, failing
```

Read the files in this order: `value.rs` → `error.rs` → `parser.rs` →
`manifest.rs`.

## 1. How it's put together

```
"text"  --parser::parse-->  Table (BTreeMap<String, Value>)  --Manifest::parse-->  Manifest
         knows TOML syntax,                                    knows Cargo, not text
         not Cargo
```

- **`Value` is the center of everything.** It's what you have once parsing is
  done. Every function either produces a `Value` or reads one. If you're unsure
  what a function should return, it's probably `Result<Value, ParseError>`.
- **v1 parses one line at a time.** Every line is blank, a `[header]`, or
  `key = value`. A `current_path: Vec<String>` keeps track of which table you're
  in.
- **`get_or_create_table`** handles the hardest borrow in v1: a `&mut` reference
  that keeps moving down into nested tables. If you get borrow-checker errors,
  copy this pattern rather than fighting it.
- **The manifest layer** turns loose data into checked types (`Option`, `Vec`,
  enums). You do it by hand here, which is the same work `serde` does for you in
  real code.
- **Two error styles on purpose.** `ParseError` is a struct with a message.
  `ManifestError` is an enum that callers can `match` on. Notice which one is
  easier to test against, then fix the other one in the refactor.

## 2. The game plan for every feature (keep this open)

**1. What goes in and what comes out?** Write it as a type signature before any
code: `fn parse_array(s: &str, line: usize) -> Result<Value, ParseError>`. Ask
yourself three things:

- _Shape:_ what goes in, and what comes out?
- _Ownership:_ does the caller give me the data (`String`), lend it (`&str`), or
  let me change it (`&mut`)?
- _Failure:_ can it be missing (`Option`) or wrong (`Result`)?

**2. Write the failing test first.** It's already there for each step. Write
extra tests for your helper functions too.

**3. Write the steps in plain English as comments,** then turn each comment into
code. If you can't write a step in English, the problem isn't Rust yet; you
haven't worked out the steps.

**4. Stub it out and let the compiler check your plan.** Put the signature in
with a `todo!()` body and run `cargo check`. If it compiles, your types fit
together.

**5. Look up methods based on the type you have** (your instinct here is right):

- `rustup doc --std` gives you offline docs. Open the page for the type you're
  holding (`str`, `Option`, `BTreeMap`) and read the list of method names in the
  sidebar.
- **rustdoc can search by type.** In the docs search box, type
  `str -> Option<(&str, &str)>` and it finds `split_once`. Type
  `Option<T> -> Result<T, E>` and it finds `ok_or`. Use this when you know the
  shape but not the name.
- Write the type you think something has (`let x: () = thing;`). The compiler
  error tells you the real type.

**6. Get it working ugly first.** Use `.clone()`, `unwrap()` in tests, long `if`
chains, or `dbg!(&x)` anywhere. Then clean it up while the tests are green.

**7. Run `cargo test`, then `cargo clippy`, then commit.** Small commits make it
cheap to experiment, since you can always go back.

## 3. When you're stuck (in this order, before asking anyone)

1. **Read only the first compiler error.** Later errors are often caused by the
   first one. For more detail, run `rustc --explain E0502`.
2. **Shrink the problem.** Put the smallest input that breaks into a `.toml`
   file and run `dump` on it. Or write a 3-line unit test that calls just the
   helper.
3. **Print what's actually there.** Put `dbg!()` around the value you think is
   wrong. Most bugs turn out to be "I assumed this was X but it was Y".
4. **Say it out loud.** Explain the problem in a comment as if to someone else.
   Half the time you'll find the answer before you finish.
5. **Look at code you already have.** Most steps have a similar function already
   in this crate (as_integer → as_float, parse_array → parse_inline_table).
6. **Time-box it to 25 minutes.** After that, open _one_ hint below, then try
   again.
7. **Only then ask,** and ask a specific question: "I have `&str`, I want
   `Vec<&str>` split on commas that aren't inside quotes; I tried X, it got Y."
   Writing that question out often solves it.

Being stuck isn't failing. Struggling with the problem is how it ends up
sticking.

## 4. The steps (with hints you reveal one at a time)

To start a step: delete one `#[ignore]` line in `tests/steps.rs` and run
`cargo test step1`.

### Step 1: `#` inside strings

<details><summary>Hint 1</summary>You need to remember whether you're inside quotes as you scan the line. That's one `bool`.</details>
<details><summary>Hint 2</summary><code>for (i, c) in line.char_indices()</code> gives you a byte index you can slice with.</details>
<details><summary>Hint 3</summary>When you see <code>'"'</code>, flip <code>in_string</code>. When you see <code>'#'</code> and <code>!in_string</code>, return <code>&line[..i]</code>. If you get to the end, return <code>line</code>. (Escaped quotes will break this. You'll fix that in step 3 or phase 2.)</details>

### Step 2: floats

<details><summary>Hint 1</summary>Same approach as integers: <code>str::parse::&lt;f64&gt;()</code>.</details>
<details><summary>Hint 2</summary>Order matters. If you try float first, <code>7</code> becomes <code>7.0</code>. Try integer first.</details>
<details><summary>Hint 3</summary>Watch out: Rust parses "inf" and "NaN" as floats, but TOML spells them <code>inf</code>/<code>nan</code>, all lowercase. Leave this alone for now and just note it.</details>

### Step 3: escapes and 'literal' strings

<details><summary>Hint 1</summary>Literal strings are easy: <code>strip_prefix('\'')</code>, then <code>strip_suffix('\'')</code>, with no escape handling.</details>
<details><summary>Hint 2</summary>For escapes, write <code>fn unescape(s: &str, line: usize) -> Result&lt;String, ParseError&gt;</code>. Loop over <code>s.chars()</code>. When you see <code>'\\'</code>, call <code>chars.next()</code> to get the next character and <code>match</code> on it.</details>
<details><summary>Hint 3</summary><code>strip_suffix('"')</code> isn't good enough anymore, because <code>"a\"</code> ends in a quote but isn't closed. Notice how this keeps pushing you toward scanning one character at a time. Keep that in mind for phase 2.</details>

### Step 4: arrays (the important one)

<details><summary>Hint 1</summary>Write <code>fn split_top_level(s: &str) -> Vec&lt;&str&gt;</code> by itself and unit-test it first. This is the real work. Then <code>parse_array</code> is just strip brackets → split → <code>parse_value</code> each piece → collect.</details>
<details><summary>Hint 2</summary>Keep track of <code>depth: i32</code> (++ on <code>[</code> or <code>{</code>, -- on <code>]</code> or <code>}</code>) and <code>in_string: bool</code>. A comma only splits when depth == 0 and you're not in a string. Remember where the last piece started (<code>start</code>).</details>
<details><summary>Hint 3</summary>Collecting results: <code>pieces.into_iter().map(|p| parse_value(p.trim(), line)).collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()?</code>. Collecting into <code>Result&lt;Vec&gt;</code> stops at the first error, and it's one of the most useful tricks in Rust. Skip empty pieces so trailing commas work.</details>

### Step 5: inline tables

<details><summary>Hint 1</summary>Reuse <code>split_top_level</code>. Each piece is <code>key = value</code>, so use <code>split_once('=')</code>.</details>
<details><summary>Hint 2</summary>Watch for <code>features = ["a"]</code>. Does <code>split_once('=')</code> pick the right <code>=</code>? (It takes the first one. Is that always the one you want?)</details>

### Step 6: dotted keys

<details><summary>Hint 1</summary><code>a.b.c = 1</code> means: path = current_path + ["a","b"], key = "c".</details>
<details><summary>Hint 2</summary><code>let mut parts = split_key(key); let last = parts.pop()</code>. Then <code>current_path.iter().cloned().chain(parts)</code> gives you the full path. Pass it to <code>get_or_create_table</code>.</details>

### Step 7: manifest details

<details><summary>Hint 1</summary>Write <code>optional_str_array(t, section, key) -> Result&lt;Vec&lt;String&gt;, ManifestError&gt;</code>. If the key is missing, return an empty Vec.</details>
<details><summary>Hint 2</summary>You'll need an <code>optional_bool</code> for <code>optional</code>. Then <code>.unwrap_or(false)</code>.</details>
<details><summary>Hint 3</summary>Add <code>dev_dependencies</code> to <code>Manifest</code>. The loop that reads <code>[dependencies]</code> can turn into a function you call twice.</details>

### Step 8: the wall

`fixtures/real_cargo.toml` has multi-line arrays and `[[bin]]`. A parser that
reads one line at a time **can't** handle them cleanly. That isn't a mistake in
your code; you've reached the limit of the approach. Welcome to phase 2.

## 5. Refactoring to idiomatic Rust (phase 2)

Do these one at a time, and keep the tests green the whole way.

- **R1, error kinds:** `ParseError { kind: ErrorKind, line, col }`, where
  `ErrorKind` is an enum. Tests can then
  `matches!(e.kind, ErrorKind::DuplicateKey(_))` instead of checking message
  text.
- **R2, standard traits:** `impl FromStr for Manifest`
  (`text.parse::<Manifest>()`), `impl TryFrom<&Table> for Package`,
  `impl Display for Value`.
- **R3, scan characters instead of lines:** a
  `struct Cursor<'a> { src: &'a str, chars: Peekable<CharIndices<'a>>, line: usize }`
  with `peek()`, `bump()`, `eat(c) -> bool`, `skip_whitespace()`. Then
  `parse_value(&mut self)` works by looking at the next character: `"` → string,
  `[` → array (which calls `parse_value` recursively and can span lines for
  free). Steps 1, 3, 4 and 8 are all much simpler this way. Read the step-4
  splitter next to this and you'll see why.
- **R4, `[[array.of.tables]]`:** an array of tables
  (`Value::Array(Vec<Value::Table>)`). `get_or_create_table` has to learn to
  step into "the last element".
- **R5 (optional), a serializer:** `Value` → TOML text. Round-trip test:
  `parse(to_string(parse(x))) == parse(x)`.
- **Stretch goals:** compare your code with the `toml` crate's
  `toml_edit`/`winnow` code after you've finished, never before.
