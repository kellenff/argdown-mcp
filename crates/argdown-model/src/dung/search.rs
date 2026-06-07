//! SCC-decomposition + backtracking search for complete labellings.

use super::propagate::{empty_labeling, grounded_fixpoint, propagate_once, Label, Labeling};
use super::scc::analyze_af;
use super::ArgumentationFramework;
use crate::ArgumentId;
use std::collections::HashMap;

/// Enumerate all complete labellings.
///
/// Returns `(labellings, used_backtracking)` where `used_backtracking` is
/// `false` on acyclic AFs (single propagation-only result).
pub fn search_complete_labelings(af: &ArgumentationFramework) -> (Vec<Labeling>, bool) {
    let meta = analyze_af(af);
    if meta.is_acyclic {
        return (vec![grounded_fixpoint(af)], false);
    }

    let cyclic_args = cyclic_scc_arguments(af, &meta);
    let mut results = Vec::new();
    let mut labeling = empty_labeling(&af.arguments);
    backtrack(af, &cyclic_args, 0, &mut labeling, &mut results);
    (results, true)
}

/// Arguments belonging to SCCs that are not acyclic singletons without self-attacks.
fn cyclic_scc_arguments(
    af: &ArgumentationFramework,
    meta: &super::scc::AfMetadata,
) -> Vec<ArgumentId> {
    let mut args = Vec::new();
    for scc in &meta.strongly_connected_components {
        if is_cyclic_scc(scc, af) {
            args.extend_from_slice(scc);
        }
    }
    args.sort_by_key(|a| a.0);
    args.dedup();
    args
}

fn is_cyclic_scc(scc: &[ArgumentId], af: &ArgumentationFramework) -> bool {
    if scc.len() > 1 {
        return true;
    }
    if let [a] = scc {
        return af.attacks.iter().any(|&(from, to)| from == *a && to == *a);
    }
    false
}

fn backtrack(
    af: &ArgumentationFramework,
    cyclic_args: &[ArgumentId],
    idx: usize,
    labeling: &mut Labeling,
    results: &mut Vec<Labeling>,
) {
    if idx == cyclic_args.len() {
        while propagate_once(af, labeling) {}
        if is_complete(af, labeling) {
            results.push(labeling.clone());
        }
        return;
    }

    let arg = cyclic_args[idx];
    for label in [Label::In, Label::Out, Label::Undec] {
        labeling.insert(arg, label);

        if !is_partially_consistent(af, arg, labeling) {
            labeling.insert(arg, Label::Undec);
            continue;
        }

        // Stable fail-fast: OUT requires a defeating IN attacker.
        if label == Label::Out && !can_be_defeated(af, arg, labeling) {
            labeling.insert(arg, Label::Undec);
            continue;
        }

        backtrack(af, cyclic_args, idx + 1, labeling, results);
        labeling.insert(arg, Label::Undec);
    }
}

fn build_attackers(af: &ArgumentationFramework) -> HashMap<ArgumentId, Vec<ArgumentId>> {
    let mut attackers_of: HashMap<ArgumentId, Vec<ArgumentId>> = HashMap::new();
    for &(from, to) in &af.attacks {
        attackers_of.entry(to).or_default().push(from);
    }
    attackers_of
}

fn is_complete(af: &ArgumentationFramework, labeling: &Labeling) -> bool {
    let attackers_of = build_attackers(af);
    af.arguments.iter().all(|&arg| {
        let lab = labeling.get(&arg).copied().unwrap_or(Label::Undec);
        let attackers = attackers_of.get(&arg).map(Vec::as_slice).unwrap_or(&[]);
        match lab {
            Label::In => attackers
                .iter()
                .all(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::Out),
            Label::Out => attackers
                .iter()
                .any(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::In),
            Label::Undec => {
                !attackers.is_empty()
                    && !attackers
                        .iter()
                        .all(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::Out)
                    && !attackers
                        .iter()
                        .any(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::In)
            }
        }
    })
}

fn is_partially_consistent(af: &ArgumentationFramework, arg: ArgumentId, labeling: &Labeling) -> bool {
    let attackers_of = build_attackers(af);
    let lab = labeling.get(&arg).copied().unwrap_or(Label::Undec);
    let attackers = attackers_of.get(&arg).map(Vec::as_slice).unwrap_or(&[]);
    match lab {
        Label::In => attackers
            .iter()
            .all(|a| {
                let al = labeling.get(a).copied().unwrap_or(Label::Undec);
                al == Label::Out || al == Label::Undec
            })
            && !attackers
                .iter()
                .any(|a| labeling.get(a).copied() == Some(Label::In)),
        Label::Out => attackers
            .iter()
            .any(|a| labeling.get(a).copied() == Some(Label::In)),
        Label::Undec => !attackers
            .iter()
            .all(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::Out)
            && !attackers
                .iter()
                .any(|a| labeling.get(a).copied() == Some(Label::In)),
    }
}

/// An OUT argument must be attacked by some IN argument (now or after search).
fn can_be_defeated(af: &ArgumentationFramework, arg: ArgumentId, labeling: &Labeling) -> bool {
    let attackers_of = build_attackers(af);
    let attackers = attackers_of.get(&arg).map(Vec::as_slice).unwrap_or(&[]);
    attackers.iter().any(|a| {
        matches!(
            labeling.get(a).copied().unwrap_or(Label::Undec),
            Label::In
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn acyclic_returns_single_labelling() {
        let framework = af(3, &[(0, 1), (1, 2)]);
        let (labellings, used_backtracking) = search_complete_labelings(&framework);
        assert!(!used_backtracking);
        assert_eq!(labellings.len(), 1);
    }

    #[test]
    fn two_cycle_has_multiple_complete() {
        let framework = af(2, &[(0, 1), (1, 0)]);
        let (labellings, used_backtracking) = search_complete_labelings(&framework);
        assert!(used_backtracking);
        assert!(labellings.len() >= 2);
    }
}
