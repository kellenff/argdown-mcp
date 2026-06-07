//! Dung abstract argumentation framework + grounded extension (Layer B, B6b).
//!
//! The project's end goal — "Dung extensions in Rust". Two pure, total
//! functions over the [`Model`]:
//!
//! - [`dung_framework`] projects the Model into an abstract argumentation
//!   framework (AF): a **sharp** projection where nodes are arguments and the
//!   attack relation is exactly the `argument→argument` edges of kind `Attack`.
//!   Supports, undercuts, contradictories, and all statement-level attacks are
//!   excluded — matching `@argdown/core`'s `dung_extensions` (confirmed by
//!   probing: an argument-level `-` counts; the same attack at the statement
//!   level is ignored).
//! - [`grounded_extension`] computes the unique grounded labelling
//!   (IN / OUT / UNDEC) via the characteristic-function least-fixpoint.
//!
//! The AF is a derived *view* of the Model (for the solver), not document data,
//! so it lives outside the `Model` as standalone projection functions.

mod scc;

pub use scc::{AfMetadata, analyze_af};

use crate::{ArgumentId, Model, Node, RelationKind};
use std::collections::HashMap;

/// A Dung abstract argumentation framework: arguments + a binary attack
/// relation. A sharp projection of the [`Model`] (argument→argument `Attack`
/// edges only); see the module docs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArgumentationFramework {
    /// All Model arguments (named + anonymous), in arena/source order.
    pub arguments: Vec<ArgumentId>,
    /// Deduped directed attacks `(attacker, attacked)`, in first-occurrence
    /// (source) order.
    pub attacks: Vec<(ArgumentId, ArgumentId)>,
}

/// The unique grounded labelling. Each argument appears in exactly one list;
/// lists follow [`ArgumentationFramework::arguments`] order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroundedLabelling {
    /// Accepted: every attacker is `out` (vacuously true if unattacked).
    pub in_: Vec<ArgumentId>,
    /// Defeated: attacked by some `in` argument.
    pub out: Vec<ArgumentId>,
    /// Undecided: in cycles / self-attacks the fixpoint cannot resolve.
    pub undec: Vec<ArgumentId>,
}

/// Project the Model into a Dung AF — arguments as nodes, `argument→argument`
/// `Attack` edges as the (deduped) attack relation. Total.
pub fn dung_framework(model: &Model) -> ArgumentationFramework {
    let arguments: Vec<ArgumentId> = model.arguments.iter().map(|a| a.id).collect();

    let mut attacks: Vec<(ArgumentId, ArgumentId)> = Vec::new();
    for edge in &model.edges {
        if edge.kind != RelationKind::Attack {
            continue;
        }
        if let (Node::Argument(from), Node::Argument(to)) = (edge.from, edge.to) {
            let pair = (from, to);
            // Linear dedup — O(n²); fine at expected sizes. B5 already deduped
            // `edges`, so a duplicate here only arises from two relations
            // expressing the same arg→arg attack; pair with a `HashSet` if large
            // maps ever matter.
            if !attacks.contains(&pair) {
                attacks.push(pair);
            }
        }
    }

    ArgumentationFramework { arguments, attacks }
}

/// Compute the unique grounded labelling of an AF via the characteristic-
/// function least-fixpoint: an argument is IN once all its attackers are OUT
/// (vacuously, if unattacked), OUT once any attacker is IN; iterate to a
/// fixpoint, and whatever remains (cycles, self-attacks) is UNDEC. Total.
pub fn grounded_extension(af: &ArgumentationFramework) -> GroundedLabelling {
    /// 0 = undecided, 1 = in, 2 = out.
    const UNDEC: u8 = 0;
    const IN: u8 = 1;
    const OUT: u8 = 2;

    let mut attackers_of: HashMap<ArgumentId, Vec<ArgumentId>> = HashMap::new();
    for &(from, to) in &af.attacks {
        attackers_of.entry(to).or_default().push(from);
    }

    let mut state: HashMap<ArgumentId, u8> = af.arguments.iter().map(|&a| (a, UNDEC)).collect();

    loop {
        let mut changed = false;
        for &arg in &af.arguments {
            if state[&arg] != UNDEC {
                continue;
            }
            let attackers = attackers_of.get(&arg).map(Vec::as_slice).unwrap_or(&[]);
            // For an AF from `dung_framework` every attacker is in `state`;
            // `unwrap_or(UNDEC)` keeps the lookup total for a hand-built AF whose
            // attacks reference an argument not in `arguments` (a phantom
            // attacker counts as undecided).
            if attackers
                .iter()
                .all(|a| state.get(a).copied().unwrap_or(UNDEC) == OUT)
            {
                state.insert(arg, IN);
                changed = true;
            } else if attackers
                .iter()
                .any(|a| state.get(a).copied().unwrap_or(UNDEC) == IN)
            {
                state.insert(arg, OUT);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut labelling = GroundedLabelling::default();
    for &arg in &af.arguments {
        match state[&arg] {
            IN => labelling.in_.push(arg),
            OUT => labelling.out.push(arg),
            _ => labelling.undec.push(arg),
        }
    }
    labelling
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_model;
    use argdown_parser::parse;

    #[test]
    fn argument_level_attack_becomes_an_af_attack() {
        // `<a>` / `  - <b>` is an argument-level attack: b attacks a.
        let m = build_model(&parse("<a>: A\n  - <b>\n\n<b>: B").unwrap());
        let af = dung_framework(&m);
        assert_eq!(af.arguments, vec![ArgumentId(0), ArgumentId(1)]);
        assert_eq!(af.attacks, vec![(ArgumentId(1), ArgumentId(0))]);
    }

    #[test]
    fn empty_model_has_empty_framework() {
        let af = dung_framework(&build_model(&parse("").unwrap()));
        assert_eq!(af, ArgumentationFramework::default());
    }

    #[test]
    fn statement_level_attack_is_excluded() {
        // The same attack nested under a PCS premise is statement-level and is
        // ignored by the reference (probe-confirmed) — its edge has a Statement
        // endpoint, so the projection drops it.
        let m = build_model(&parse("<a>: A\n\n(1) p\n  - <b>\n----\n(2) c\n\n<b>: B").unwrap());
        let af = dung_framework(&m);
        assert_eq!(af.arguments.len(), 2);
        assert!(af.attacks.is_empty());
    }

    #[test]
    fn support_and_undercut_are_excluded() {
        let support = dung_framework(&build_model(&parse("<a>: A\n  + <b>\n\n<b>: B").unwrap()));
        assert!(support.attacks.is_empty());
        let undercut = dung_framework(&build_model(&parse("<a>: A\n  _ <b>\n\n<b>: B").unwrap()));
        assert!(undercut.attacks.is_empty());
    }

    #[test]
    fn the_same_attack_written_twice_is_one_edge() {
        // `<a> / - <b>` and `<b> / -> <a>` both mean b attacks a.
        let m = build_model(&parse("<a>: A\n  - <b>\n\n<b>: B\n  -> <a>").unwrap());
        let af = dung_framework(&m);
        assert_eq!(af.attacks, vec![(ArgumentId(1), ArgumentId(0))]);
    }

    #[test]
    fn anonymous_argument_is_an_af_node() {
        // A statement detaches the PCS, which becomes an anonymous argument.
        let m = build_model(&parse("<a>: A\n\n[s]: s\n\n(1) p\n----\n(2) c").unwrap());
        let af = dung_framework(&m);
        assert_eq!(af.arguments, vec![ArgumentId(0), ArgumentId(1)]);
        assert_eq!(m.arguments[1].title, None); // the anonymous argument
    }

    // ── grounded_extension (directly-constructed AFs) ───────────────────────

    fn af(n: usize, attacks: &[(usize, usize)]) -> ArgumentationFramework {
        ArgumentationFramework {
            arguments: (0..n).map(ArgumentId).collect(),
            attacks: attacks
                .iter()
                .map(|&(f, t)| (ArgumentId(f), ArgumentId(t)))
                .collect(),
        }
    }

    #[test]
    fn an_unattacked_argument_is_in() {
        let g = grounded_extension(&af(1, &[]));
        assert_eq!(g.in_, vec![ArgumentId(0)]);
        assert!(g.out.is_empty() && g.undec.is_empty());
    }

    #[test]
    fn an_attacker_defeats_its_target() {
        // 0 attacks 1.
        let g = grounded_extension(&af(2, &[(0, 1)]));
        assert_eq!(g.in_, vec![ArgumentId(0)]);
        assert_eq!(g.out, vec![ArgumentId(1)]);
    }

    #[test]
    fn an_attack_chain_reinstates_the_grandchild() {
        // 0 -> 1 -> 2: 0 in, 1 out, 2 reinstated (in).
        let g = grounded_extension(&af(3, &[(0, 1), (1, 2)]));
        assert_eq!(g.in_, vec![ArgumentId(0), ArgumentId(2)]);
        assert_eq!(g.out, vec![ArgumentId(1)]);
        assert!(g.undec.is_empty());
    }

    #[test]
    fn a_self_attacker_is_undecided() {
        let g = grounded_extension(&af(1, &[(0, 0)]));
        assert_eq!(g.undec, vec![ArgumentId(0)]);
        assert!(g.in_.is_empty() && g.out.is_empty());
    }

    #[test]
    fn a_two_cycle_is_undecided() {
        let g = grounded_extension(&af(2, &[(0, 1), (1, 0)]));
        assert_eq!(g.undec, vec![ArgumentId(0), ArgumentId(1)]);
        assert!(g.in_.is_empty() && g.out.is_empty());
    }

    #[test]
    fn odd_and_even_cycles_are_undecided() {
        let three = grounded_extension(&af(3, &[(0, 1), (1, 2), (2, 0)]));
        assert_eq!(three.undec.len(), 3);
        let four = grounded_extension(&af(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]));
        assert_eq!(four.undec.len(), 4);
        assert!(three.in_.is_empty() && four.in_.is_empty());
    }

    #[test]
    fn an_argument_with_one_in_and_one_undec_attacker_is_out() {
        // 0 unattacked → in; 2 self-attacks → undec; 1 is attacked by both 0
        // (in) and 2 (undec), so the "any attacker in" rule wins → out.
        let g = grounded_extension(&af(3, &[(0, 1), (2, 1), (2, 2)]));
        assert_eq!(g.in_, vec![ArgumentId(0)]);
        assert_eq!(g.out, vec![ArgumentId(1)]);
        assert_eq!(g.undec, vec![ArgumentId(2)]);
    }

    #[test]
    fn an_attack_from_a_non_listed_argument_is_total_and_undecided() {
        // A hand-built AF whose attacker is not in `arguments` (phantom): the
        // function must not panic and treats the phantom as undecided.
        let af = ArgumentationFramework {
            arguments: vec![ArgumentId(0)],
            attacks: vec![(ArgumentId(9), ArgumentId(0))],
        };
        let g = grounded_extension(&af);
        assert_eq!(g.undec, vec![ArgumentId(0)]);
        assert!(g.in_.is_empty() && g.out.is_empty());
    }

    #[test]
    fn end_to_end_matches_the_reference() {
        // `<a> / - <b> / <b>` → reference grounded: {b in, a out}.
        let m = build_model(&parse("<a>: A\n  - <b>\n\n<b>: B").unwrap());
        let g = grounded_extension(&dung_framework(&m));
        assert_eq!(g.in_, vec![ArgumentId(1)]); // b
        assert_eq!(g.out, vec![ArgumentId(0)]); // a
        assert!(g.undec.is_empty());
    }
}
