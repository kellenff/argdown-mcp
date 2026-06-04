//! Premise-conclusion structures (PCS): numbered statement lines, inference
//! lines, and interspersed relations, emitted as a flat sequence of form-tagged
//! items in source order. Role assignment, inference→conclusion binding, and
//! relation association are Layer B's job.

use argdown_core::{Pcs, PcsItem, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{digit1, line_ending};
use winnow::combinator::{alt, cut_err, delimited, eof, opt, peek, preceded};
use winnow::token::{take_till, take_while};

use crate::Input;
use crate::relation::relation;
use crate::statement::statement;
use crate::text::inline_ws;

/// Parse a PCS block: a leading numbered statement (which commits to a PCS),
/// then a run of items until a non-item line. The numbered marker `(n)` is what
/// distinguishes a PCS from a plain statement, so a line that is not a numbered
/// statement makes `pcs` backtrack and the dispatcher fall through.
pub(crate) fn pcs(input: &mut Input<'_>) -> ModalResult<Pcs> {
    let first = numbered_statement_item(input)?;
    let start = item_span_start(&first);
    let mut items = vec![first];
    while let Some(item) = opt(alt((
        numbered_statement_item,
        inference_item,
        relation_item,
    )))
    .parse_next(input)?
    {
        items.push(item);
    }
    let end = item_span_end(items.last().expect("pcs has at least one item"));
    Ok(Pcs {
        items,
        span: Span { start, end },
    })
}

/// `(n) <statement>` — the marker commits (so a bad/empty body is an error),
/// but a line that is not `( digits )` backtracks so dispatch can continue.
fn numbered_statement_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    inline_ws.parse_next(input)?;
    let (number, marker_span) = pcs_number.with_span().parse_next(input)?;
    inline_ws.parse_next(input)?;
    let stmt = cut_err(statement).parse_next(input)?;
    let span = Span {
        start: marker_span.start,
        end: stmt.span.end,
    };
    Ok(PcsItem::Statement {
        number,
        statement: stmt,
        span,
    })
}

/// `( digits )` → the numeric value. Backtracks if the line is not a numbered
/// marker (e.g. `(see note)`), so such lines fall through to a plain statement.
fn pcs_number(input: &mut Input<'_>) -> ModalResult<usize> {
    delimited('(', digit1, ')')
        .try_map(|s: &str| s.parse::<usize>())
        .parse_next(input)
}

/// A bare divider (`-{4,}`) or a ruled divider (`-- Rule, Rule --`). A line
/// starting with `--` commits to an inference line; a malformed one (e.g. `---`,
/// or a ruled opener with no closing `--`) is an error.
fn inference_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    inline_ws.parse_next(input)?;
    peek("--").parse_next(input)?;
    let (rules, span) = cut_err(inference_rules).with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok(PcsItem::Inference {
        rules,
        metadata: None,
        span: span.into(),
    })
}

/// Classify an inference line already known to start with `--`.
fn inference_rules(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    alt((bare_divider, ruled_divider)).parse_next(input)
}

/// `-{4,}` followed by only trailing whitespace → no rules.
fn bare_divider(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    (
        take_while(4.., '-'),
        inline_ws,
        peek(alt((line_ending.void(), eof.void()))),
    )
        .map(|_| Vec::new())
        .parse_next(input)
}

/// `-- <content> --` on a single line → content split on commas into trimmed
/// rule names. Content is bounded to the current line (`take_till` stops at a
/// line ending), so a malformed line with no closing `--` fails here and
/// `inference_item`'s `cut_err` turns it into a hard error — rather than scanning
/// ahead to a later divider on another line.
fn ruled_divider(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    preceded("--", take_till(0.., ['\r', '\n']))
        .verify_map(|rest: &str| {
            let inner = rest.trim_end().strip_suffix("--")?;
            Some(
                inner
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect(),
            )
        })
        .parse_next(input)
}

/// An interspersed relation line, reusing the relation parser.
fn relation_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    relation.map(PcsItem::Relation).parse_next(input)
}

fn item_span_start(item: &PcsItem) -> usize {
    match item {
        PcsItem::Statement { span, .. } | PcsItem::Inference { span, .. } => span.start,
        PcsItem::Relation(relation) => relation.span.start,
    }
}

fn item_span_end(item: &PcsItem) -> usize {
    match item {
        PcsItem::Statement { span, .. } | PcsItem::Inference { span, .. } => span.end,
        PcsItem::Relation(relation) => relation.span.end,
    }
}
