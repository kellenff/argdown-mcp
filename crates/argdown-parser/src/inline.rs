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
