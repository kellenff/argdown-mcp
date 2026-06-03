//! Premise-conclusion structures (PCS): numbered statement lines, inference
//! lines, and interspersed relations, emitted as a flat sequence of form-tagged
//! items in source order. Role assignment, inference→conclusion binding, and
//! relation association are Layer B's job.

use argdown_core::{Pcs, PcsItem, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::digit1;
use winnow::combinator::{cut_err, delimited, opt};

use crate::Input;
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
    while let Some(item) = opt(numbered_statement_item).parse_next(input)? {
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
