# Argdown Parser — A4 (Inline Elements) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recognize inline elements (bold, italic, link, statement/argument mentions, tags) inside statement text and argument descriptions, as a flat `Vec<Inline>` of typed source-byte spans overlaid on the source.

**Architecture:** Additive `inlines` field on `Statement`/`Argument` in `argdown-core`. A new `inline.rs` provides a plain recursive scanner `scan_line(line, base) -> Result<(Vec<Inline>, content_len), InlineError>` that recognizes elements (recursing into element content so nesting yields contained flat spans) and reports where a trailing `//` comment begins. The existing body readers (`definition_body`, `plain_statement`) call it per source line, collect the inlines, and turn an `InlineError` into a hard parse failure.

**Tech Stack:** Rust, `winnow` 1.x (`LocatingSlice` for byte spans), Cargo workspace.

**Spec:** `docs/snowball/specs/2026-06-02-argdown-parser-a4-inline-design.md`

**Conventions (follow exactly):**
- TDD: write the failing test, run it, watch it fail for the right reason, then minimal code to pass. Tests go through the public `parse()` in the existing `#[cfg(test)] mod tests` of `crates/argdown-parser/src/lib.rs`.
- Inline spans are **absolute source offsets**. `text`/`description` are unchanged.
- Recognition is per body line (elements do not cross line breaks); an unclosed recognized opener is an error.
- Run `cargo test -p argdown-parser` after each step; keep all prior tests green.

---

### Task 1: Add `inlines` AST field + update existing literals

Adds the field and the new types. Because this adds a field to `Statement`/`Argument`, **every existing literal must gain `inlines: vec![]`** or the crate won't compile. This task is done when the workspace compiles and all prior tests pass (still green, all inlines empty).

**Files:**
- Modify: `crates/argdown-core/src/ast.rs`
- Modify: `crates/argdown-core/src/lib.rs`
- Modify: `crates/argdown-parser/src/statement.rs`, `argument.rs` (literal construction)
- Modify: `crates/argdown-parser/src/lib.rs` (test literals)

- [ ] **Step 1: Add the inline types and fields**

In `crates/argdown-core/src/ast.rs`, add `inlines` to both structs and add the new types after `Argument`:

```rust
pub struct Statement {
    pub title: Option<String>,
    pub text: String,
    pub is_reference: bool,
    pub span: Span,
    pub inlines: Vec<Inline>,
}

pub struct Argument {
    pub title: String,
    pub description: String,
    pub is_reference: bool,
    pub span: Span,
    pub inlines: Vec<Inline>,
}
```

Add after the `Argument` struct (keep existing derives `#[derive(Debug, Clone, PartialEq, Eq)]` on each new item):

```rust
/// One inline element inside a statement/argument body. `span` is the full
/// source extent of the element (opening delimiter through closing delimiter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inline {
    pub kind: InlineKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineKind {
    Bold,
    Italic,
    Link { url: String },
    StatementMention { title: String },
    ArgumentMention { title: String },
    Tag { tag: String },
}
```

- [ ] **Step 2: Export the new types**

In `crates/argdown-core/src/lib.rs`, add `Inline, InlineKind` to the `pub use ast::{...}` list (keep alphabetical-ish ordering with the rest).

```rust
pub use ast::{
    Argument, Block, Document, Heading, Inline, InlineKind, Pcs, PcsItem, Relation,
    RelationDirection, RelationOperator, RelationTarget, Span, Statement,
};
```

- [ ] **Step 3: Fix the production literal sites**

In `crates/argdown-parser/src/statement.rs`, every `Statement { ... }` constructed adds `inlines: vec![]` (there are three: the definition branch, the reference branch in `bracketed_statement`, and `plain_statement`). Example for the reference branch:

```rust
        Ok(Statement {
            title: Some(title),
            text: String::new(),
            is_reference: true,
            span: span.into(),
            inlines: vec![],
        })
```

In `crates/argdown-parser/src/argument.rs`, both `Argument { ... }` literals (definition and reference) add `inlines: vec![]`.

- [ ] **Step 4: Fix all test literals**

In `crates/argdown-parser/src/lib.rs` and `crates/argdown-core/src/ast.rs`, every `Statement { ... }` / `Argument { ... }` literal in tests adds `inlines: vec![]`. Compile to find them all:

Run: `cargo build --workspace 2>&1 | rg "missing field|argdown-parser|argdown-core"`
Expected initially: errors `missing field 'inlines' in initializer of ...`. Add `inlines: vec![]` to each until the build is clean. (These are mechanical, compiler-listed edits.)

- [ ] **Step 5: Verify build + all prior tests pass**

Run: `cargo test --workspace`
Expected: all existing tests pass (PCS, relation, statement, argument, heading, etc.), now with empty `inlines`.

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-core/src crates/argdown-parser/src
git commit -m "feat: add inline AST field to statements and arguments (A4)"
```

---

### Task 2: Inline scanner + emphasis, integrated end to end

Build `scan_line` with the scan loop, escape handling, comment-end detection, and emphasis (bold/italic) recognition with nesting. Wire it into `plain_statement` and `definition_body` so every statement/argument body is scanned.

**Files:**
- Create: `crates/argdown-parser/src/inline.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (`mod inline;`)
- Modify: `crates/argdown-parser/src/text.rs` (`definition_body` collects inlines; shared body-line scan helper)
- Modify: `crates/argdown-parser/src/statement.rs` (`plain_statement` collects inlines)

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block in `crates/argdown-parser/src/lib.rs` (helper + emphasis cases):

```rust
    use argdown_core::{Inline, InlineKind};

    /// The single statement a source parses to, panicking otherwise.
    fn only_statement(src: &str) -> Statement {
        match parse(src).unwrap().blocks.as_slice() {
            [Block::Statement(s)] => s.clone(),
            other => panic!("{src:?} did not parse as a single statement: {other:?}"),
        }
    }

    #[test]
    fn inline_italic_and_bold_plain_statement() {
        let s = only_statement("this is *it* and **bold**");
        assert_eq!(
            s.inlines,
            vec![
                Inline { kind: InlineKind::Italic, span: Span { start: 8, end: 12 } },
                Inline { kind: InlineKind::Bold, span: Span { start: 17, end: 25 } },
            ]
        );
    }

    #[test]
    fn inline_underscore_emphasis() {
        let s = only_statement("_i_ and __b__");
        assert_eq!(s.inlines[0].kind, InlineKind::Italic);
        assert_eq!(s.inlines[1].kind, InlineKind::Bold);
    }

    #[test]
    fn inline_emphasis_nests_as_contained_spans() {
        let s = only_statement("**bold and *italic* inside**");
        // Bold first (source order by start), then the contained italic.
        assert_eq!(s.inlines[0].kind, InlineKind::Bold);
        assert_eq!(s.inlines[1].kind, InlineKind::Italic);
        let (b, i) = (s.inlines[0].span, s.inlines[1].span);
        assert!(b.start <= i.start && i.end <= b.end, "italic must be contained in bold");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser inline_italic inline_underscore inline_emphasis`
Expected: FAIL — `s.inlines` is empty (`scan_line` not wired yet).

- [ ] **Step 3: Create the scanner**

Create `crates/argdown-parser/src/inline.rs`:

```rust
//! Inline element recognition over a single body line. Produces a flat list of
//! typed source-span `Inline`s; nesting yields contained spans (a recognizer
//! recurses into its element's content). `text`/`description` are unchanged.

use argdown_core::{Inline, InlineKind, Span};

/// An inline recognition failure: a recognized opener that never closes.
pub(crate) struct InlineError;

/// Scan one body-line slice. `base` is the absolute source offset of `line`'s
/// first byte. Returns the inline elements (absolute spans) and the byte index
/// in `line` where content ends — the start of a trailing `//` comment, else
/// `line.len()`. Errors on unclosed recognized markup.
pub(crate) fn scan_line(line: &str, base: usize) -> Result<(Vec<Inline>, usize), InlineError> {
    let mut inlines = Vec::new();
    let end = scan_run(line, 0, line.len(), base, &mut inlines, true)?;
    Ok((inlines, end))
}

/// Scan `line[start..limit]`, pushing recognized inlines. `top` enables the
/// trailing `//`-comment stop (only at the outermost level, not inside an
/// element). Returns the byte index where scanning stopped.
fn scan_run(
    line: &str,
    start: usize,
    limit: usize,
    base: usize,
    out: &mut Vec<Inline>,
    top: bool,
) -> Result<usize, InlineError> {
    let mut i = start;
    while i < limit {
        let rest = &line[i..limit];
        if rest.starts_with('\\') {
            // Escape: consume the backslash and the next char literally.
            i += 1;
            if i < limit {
                i += char_len(line, i);
            }
            continue;
        }
        if top && rest.starts_with("//") {
            return Ok(i);
        }
        match recognize(line, i, limit, base, out)? {
            Some(consumed) => i += consumed,
            None => i += char_len(line, i),
        }
    }
    Ok(limit)
}

/// Try every element at `i`. On success pushes the element (and any nested
/// inlines) to `out` and returns the consumed byte length; `None` if `i` is not
/// an element opener.
fn recognize(
    line: &str,
    i: usize,
    limit: usize,
    base: usize,
    out: &mut Vec<Inline>,
) -> Result<Option<usize>, InlineError> {
    if let Some(n) = try_emphasis(line, i, limit, base, out)? {
        return Ok(Some(n));
    }
    Ok(None)
}

/// `**X**` / `__X__` (bold) or `*X*` / `_X_` (italic). Bold is tried first.
fn try_emphasis(
    line: &str,
    i: usize,
    limit: usize,
    base: usize,
    out: &mut Vec<Inline>,
) -> Result<Option<usize>, InlineError> {
    let bytes = line.as_bytes();
    let c = bytes[i];
    if c != b'*' && c != b'_' {
        return Ok(None);
    }
    let double = i + 1 < limit && bytes[i + 1] == c;
    let delim_len = if double { 2 } else { 1 };
    let open_end = i + delim_len;
    // Opener must be followed by a non-space, non-same-delimiter char.
    if open_end >= limit || is_space(bytes[open_end]) || bytes[open_end] == c {
        return Ok(None);
    }
    // For `_`, the char before the opener must not be alphanumeric (word guard).
    if c == b'_' && i > 0 && is_alnum(bytes[i - 1]) {
        return Ok(None);
    }
    // Find the matching closer: a same-delimiter run preceded by a non-space,
    // and (for `_`) not followed by an alphanumeric.
    let mut j = open_end;
    while j < limit {
        if bytes[j] == c
            && (!double || (j + 1 < limit && bytes[j + 1] == c))
            && !is_space(bytes[j - 1])
        {
            let after = j + delim_len;
            if c == b'_' && after < limit && is_alnum(bytes[after]) {
                j += 1;
                continue;
            }
            // Closer found at j..after. Emit this element, then recurse inner.
            let kind = if double { InlineKind::Bold } else { InlineKind::Italic };
            out.push(Inline {
                kind,
                span: Span { start: base + i, end: base + after },
            });
            scan_run(line, open_end, j, base, out, false)?;
            return Ok(Some(after - i));
        }
        j += char_len(line, j);
    }
    // Recognized opener with no closer on this line.
    Err(InlineError)
}

fn char_len(line: &str, i: usize) -> usize {
    line[i..].chars().next().map_or(1, char::len_utf8)
}

fn is_space(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn is_alnum(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}
```

- [ ] **Step 4: Register the module**

In `crates/argdown-parser/src/lib.rs`, add `mod inline;` in the module list (after `heading`, before `pcs`, alphabetical):

```rust
mod argument;
mod heading;
mod inline;
mod pcs;
mod relation;
mod statement;
mod text;
mod trivia;
```

- [ ] **Step 5: Collect inlines in `definition_body` (text.rs)**

In `crates/argdown-parser/src/text.rs`, add the import and a shared per-line scan helper, and change `definition_body` to return inlines. Add near the top:

```rust
use argdown_core::Inline;
use winnow::error::{ContextError, ErrMode};

use crate::inline::scan_line;
```

Add a helper that scans a body line and appends its inlines, using the content length for normalization:

```rust
/// Scan one raw body line (`text`, absolute start `base`); append its inlines to
/// `out` and return the comment-stripped content slice for normalization.
pub(crate) fn body_line<'s>(
    text: &'s str,
    base: usize,
    out: &mut Vec<Inline>,
) -> ModalResult<&'s str> {
    match scan_line(text, base) {
        Ok((mut inlines, content_len)) => {
            out.append(&mut inlines);
            Ok(&text[..content_len])
        }
        Err(_) => Err(ErrMode::Cut(ContextError::new())),
    }
}
```

Replace `definition_body` with a version that threads inlines:

```rust
/// Read a definition body: remainder of the current line plus continuation
/// lines. Returns the normalized text, the body's end offset, and the inlines.
pub(crate) fn definition_body(input: &mut Input<'_>) -> ModalResult<(String, usize, Vec<Inline>)> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);

    let mut inlines = Vec::new();
    let mut contents: Vec<&str> = Vec::new();
    contents.push(body_line(first, first_span.start, &mut inlines)?);
    for (line, span) in &rest {
        contents.push(body_line(line, span.start, &mut inlines)?);
    }
    let text = normalize_contents(contents);
    Ok((text, end, inlines))
}
```

Add a normalizer that trims/joins already-comment-stripped content (replacing the comment-stripping that `body_line` now owns). Make it `pub(crate)` so `plain_statement` (Step 7) can reuse it:

```rust
/// Trim each content slice, drop empties, join with a single space.
pub(crate) fn normalize_contents<'a>(contents: impl IntoIterator<Item = &'a str>) -> String {
    let mut parts: Vec<&'a str> = Vec::new();
    for c in contents {
        let trimmed = c.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
}
```

(Leave the existing `normalize_lines` in place — `heading.rs` and any other caller still use `strip_trailing_line_comment`.)

- [ ] **Step 6: Thread inlines through statement/argument definitions**

In `crates/argdown-parser/src/statement.rs`, the definition branch of `bracketed_statement` now receives inlines:

```rust
    if opt(':').parse_next(input)?.is_some() {
        let (text, end, inlines) = definition_body(input)?;
        Ok(Statement {
            title: Some(title),
            text,
            is_reference: false,
            span: Span { start: span.start, end },
            inlines,
        })
    } else {
```

In `crates/argdown-parser/src/argument.rs`, the definition branch:

```rust
    if opt(':').parse_next(input)?.is_some() {
        let (description, end, inlines) = definition_body(input)?;
        Ok(Argument {
            title,
            description,
            is_reference: false,
            span: Span { start: span.start, end },
            inlines,
        })
    } else {
```

- [ ] **Step 7: Collect inlines in `plain_statement` (statement.rs)**

`plain_statement` reads a first line + continuation `content_line`s. Replace its text build to also collect inlines via `body_line`. Update `statement.rs` imports to add `argdown_core::Inline` and `crate::text::{body_line, normalize_contents}` (keep `content_line`; drop `normalize_lines`/`definition_body` from the import only if the compiler flags them as unused), then:

```rust
fn plain_statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
    )
        .parse_next(input)?;
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);

    let mut inlines: Vec<Inline> = Vec::new();
    let mut contents: Vec<&str> = Vec::new();
    contents.push(body_line(first, first_span.start, &mut inlines)?);
    for (line, span) in &rest {
        contents.push(body_line(line, span.start, &mut inlines)?);
    }
    let text = normalize_contents(contents);
    Ok(Statement {
        title: None,
        text,
        is_reference: false,
        span: Span { start: first_span.start, end },
        inlines,
    })
}
```

`normalize_contents` and `body_line` are the `pub(crate)` helpers from `text.rs` (Step 5). Remove any now-unused imports (`normalize_lines`) from `statement.rs` if the compiler flags them.

- [ ] **Step 8: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — the three emphasis tests pass, and **all prior tests stay green** (existing statement/argument text unchanged; inlines empty where there's no markup).

- [ ] **Step 9: Commit**

```bash
git add crates/argdown-parser/src
git commit -m "feat: recognize inline emphasis with nesting (A4)"
```

---

### Task 3: Links

Add link recognition (`[text](url)`), including URLs containing `//` (the link is recognized before any comment stop because `recognize` runs first).

**Files:**
- Modify: `crates/argdown-parser/src/inline.rs`
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn inline_link_with_url() {
        let s = only_statement("see [the site](http://x.com) now");
        assert_eq!(s.inlines.len(), 1);
        match &s.inlines[0].kind {
            InlineKind::Link { url } => assert_eq!(url, "http://x.com"),
            other => panic!("expected a link, got {other:?}"),
        }
        // Span covers the whole `[the site](http://x.com)`.
        assert_eq!(s.inlines[0].span, Span { start: 4, end: 28 });
    }

    #[test]
    fn bracket_without_paren_is_literal() {
        let s = only_statement("note [1] applies");
        assert!(s.inlines.is_empty());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser inline_link bracket_without_paren`
Expected: FAIL — `[the site](...)` is not recognized (link arm missing); `inlines` empty for the link case.

- [ ] **Step 3: Add the link recognizer**

In `crates/argdown-parser/src/inline.rs`, add a link arm to `recognize` (before returning `None`):

```rust
    if let Some(n) = try_link(line, i, limit, base, out)? {
        return Ok(Some(n));
    }
```

Add the recognizer:

```rust
/// `[display](url)` — `display` may contain nested inlines; `url` is literal.
/// A `[...]` not immediately followed by `(` is not a link (returns `None`); a
/// `[...](` with no closing `)` is an error.
fn try_link(
    line: &str,
    i: usize,
    limit: usize,
    base: usize,
    out: &mut Vec<Inline>,
) -> Result<Option<usize>, InlineError> {
    if line.as_bytes()[i] != b'[' {
        return Ok(None);
    }
    let Some(close_bracket) = find_byte(line, i + 1, limit, b']') else {
        return Ok(None);
    };
    let paren_open = close_bracket + 1;
    if paren_open >= limit || line.as_bytes()[paren_open] != b'(' {
        return Ok(None);
    }
    let Some(close_paren) = find_byte(line, paren_open + 1, limit, b')') else {
        return Err(InlineError);
    };
    let url = line[paren_open + 1..close_paren].to_string();
    let end = close_paren + 1;
    out.push(Inline {
        kind: InlineKind::Link { url },
        span: Span { start: base + i, end: base + end },
    });
    // Nested inlines live in the display text between the brackets.
    scan_run(line, i + 1, close_bracket, base, out, false)?;
    Ok(Some(end - i))
}

/// First index of byte `b` in `line[from..limit]`, or `None`.
fn find_byte(line: &str, from: usize, limit: usize, b: u8) -> Option<usize> {
    line.as_bytes()[from..limit]
        .iter()
        .position(|&x| x == b)
        .map(|p| from + p)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — link recognized with its url; `[1]` stays literal; all prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-parser/src/inline.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: recognize inline links (A4)"
```

---

### Task 4: Mentions

Add statement-mention `@[Title]` and argument-mention `@<Title>`.

**Files:**
- Modify: `crates/argdown-parser/src/inline.rs`
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn inline_statement_mention() {
        let s = only_statement("recall @[Other Claim] here");
        match &s.inlines[0].kind {
            InlineKind::StatementMention { title } => assert_eq!(title, "Other Claim"),
            other => panic!("expected a statement mention, got {other:?}"),
        }
    }

    #[test]
    fn inline_argument_mention() {
        let s = only_statement("per @<Some Arg> there");
        match &s.inlines[0].kind {
            InlineKind::ArgumentMention { title } => assert_eq!(title, "Some Arg"),
            other => panic!("expected an argument mention, got {other:?}"),
        }
    }

    #[test]
    fn bare_at_is_literal() {
        let s = only_statement("email a@b.com please");
        assert!(s.inlines.is_empty());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser inline_statement_mention inline_argument_mention bare_at`
Expected: FAIL — `@[...]`/`@<...>` not recognized; `inlines` empty.

- [ ] **Step 3: Add the mention recognizer**

Add a mention arm to `recognize` (before `None`):

```rust
    if let Some(n) = try_mention(line, i, limit, base, out) {
        return Ok(Some(n));
    }
```

(Note: mentions never error — a lone `@` or `@[` with no closer is just literal, so this recognizer returns `Option`, not `Result`.) Add:

```rust
/// `@[Title]` (statement) or `@<Title>` (argument). A lone `@`, or `@[`/`@<`
/// with no closer, is literal (`None`).
fn try_mention(line: &str, i: usize, limit: usize, base: usize, out: &mut Vec<Inline>) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes[i] != b'@' || i + 1 >= limit {
        return None;
    }
    let (open, close) = match bytes[i + 1] {
        b'[' => (b'[', b']'),
        b'<' => (b'<', b'>'),
        _ => return None,
    };
    debug_assert_eq!(open, bytes[i + 1]);
    let close_idx = find_byte(line, i + 2, limit, close)?;
    let title = line[i + 2..close_idx].trim().to_string();
    let end = close_idx + 1;
    let kind = if open == b'[' {
        InlineKind::StatementMention { title }
    } else {
        InlineKind::ArgumentMention { title }
    };
    out.push(Inline {
        kind,
        span: Span { start: base + i, end: base + end },
    });
    Some(end - i)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — both mentions recognized with titles; `a@b.com` stays literal; prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-parser/src/inline.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: recognize inline statement and argument mentions (A4)"
```

---

### Task 5: Tags

Add tag `#tag` (contiguous) and `#(multi word)` (parenthesized).

**Files:**
- Modify: `crates/argdown-parser/src/inline.rs`
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn inline_contiguous_tag() {
        let s = only_statement("flagged #simple-tag here");
        match &s.inlines[0].kind {
            InlineKind::Tag { tag } => assert_eq!(tag, "simple-tag"),
            other => panic!("expected a tag, got {other:?}"),
        }
    }

    #[test]
    fn inline_parenthesized_tag() {
        let s = only_statement("flagged #(multi word) here");
        match &s.inlines[0].kind {
            InlineKind::Tag { tag } => assert_eq!(tag, "multi word"),
            other => panic!("expected a tag, got {other:?}"),
        }
    }

    #[test]
    fn bare_hash_is_literal() {
        let s = only_statement("rooms # and # are free");
        assert!(s.inlines.is_empty());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser inline_contiguous_tag inline_parenthesized_tag bare_hash`
Expected: FAIL — tags not recognized; `inlines` empty.

- [ ] **Step 3: Add the tag recognizer**

Add a tag arm to `recognize` (before `None`):

```rust
    if let Some(n) = try_tag(line, i, limit, base, out)? {
        return Ok(Some(n));
    }
```

Add:

```rust
/// `#tag` (contiguous `[A-Za-z0-9_-]`) or `#(multi word)`. A `#` not followed
/// by a tag char or `(` is literal (`None`); `#(` with no `)` is an error.
fn try_tag(
    line: &str,
    i: usize,
    limit: usize,
    base: usize,
    out: &mut Vec<Inline>,
) -> Result<Option<usize>, InlineError> {
    let bytes = line.as_bytes();
    if bytes[i] != b'#' || i + 1 >= limit {
        return Ok(None);
    }
    if bytes[i + 1] == b'(' {
        let Some(close) = find_byte(line, i + 2, limit, b')') else {
            return Err(InlineError);
        };
        let tag = line[i + 2..close].trim().to_string();
        let end = close + 1;
        out.push(Inline {
            kind: InlineKind::Tag { tag },
            span: Span { start: base + i, end: base + end },
        });
        return Ok(Some(end - i));
    }
    if !is_tag_char(bytes[i + 1]) {
        return Ok(None);
    }
    let mut j = i + 1;
    while j < limit && is_tag_char(bytes[j]) {
        j += 1;
    }
    let tag = line[i + 1..j].to_string();
    out.push(Inline {
        kind: InlineKind::Tag { tag },
        span: Span { start: base + i, end: base + j },
    });
    Ok(Some(j - i))
}

fn is_tag_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — both tag forms recognized; `# ` stays literal; prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-parser/src/inline.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: recognize inline tags (A4)"
```

---

### Task 6: Escaping, prose-stays-literal, and strict completion errors

Tests for the behaviors already implemented by the scanner: escaping, prose with stray delimiters staying literal, and unclosed recognized markup erroring.

**Files:**
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the tests**

```rust
    #[test]
    fn inline_escape_suppresses_emphasis() {
        let s = only_statement(r"this \*is not\* italic");
        assert!(s.inlines.is_empty());
    }

    #[test]
    fn prose_with_stray_delimiters_stays_literal() {
        for src in ["cost is 5 * 3 dollars", "use snake_case names", "item # 4 here"] {
            let s = only_statement(src);
            assert!(s.inlines.is_empty(), "{src:?} should have no inlines");
        }
    }

    #[test]
    fn unclosed_emphasis_is_an_error() {
        assert!(parse("this is **bold with no close").is_err());
    }

    #[test]
    fn link_without_closing_paren_is_an_error() {
        assert!(parse("see [text](http://x.com here").is_err());
    }

    #[test]
    fn parenthesized_tag_without_close_is_an_error() {
        assert!(parse("flagged #(multi word here").is_err());
    }
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p argdown-parser inline_escape prose_with_stray unclosed_emphasis link_without_closing parenthesized_tag_without`
Expected: PASS — these exercise behavior built in Tasks 2–5 (escape skips the next char; `*`/`_` flanking and word-guard keep prose literal; recognized-but-unclosed openers return `InlineError` → `ErrMode::Cut`). If `prose_with_stray_delimiters_stays_literal` fails on `"5 * 3"`, re-check the opener rule (a `*` followed by a space must not open).

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-parser/src/lib.rs
git commit -m "test: cover inline escaping, literal prose, and strict errors (A4)"
```

---

### Task 7: Argument descriptions, multi-line, and PCS reuse

Verify inline recognition flows through argument descriptions, multi-line bodies, and PCS numbered statements (all via the shared `definition_body`/`plain_statement`).

**Files:**
- Test: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Write the tests**

```rust
    #[test]
    fn inline_in_argument_description() {
        let blocks = parse("<A>: this has **bold** text").unwrap().blocks;
        match &blocks[0] {
            Block::Argument(a) => assert_eq!(a.inlines[0].kind, InlineKind::Bold),
            other => panic!("expected an argument, got {other:?}"),
        }
    }

    #[test]
    fn inline_in_pcs_numbered_statement() {
        let blocks = parse("(1) a claim with *emphasis*").unwrap().blocks;
        match &blocks[0] {
            Block::Pcs(p) => match &p.items[0] {
                PcsItem::Statement { statement, .. } => {
                    assert_eq!(statement.inlines[0].kind, InlineKind::Italic);
                }
                other => panic!("expected a statement item, got {other:?}"),
            },
            other => panic!("expected a PCS, got {other:?}"),
        }
    }

    #[test]
    fn inline_span_absolute_across_a_definition_title() {
        // `[T]: ` is 5 bytes, so the bold opener `**` starts at byte 5.
        let s = only_statement("[T]: **b**");
        assert_eq!(s.inlines[0].span, Span { start: 5, end: 10 });
    }
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p argdown-parser inline_in_argument inline_in_pcs inline_span_absolute`
Expected: PASS — argument descriptions and PCS statements reuse the same body readers, so inlines appear automatically; the absolute span starts after the `[T]: ` prefix.

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-parser/src/lib.rs
git commit -m "test: cover inline in argument descriptions and PCS statements (A4)"
```

---

### Task 8: Full verification

**Files:** none (verification only).

- [ ] **Step 1: Run the whole workspace test suite**

Run: `cargo test --workspace`
Expected: all prior tests (A1–A3) plus the new inline tests pass. 0 failures.

- [ ] **Step 2: Lint**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: exit 0, no warnings. (Ignore the unrelated `failed to auto-clean cache data` cargo message.)

- [ ] **Step 3: Format**

Run: `cargo fmt --all` then `cargo fmt --all -- --check`
Expected: `--check` exits 0 (clean).

- [ ] **Step 4: Commit any formatting changes**

```bash
git add -A -- crates/
git commit -m "chore: cargo fmt after A4 inline" || echo "nothing to format"
```

(Stage only `crates/` — leave `docs/snowball/decisions/` and ignored tooling dirs for a separate decisions commit at finish.)

---

## Done criteria (from the spec)

- `cargo test` passes, including the new inline tests and all prior tests.
- `parse()` populates `Statement.inlines` / `Argument.inlines` with correct kinds, data, and source spans, in source order, nesting by containment; prose with stray delimiters stays literal; unclosed recognized markup errors.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
- Deferred (do NOT implement here): display-text `clean_text` helper, containment/interval index, mention/tag resolution and aggregation (Layer B), `{…}` metadata (A5), inline in headings.
