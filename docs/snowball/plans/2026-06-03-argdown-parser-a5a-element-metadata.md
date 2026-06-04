# Argdown Parser — A5a (Element Metadata) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recognize trailing `{yaml}` metadata blocks on statements, argument descriptions, headings, and inference lines, capturing each as `Metadata { raw, span }` stripped from the element's text.

**Architecture:** Additive `Metadata` type + `metadata: Option<Metadata>` field on `Statement`/`Argument`/`Heading`/`PcsItem::Inference`. A4's per-line inline scanner (`scan_line`) is extended to report a top-level unescaped `{` (the metadata opener — it already skips inline-element interiors). A new `metadata.rs` provides `capture_metadata`, a balanced, quote/escape-aware, multi-line `{…}` scanner. The shared body readers split the text from a trailing metadata block (with a brace-aware body extent for multi-line blocks) and enforce a strict text-after-metadata rule. No YAML parse, no `serde_yaml` dependency.

**Tech Stack:** Rust, `winnow` 1.x (`LocatingSlice` for byte spans), Cargo workspace.

**Spec:** `docs/snowball/specs/2026-06-03-argdown-parser-a5a-element-metadata-design.md`

**Conventions (follow exactly):**
- TDD: failing test → run/watch it fail → minimal code → pass. Tests go through the public `parse()` in the existing `#[cfg(test)] mod tests` of `crates/argdown-parser/src/lib.rs`.
- `metadata.span` covers the whole `{…}` block (braces included); `metadata.raw` is the inner content verbatim (between `{` and `}`, not trimmed).
- The `{` is reserved: an unescaped top-level `{` opens metadata; `\{` is literal. Text after the closing `}` (other than trivia) is a hard error.
- Run `cargo fmt --all` then `cargo test -p argdown-parser` and `cargo clippy --workspace --all-targets -- -D warnings` before each commit; stage ONLY `crates/`.

---

### Task 1: Add `Metadata` AST type + fields + literal churn

Pure additive declarations. No metadata parsing yet — the field is always `None` after this task; all prior tests stay green.

**Files:**
- Modify: `crates/argdown-core/src/ast.rs`, `crates/argdown-core/src/lib.rs`
- Modify: `crates/argdown-parser/src/statement.rs`, `argument.rs`, `heading.rs`, `pcs.rs` (literal construction)
- Modify: `crates/argdown-parser/src/lib.rs` (test literals)

- [ ] **Step 1: Add the `Metadata` type and fields**

In `crates/argdown-core/src/ast.rs`, add `metadata: Option<Metadata>` to `Statement`, `Argument`, and `Heading` (as the last field on each). Add `metadata: Option<Metadata>` to the `PcsItem::Inference` variant. Add the new type (after `Heading`, with the file's standard derives):

```rust
/// A trailing `{yaml}` metadata block: raw inner content + source span. The
/// content is not YAML-parsed here (a Layer-B utility does that).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// Inner content, verbatim (between `{` and `}`).
    pub raw: String,
    /// The whole `{…}` block, source range.
    pub span: Span,
}
```

The `PcsItem::Inference` variant becomes:

```rust
    /// `----` (bare → empty rules) or `-- Rule, Rule --` (ruled).
    Inference {
        rules: Vec<String>,
        metadata: Option<Metadata>,
        span: Span,
    },
```

- [ ] **Step 2: Export the type**

In `crates/argdown-core/src/lib.rs`, add `Metadata` to the `pub use ast::{...}` list (keep the alphabetical-ish ordering).

- [ ] **Step 3: Fix production literals**

Add `metadata: None` to every `Statement`/`Argument`/`Heading` literal in `crates/argdown-parser/src/statement.rs` (3 sites), `argument.rs` (2 sites), `heading.rs` (1 site). In `crates/argdown-parser/src/pcs.rs`, the `inference_item` constructs `PcsItem::Inference { rules, span }` — change it to `PcsItem::Inference { rules, metadata: None, span }`.

- [ ] **Step 4: Fix all test literals**

Run: `cargo build --workspace 2>&1 | rg "missing field"`
Expected: errors `missing field 'metadata' ...` across `crates/argdown-parser/src/lib.rs` tests (every `Statement`/`Heading`/`Argument` literal, and any `PcsItem::Inference` literal). Add `metadata: None` to each until the build is clean.

- [ ] **Step 5: Verify + commit**

Run: `cargo test --workspace` (all prior tests pass), `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.

```bash
git add crates/
git commit -m "feat: add Metadata AST type and fields (A5a)"
```

---

### Task 2: Balanced metadata scanner (`metadata.rs`)

A standalone scanner that captures one balanced `{…}` block. Pure function — exercised end-to-end in Task 3, but built and unit-tested here in isolation.

**Files:**
- Create: `crates/argdown-parser/src/metadata.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (`mod metadata;`)

- [ ] **Step 1: Write the failing unit tests**

Add a test module at the bottom of the new file (Step 3 creates the file; write tests first conceptually, but since this is a pure function, put the tests inside `metadata.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_single_line_block() {
        let src = "{k: v}";
        let m = capture_metadata(src, 100, 0).unwrap();
        assert_eq!(m.raw, "k: v");
        assert_eq!(m.span, argdown_core::Span { start: 100, end: 106 });
    }

    #[test]
    fn captures_nested_and_quoted_braces() {
        assert_eq!(capture_metadata("{a: {b: 1}}", 0, 0).unwrap().raw, "a: {b: 1}");
        assert_eq!(capture_metadata("{n: \"a } b\"}", 0, 0).unwrap().raw, "n: \"a } b\"");
    }

    #[test]
    fn captures_multi_line_block() {
        let src = "{\n  a: b\n  c: d\n}";
        let m = capture_metadata(src, 0, 0).unwrap();
        assert_eq!(m.raw, "\n  a: b\n  c: d\n");
        assert_eq!(m.span.end, src.len());
    }

    #[test]
    fn unterminated_block_errors() {
        assert!(capture_metadata("{a: b", 0, 0).is_err());
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p argdown-parser captures_single_line_block` (after Step 3 wires the module)
Expected: until Step 3, compilation fails (module/function missing). That is the RED state.

- [ ] **Step 3: Create the scanner**

Create `crates/argdown-parser/src/metadata.rs`:

```rust
//! Trailing `{yaml}` metadata block recognition. Captures the raw inner content
//! and source span of a balanced `{…}` block; the YAML is not parsed here.

use argdown_core::{Metadata, Span};

/// A metadata recognition failure: an unterminated `{` block.
pub(crate) struct MetaError;

/// Capture the balanced `{…}` block in `src` that starts at byte index `open`
/// (`src[open]` must be `{`). `base` is the absolute source offset of `src[0]`.
/// Brace depth is tracked while skipping over quoted strings (`"…"`, `'…'`), so
/// braces inside quotes don't miscount; the block may span multiple lines.
/// Returns the metadata (`raw` = inner content verbatim, `span` = the whole
/// block) or `MetaError` if there is no matching `}` before the end of `src`.
pub(crate) fn capture_metadata(src: &str, base: usize, open: usize) -> Result<Metadata, MetaError> {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[open], b'{');
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut i = open;
    while i < src.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q == b'"' {
                    i += 2; // escaped char inside a double-quoted string
                    continue;
                }
                if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Metadata {
                            raw: src[open + 1..i].to_string(),
                            span: Span {
                                start: base + open,
                                end: base + i + 1,
                            },
                        });
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    Err(MetaError)
}
```

- [ ] **Step 4: Register the module + run tests**

In `crates/argdown-parser/src/lib.rs`, add `mod metadata;` (alphabetical, after `inline`).
Run: `cargo test -p argdown-parser captures_ unterminated_block_errors`
Expected: PASS (all four scanner tests).

- [ ] **Step 5: Commit**

Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.

```bash
git add crates/
git commit -m "feat: balanced metadata block scanner (A5a)"
```

---

### Task 3: Extend the inline scan to report a metadata opener; capture metadata in statement/argument bodies (single line)

Make `scan_line` report the first top-level unescaped `{`, and have the shared body readers capture a single-line trailing metadata block, stripping it from the text.

**Files:**
- Modify: `crates/argdown-parser/src/inline.rs` (report the opener)
- Modify: `crates/argdown-parser/src/text.rs` (`body_line` + `definition_body` capture metadata)
- Modify: `crates/argdown-parser/src/statement.rs`, `argument.rs` (thread the metadata out)
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block in `crates/argdown-parser/src/lib.rs`:

```rust
    use argdown_core::Metadata;

    #[test]
    fn statement_definition_metadata() {
        let s = only_statement("[S]: claim text {certainty: 0.8}");
        assert_eq!(s.text, "claim text");
        let m = s.metadata.expect("metadata");
        assert_eq!(m.raw, "certainty: 0.8");
    }

    #[test]
    fn argument_definition_metadata() {
        let blocks = parse("<A>: a description {author: x}").unwrap().blocks;
        match &blocks[0] {
            Block::Argument(a) => {
                assert_eq!(a.description, "a description");
                assert_eq!(a.metadata.as_ref().unwrap().raw, "author: x");
            }
            other => panic!("expected an argument, got {other:?}"),
        }
    }

    #[test]
    fn metadata_coexists_with_inline() {
        let s = only_statement("[S]: a **bold** claim {k: v}");
        assert_eq!(s.text, "a **bold** claim");
        assert_eq!(s.inlines.len(), 1);
        assert_eq!(s.metadata.unwrap().raw, "k: v");
    }
```

(`only_statement` already exists in the test module from A4.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p argdown-parser statement_definition_metadata argument_definition_metadata metadata_coexists`
Expected: FAIL — `metadata` is `None` and the `{…}` is still part of `text`.

- [ ] **Step 3: Report the metadata opener from `scan_line`**

In `crates/argdown-parser/src/inline.rs`, change `scan_run` to stop at a top-level unescaped `{` and have `scan_line` report its position. Replace `scan_line` and the top-level `{` handling in `scan_run`:

```rust
/// Scan one body-line slice. `base` is the absolute source offset of `line`'s
/// first byte. Returns the inline elements, the byte index where content ends
/// (a trailing `//` comment or a top-level metadata `{`, else `line.len()`),
/// and `Some(index)` if a top-level unescaped `{` (a metadata opener) was found.
pub(crate) fn scan_line(
    line: &str,
    base: usize,
) -> Result<(Vec<Inline>, usize, Option<usize>), InlineError> {
    let mut inlines = Vec::new();
    let mut meta_open: Option<usize> = None;
    let end = scan_run(line, 0, line.len(), base, &mut inlines, true, &mut meta_open)?;
    Ok((inlines, end, meta_open))
}
```

Add the `meta_open` out-param to `scan_run` and stop at a top-level `{`:

```rust
fn scan_run(
    line: &str,
    start: usize,
    limit: usize,
    base: usize,
    out: &mut Vec<Inline>,
    top: bool,
    meta_open: &mut Option<usize>,
) -> Result<usize, InlineError> {
    let mut i = start;
    while i < limit {
        let rest = &line[i..limit];
        if rest.starts_with('\\') {
            i += 1;
            if i < limit {
                i += char_len(line, i);
            }
            continue;
        }
        if top && rest.starts_with("//") {
            return Ok(i);
        }
        if top && line.as_bytes()[i] == b'{' {
            *meta_open = Some(i);
            return Ok(i);
        }
        match recognize(line, i, limit, base, out, meta_open)? {
            Some(consumed) => i += consumed,
            None => i += char_len(line, i),
        }
    }
    Ok(limit)
}
```

Thread `meta_open` through `recognize` and the recursive `scan_run` calls in `try_emphasis`/`try_link` (a `{` inside an element is NOT top-level, so those recursive calls pass `top = false` and the same `meta_open` ref). Update `recognize`'s signature to take `meta_open: &mut Option<usize>` and pass it to each `try_*` that recurses; the recursive `scan_run(line, open_end, j, base, out, false, meta_open)` and `scan_run(line, i + 1, close_bracket, base, out, false, meta_open)` calls add the `false, meta_open` arguments.

- [ ] **Step 4: Capture metadata in `body_line` and `definition_body`**

In `crates/argdown-parser/src/text.rs`, update imports:

```rust
use argdown_core::{Inline, Metadata};
use crate::inline::scan_line;
use crate::metadata::capture_metadata;
```

`scan_line` now returns a triple. Update `body_line` to surface the metadata opener and capture the block when present. Because a single-line block is fully on this line, capture it here:

```rust
/// Scan one raw body line (`text`, absolute start `base`); append inlines to
/// `out`. Returns the content slice (for normalization) and, if the line opened
/// a single-line metadata block, the captured `Metadata`.
pub(crate) fn body_line<'s>(
    text: &'s str,
    base: usize,
    out: &mut Vec<Inline>,
    meta: &mut Option<Metadata>,
) -> ModalResult<&'s str> {
    match scan_line(text, base) {
        Ok((mut inlines, content_len, meta_open)) => {
            out.append(&mut inlines);
            if let Some(open) = meta_open {
                match capture_metadata(text, base, open) {
                    Ok(m) => {
                        let end_in_line = m.span.end - base;
                        // Only trivia may follow the closing `}` on this line.
                        let tail = text[end_in_line..].trim_start();
                        if !(tail.is_empty() || tail.starts_with("//")) {
                            return Err(ErrMode::Cut(ContextError::new()));
                        }
                        *meta = Some(m);
                    }
                    Err(_) => return Err(ErrMode::Cut(ContextError::new())),
                }
            }
            Ok(&text[..content_len])
        }
        Err(_) => Err(ErrMode::Cut(ContextError::new())),
    }
}
```

Update `definition_body` to thread the metadata and return it:

```rust
pub(crate) fn definition_body(
    input: &mut Input<'_>,
) -> ModalResult<(String, usize, Vec<Inline>, Option<Metadata>)> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);

    let mut inlines = Vec::new();
    let mut metadata: Option<Metadata> = None;
    let mut contents: Vec<&str> = Vec::new();
    contents.push(body_line(first, first_span.start, &mut inlines, &mut metadata)?);
    for (line, span) in &rest {
        contents.push(body_line(line, span.start, &mut inlines, &mut metadata)?);
    }
    let text = normalize_contents(contents);
    Ok((text, end, inlines, metadata))
}
```

- [ ] **Step 5: Thread metadata through statement/argument definitions**

In `crates/argdown-parser/src/statement.rs`, the definition branch of `bracketed_statement`:

```rust
        let (text, end, inlines, metadata) = definition_body(input)?;
        Ok(Statement {
            title: Some(title),
            text,
            is_reference: false,
            span: Span { start: span.start, end },
            inlines,
            metadata,
        })
```

`plain_statement` also calls the body readers — update its `body_line` calls to pass a `&mut Option<Metadata>` and set `metadata` on the returned `Statement` (declare `let mut metadata: Option<Metadata> = None;` alongside `inlines`, thread it through the `body_line` calls, and add `metadata` to the `Statement { … }` literal). Add `use argdown_core::Metadata;` to the imports if needed.

In `crates/argdown-parser/src/argument.rs`, the definition branch:

```rust
        let (description, end, inlines, metadata) = definition_body(input)?;
        Ok(Argument {
            title,
            description,
            is_reference: false,
            span: Span { start: span.start, end },
            inlines,
            metadata,
        })
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p argdown-parser`
Expected: the three new tests pass; all prior tests stay green (no metadata in existing inputs → `None`; existing text unchanged).

- [ ] **Step 7: Commit**

Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.

```bash
git add crates/
git commit -m "feat: capture single-line element metadata on statements and arguments (A5a)"
```

---

### Task 4: Multi-line metadata blocks

A metadata block may span lines; the body must extend across it. Make the continuation-line loop brace-aware so it keeps reading lines while inside an open `{…}` block, then capture the (now multi-line) block.

**Files:**
- Modify: `crates/argdown-parser/src/text.rs`
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn multi_line_metadata_block() {
        let s = only_statement("[S]: text {\n  a: b\n  c: d\n}");
        assert_eq!(s.text, "text");
        let raw = s.metadata.unwrap().raw;
        assert!(raw.contains("a: b") && raw.contains("c: d"), "raw was {raw:?}");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p argdown-parser multi_line_metadata_block`
Expected: FAIL — `content_line` stops at the blank/odd lines inside the braces, so the block is truncated and `capture_metadata` errors (unterminated) → parse error, or the wrong text is captured.

- [ ] **Step 3: Make the body extent brace-aware**

In `crates/argdown-parser/src/text.rs`, add a helper that counts the net brace delta of a raw line outside quotes (reusing the same quote rules as the scanner), and a brace-aware continuation reader. Replace the `rest` collection in `definition_body` (and the equivalent in `plain_statement`, Task 3 Step 5) with a loop that keeps consuming raw lines while a metadata block is open:

```rust
/// Net `{` minus `}` in `line`, ignoring braces inside quotes and after `\`.
fn brace_delta(line: &str) -> isize {
    let bytes = line.as_bytes();
    let mut delta = 0isize;
    let mut quote: Option<u8> = None;
    let mut i = 0;
    while i < line.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q == b'"' {
                    i += 2;
                    continue;
                }
                if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'\\' => {
                    i += 2;
                    continue;
                }
                b'"' | b'\'' => quote = Some(b),
                b'{' => delta += 1,
                b'}' => delta -= 1,
                _ => {}
            },
        }
        i += 1;
    }
    delta
}

/// Read continuation lines: normal `content_line`s, plus any raw lines needed to
/// close an open metadata `{…}` block (tracked by cumulative brace depth).
fn body_continuation<'s>(
    input: &mut Input<'s>,
    open_depth: isize,
) -> ModalResult<Vec<(&'s str, Range<usize>)>> {
    let mut depth = open_depth;
    let mut lines: Vec<(&str, Range<usize>)> = Vec::new();
    loop {
        if depth > 0 {
            // Inside a metadata block: consume the next raw line unconditionally.
            if eof::<_, ContextError>.parse_peek(*input).is_ok() {
                return Err(ErrMode::Cut(ContextError::new())); // unterminated block
            }
            let (line, span) = till_line_ending.with_span().parse_next(input)?;
            opt(line_ending).parse_next(input)?;
            depth += brace_delta(line);
            lines.push((line, span));
        } else {
            match opt(content_line).parse_next(input)? {
                Some((line, span)) => {
                    depth += brace_delta(line);
                    lines.push((line, span));
                }
                None => return Ok(lines),
            }
        }
    }
}
```

Update `definition_body` to use it (compute the first line's delta, then read continuations brace-aware), and capture the metadata from the *joined* body source rather than per-line. Replace the body-collection part of `definition_body`:

```rust
pub(crate) fn definition_body(
    input: &mut Input<'_>,
) -> ModalResult<(String, usize, Vec<Inline>, Option<Metadata>)> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest = body_continuation(input, brace_delta(first))?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);

    let lines: Vec<(&str, Range<usize>)> =
        std::iter::once((first, first_span.start..first_span.end))
            .chain(rest.iter().cloned())
            .collect();
    let (text, inlines, metadata) = process_body(&lines)?;
    Ok((text, end, inlines, metadata))
}
```

Add `process_body`, which scans line by line for inlines + the metadata opener, then captures the (possibly multi-line) block from the contiguous source. Because lines are contiguous in source, the block from the opener spans into later lines; reconstruct the source slice from the line spans:

```rust
/// Inline-scan each line and locate a top-level metadata `{`. When found, the
/// block is captured from the contiguous body source (it may run into later
/// lines). Returns the normalized text (pre-metadata), inlines, and metadata.
fn process_body(lines: &[(&str, Range<usize>)]) -> ModalResult<(String, Vec<Inline>, Option<Metadata>)> {
    let mut inlines = Vec::new();
    let mut contents: Vec<&str> = Vec::new();
    let mut metadata: Option<Metadata> = None;
    for (idx, (line, span)) in lines.iter().enumerate() {
        match scan_line(line, span.start) {
            Ok((mut found, content_len, meta_open)) => {
                inlines.append(&mut found);
                contents.push(&line[..content_len]);
                if let Some(open) = meta_open {
                    // Reconstruct the contiguous source from this line's `{`
                    // through the last body line, and capture the block.
                    let block_start = span.start + open;
                    let last_end = lines.last().unwrap().1.end;
                    let block_src = reconstruct(lines, idx, open, last_end);
                    let m = capture_metadata(&block_src, block_start, 0)
                        .map_err(|_| ErrMode::<ContextError>::Cut(ContextError::new()))?;
                    // Trivia-only after the closing `}`.
                    let after = (m.span.end - block_start) as usize;
                    if block_src[after..].split("//").next().unwrap().trim().is_empty() {
                        metadata = Some(m);
                        break;
                    }
                    return Err(ErrMode::Cut(ContextError::new()));
                }
            }
            Err(_) => return Err(ErrMode::Cut(ContextError::new())),
        }
    }
    Ok((normalize_contents(contents), inlines, metadata))
}

/// Join body lines from `(start_line, start_col)` to the last line's end into a
/// single owned string matching the original source (lines re-joined with `\n`).
fn reconstruct(lines: &[(&str, Range<usize>)], start_line: usize, start_col: usize, _last_end: usize) -> String {
    let mut out = String::new();
    out.push_str(&lines[start_line].0[start_col..]);
    for (line, _) in &lines[start_line + 1..] {
        out.push('\n');
        out.push_str(line);
    }
    out
}
```

(`capture_metadata`'s `base` is `block_start`, and `open` is `0` because `block_src` begins at the `{`. The `raw`/`span` are therefore correct absolute values.)

Apply the same `body_continuation` + `process_body` change to `plain_statement` in `statement.rs` (it reads `first` + continuation lines the same way). Remove the now-unused single-line metadata capture from `body_line` (Task 3) — `process_body` owns metadata capture now; `body_line` reverts to returning just the content slice and appending inlines (drop its `meta` parameter), OR delete `body_line` and inline its logic into `process_body`. Keep one path; the compiler will flag the dead one.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p argdown-parser`
Expected: the multi-line test passes; the Task 3 single-line tests still pass (single line is the one-line case of `process_body`); all prior tests green.

- [ ] **Step 5: Commit**

Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.

```bash
git add crates/
git commit -m "feat: support multi-line metadata blocks (A5a)"
```

---

### Task 5: Heading metadata

**Files:**
- Modify: `crates/argdown-parser/src/heading.rs`
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn heading_metadata() {
        let blocks = parse("# Title {k: v}").unwrap().blocks;
        match &blocks[0] {
            Block::Heading(h) => {
                assert_eq!(h.text, "Title");
                assert_eq!(h.metadata.as_ref().unwrap().raw, "k: v");
            }
            other => panic!("expected a heading, got {other:?}"),
        }
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p argdown-parser heading_metadata`
Expected: FAIL — heading `text` is `"Title {k: v}"` and `metadata` is `None`.

- [ ] **Step 3: Capture metadata in `heading`**

In `crates/argdown-parser/src/heading.rs`, after reading the heading text line, split a trailing single-line metadata block. Headings are single-line, so reuse `scan_line` + `capture_metadata` on the heading text. Add imports `use argdown_core::Metadata; use crate::inline::scan_line; use crate::metadata::capture_metadata;` and, after capturing the raw heading text `raw` with absolute start `text_start`:

```rust
    // (raw: &str is the heading text after the `#`s and spaces; text_start its
    //  absolute byte offset.)
    let (_inlines, content_len, meta_open) =
        scan_line(raw, text_start).map_err(|_| winnow::error::ErrMode::Cut(winnow::error::ContextError::new()))?;
    let metadata = match meta_open {
        Some(open) => Some(
            capture_metadata(raw, text_start, open)
                .map_err(|_| winnow::error::ErrMode::Cut(winnow::error::ContextError::new()))?,
        ),
        None => None,
    };
    let text = strip_trailing_line_comment(&raw[..content_len]).trim().to_string();
```

Set `metadata` on the returned `Heading { … }`. (Adjust the existing `heading` function to compute `text_start` — the byte offset where the heading text begins — via `.with_span()` on the text portion. Note: A5a does not surface heading inlines, so `_inlines` is discarded; only the metadata-opener position and content length are used.)

- [ ] **Step 4: Run + commit**

Run: `cargo test -p argdown-parser` (heading test passes, no regressions). Then `cargo fmt --all`, clippy.

```bash
git add crates/
git commit -m "feat: capture heading metadata (A5a)"
```

---

### Task 6: Inference-line metadata (fix the A3 limitation)

Split the trailing `{…}` from the inference rule names so `-- Modus Ponens {uses:[1,2]} --` yields clean rules + metadata.

**Files:**
- Modify: `crates/argdown-parser/src/pcs.rs`
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn inference_metadata_splits_from_rules() {
        let pcs = only_pcs("(1) p\n-- Modus Ponens {uses: [1,2]} --\n(2) q");
        match &pcs.items[1] {
            PcsItem::Inference { rules, metadata, .. } => {
                assert_eq!(rules, &vec!["Modus Ponens".to_string()]);
                assert_eq!(metadata.as_ref().unwrap().raw, "uses: [1,2]");
            }
            other => panic!("expected an inference item, got {other:?}"),
        }
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p argdown-parser inference_metadata_splits_from_rules`
Expected: FAIL — A3's `ruled_divider` captures `"Modus Ponens {uses: [1,2]}"` as a single rule name; `metadata` is `None`.

- [ ] **Step 3: Split metadata in the inference parser**

In `crates/argdown-parser/src/pcs.rs`, add `use argdown_core::Metadata; use crate::metadata::capture_metadata;`. Change `inference_rules` to also return an optional metadata block, and update `inference_item` to thread it. The ruled content is on one line; find a top-level `{` in it and split:

```rust
fn inference_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    inline_ws.parse_next(input)?;
    peek("--").parse_next(input)?;
    let ((rules, metadata), span) = cut_err(inference_rules).with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok(PcsItem::Inference {
        rules,
        metadata,
        span: span.into(),
    })
}

fn inference_rules(input: &mut Input<'_>) -> ModalResult<(Vec<String>, Option<Metadata>)> {
    alt((
        bare_divider.map(|rules| (rules, None)),
        ruled_divider,
    ))
    .parse_next(input)
}

/// `-{4,}` bare divider → no rules, no metadata.
fn bare_divider(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    (
        take_while(4.., '-'),
        inline_ws,
        peek(alt((line_ending.void(), eof.void()))),
    )
        .map(|_| Vec::new())
        .parse_next(input)
}

/// `-- <names> {metadata}? --` on a single line. Splits a trailing top-level
/// `{…}` block out of the content; the remainder is comma-split rule names.
fn ruled_divider(input: &mut Input<'_>) -> ModalResult<(Vec<String>, Option<Metadata>)> {
    let (content, content_span) = preceded("--", take_till(0.., ['\r', '\n']))
        .with_span()
        .parse_next(input)?;
    let inner = match content.trim_end().strip_suffix("--") {
        Some(inner) => inner,
        None => return Err(ErrMode::Cut(ContextError::new())),
    };
    let names_part = inner;
    // A top-level `{` in `inner` opens metadata; split there.
    let (names_str, metadata) = match find_top_level_brace(names_part) {
        Some(open) => {
            // Absolute offset of `inner[0]`: content starts after the opening
            // `--`, i.e. content_span.start + 2.
            let base = content_span.start + 2;
            let m = capture_metadata(names_part, base, open)
                .map_err(|_| ErrMode::<ContextError>::Cut(ContextError::new()))?;
            (&names_part[..open], Some(m))
        }
        None => (names_part, None),
    };
    let rules = names_str
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect();
    Ok((rules, metadata))
}

/// Byte index of the first unescaped `{` in `s`, or `None`.
fn find_top_level_brace(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'{' => return Some(i),
            _ => i += 1,
        }
    }
    None
}
```

Add the imports `ErrMode`, `ContextError` (from `winnow::error`) to `pcs.rs` if not present.

- [ ] **Step 4: Run + commit**

Run: `cargo test -p argdown-parser` (inference test passes; the existing A3 inference tests — `pcs_ruled_inference_single_rule`, `pcs_multi_step_interleaved` — still pass, since they have no `{…}`). Then `cargo fmt --all`, clippy.

```bash
git add crates/
git commit -m "feat: split inference-line metadata from rule names (A5a)"
```

---

### Task 7: Strict errors, escaping, references, and plain statements

Cover the strict text-after-metadata and unterminated errors, the `\{` escape, metadata on references and plain statements.

**Files:**
- Test: `crates/argdown-parser/src/lib.rs` (these exercise behavior built in Tasks 3–4; investigate, don't hack, if any fails)

- [ ] **Step 1: Write the tests**

```rust
    #[test]
    fn text_after_metadata_is_an_error() {
        assert!(parse("[S]: the set {a} more text").is_err());
    }

    #[test]
    fn unterminated_metadata_is_an_error() {
        assert!(parse("[S]: text {a: b").is_err());
    }

    #[test]
    fn escaped_brace_is_literal() {
        let s = only_statement(r"[S]: a \{ literal brace");
        assert!(s.metadata.is_none());
        assert!(s.text.contains('{'));
    }

    #[test]
    fn plain_statement_metadata() {
        let s = only_statement("a plain claim {k: v}");
        assert_eq!(s.text, "a plain claim");
        assert_eq!(s.metadata.unwrap().raw, "k: v");
    }

    #[test]
    fn reference_with_metadata() {
        let s = only_statement("[T] {k: v}");
        assert!(s.is_reference);
        assert_eq!(s.metadata.unwrap().raw, "k: v");
    }
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p argdown-parser text_after_metadata unterminated_metadata escaped_brace plain_statement_metadata reference_with_metadata`
Expected: `text_after_metadata`, `unterminated_metadata`, `escaped_brace`, `plain_statement_metadata` should PASS from Tasks 3–4. `reference_with_metadata` may FAIL — a reference (`[T]`) goes through `finish_reference`, not `definition_body`, so reference metadata is not yet captured.

- [ ] **Step 3: Capture metadata after a reference (if `reference_with_metadata` failed)**

If references need metadata, the reference branch in `bracketed_statement`/`argument` (which calls `finish_reference`) must check for a trailing `{…}` before requiring end-of-line. The minimal change: in `finish_reference` (text.rs), after `inline_ws`, optionally capture a metadata block from the rest of the line and return it, OR — simpler and consistent — have the reference branches, after `inline_ws`, attempt `scan_line` + `capture_metadata` on the remainder of the line and attach it.

Implement the simplest version that makes the test pass without weakening the existing "text after a reference is an error" guard: a `{…}` is allowed after a reference (it is metadata), but other text is still an error. Add a `finish_reference`-with-metadata path that returns `Option<Metadata>`; wire the statement/argument reference branches to set `metadata` from it. (If you find this expands scope too far, mark the task DONE_WITH_CONCERNS and propose deferring reference metadata to a follow-up, since the spec lists it as a single bullet.)

- [ ] **Step 4: Run + commit**

Run: `cargo test -p argdown-parser` (all pass). Then `cargo fmt --all`, clippy.

```bash
git add crates/
git commit -m "test: cover metadata errors, escaping, references, plain statements (A5a)"
```

---

### Task 8: Full verification

**Files:** none (verification only).

- [ ] **Step 1:** `cargo test --workspace` — all prior (A1–A4) tests plus new metadata tests pass, 0 failures.
- [ ] **Step 2:** `cargo clippy --workspace --all-targets -- -D warnings` — exit 0 (ignore the unrelated `failed to auto-clean cache data` message).
- [ ] **Step 3:** `cargo fmt --all` then `cargo fmt --all -- --check` — clean.
- [ ] **Step 4:** Commit any formatting:

```bash
git add -A -- crates/
git commit -m "chore: cargo fmt after A5a metadata" || echo "nothing to format"
```

---

## Done criteria (from the spec)

- `cargo test` passes, including new metadata tests and all prior tests.
- `parse()` populates `metadata` on statements, arguments, headings, and inference lines with the correct raw content and source span, stripped from the element text; inference rule names are clean; strict text-after and unterminated-block errors fire; `\{` is literal.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
- Deferred (do NOT implement): YAML parsing, the `parse_metadata` utility + `serde_yaml` dependency, `tags`-key promotion (Layer B), document frontmatter (A5b).
