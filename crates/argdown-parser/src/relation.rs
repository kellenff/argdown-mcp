//! Relations: `+`, `<+`, `+>`, `-`, `<-`, `->`, `_`, `<_`, `_>`, `><` and their
//! targets. Emitted flat with raw indentation; assembling the parent/child tree
//! is Layer B's job.

use argdown_core::{Relation, RelationDirection, RelationOperator, RelationTarget, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{alt, cut_err, preceded};
use winnow::token::take_while;

use crate::Input;
use crate::argument::argument;
use crate::statement::statement;
use crate::text::inline_ws;

/// Parse one relation line: leading indent, an operator (which fixes both
/// operator and direction), then a target. The operator is backtrackable, so a
/// non-relation line (e.g. `<Arg>`) falls through to the argument parser; once
/// the operator matches, the target is required via `cut_err` (a missing target
/// is a hard parse error, not plain text).
pub(crate) fn relation(input: &mut Input<'_>) -> ModalResult<Relation> {
    let indent = take_while(0.., [' ', '\t'])
        .map(|ws: &str| ws.len())
        .parse_next(input)?;
    let ((operator, direction), op_span) = relation_operator.with_span().parse_next(input)?;
    let target = cut_err(preceded(inline_ws, relation_target)).parse_next(input)?;
    let end = match &target {
        RelationTarget::Statement(s) => s.span.end,
        RelationTarget::Argument(a) => a.span.end,
    };
    Ok(Relation {
        indent,
        operator,
        direction,
        target,
        span: Span {
            start: op_span.start,
            end,
        },
    })
}

/// Match a relation operator token, longest-first so a prefix never shadows a
/// longer token (`+>` before `+`, `<+` before any `+`-form). `+`≡`<+` (and the
/// `-`/`_` analogues) collapse to the same `(operator, direction)`.
fn relation_operator(input: &mut Input<'_>) -> ModalResult<(RelationOperator, RelationDirection)> {
    use RelationDirection::{Bidirectional, Inbound, Outbound};
    use RelationOperator::{Attack, Contradictory, Support, Undercut};
    // Two-char tokens are matched before one-char tokens so a prefix never
    // shadows a longer token (e.g. `+>` before `+`, `<+` before any `+`-form).
    alt((
        alt((
            "<+".value((Support, Inbound)),
            "<-".value((Attack, Inbound)),
            "<_".value((Undercut, Inbound)),
            "+>".value((Support, Outbound)),
            "->".value((Attack, Outbound)),
            "_>".value((Undercut, Outbound)),
            "><".value((Contradictory, Bidirectional)),
        )),
        alt((
            "+".value((Support, Inbound)),
            "-".value((Attack, Inbound)),
            "_".value((Undercut, Inbound)),
        )),
    ))
    .parse_next(input)
}

/// A relation's target reuses the statement and argument parsers verbatim, so
/// `+ [B]`, `+ [B]: x`, `+ plain`, `+ <Arg>`, and `+ <Arg>: desc` all parse —
/// including the strict text-after-reference error.
fn relation_target(input: &mut Input<'_>) -> ModalResult<RelationTarget> {
    alt((
        argument.map(RelationTarget::Argument),
        statement.map(RelationTarget::Statement),
    ))
    .parse_next(input)
}
