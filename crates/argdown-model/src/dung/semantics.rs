//! Dung semantics dispatch: grounded, complete, preferred, stable.

use super::ArgumentationFramework;
use super::propagate::{Label, Labeling, grounded_fixpoint};
use super::search::search_complete_labelings;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Which Dung-style extension semantics to compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Semantics {
    Grounded,
    Preferred,
    Stable,
    Complete,
}

/// Which solver path produced the result (for provenance / debugging).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    GroundedFixpoint,
    SccPropagationOnly,
    SccWithBacktracking,
    FilteredComplete,
}

/// Result of computing one semantics over an AF.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticsResult {
    pub semantics: Semantics,
    pub algorithm: Algorithm,
    pub labellings: Vec<Labeling>,
}

/// Compute all labellings (or the unique labelling) for `semantics` on `af`.
pub fn solve(af: &ArgumentationFramework, semantics: Semantics) -> SemanticsResult {
    match semantics {
        Semantics::Grounded => SemanticsResult {
            semantics,
            algorithm: Algorithm::GroundedFixpoint,
            labellings: vec![grounded_fixpoint(af)],
        },
        Semantics::Complete => {
            let (labellings, used_backtracking) = search_complete_labelings(af);
            SemanticsResult {
                semantics,
                algorithm: search_algorithm(used_backtracking),
                labellings,
            }
        }
        Semantics::Preferred => {
            let (complete, used_backtracking) = search_complete_labelings(af);
            let labellings = filter_preferred(complete);
            SemanticsResult {
                semantics,
                algorithm: search_algorithm(used_backtracking),
                labellings,
            }
        }
        Semantics::Stable => {
            let (complete, used_backtracking) = search_complete_labelings(af);
            let labellings = filter_stable(af, complete);
            let algorithm = if used_backtracking {
                Algorithm::FilteredComplete
            } else {
                Algorithm::SccPropagationOnly
            };
            SemanticsResult {
                semantics,
                algorithm,
                labellings,
            }
        }
    }
}

/// Keep labellings whose IN sets are subset-maximal among the input.
fn filter_preferred(labellings: Vec<Labeling>) -> Vec<Labeling> {
    let in_sets: Vec<HashSet<_>> = labellings.iter().map(in_set).collect();
    labellings
        .into_iter()
        .enumerate()
        .filter(|(i, _)| {
            let set_i = &in_sets[*i];
            !in_sets
                .iter()
                .enumerate()
                .any(|(j, set_j)| *i != j && set_i.is_subset(set_j) && set_i.len() < set_j.len())
        })
        .map(|(_, l)| l)
        .collect()
}

/// Keep complete labellings with no UNDEC (stable extensions).
fn filter_stable(af: &ArgumentationFramework, labellings: Vec<Labeling>) -> Vec<Labeling> {
    labellings
        .into_iter()
        .filter(|l| is_stable(af, l))
        .collect()
}

fn is_stable(af: &ArgumentationFramework, labeling: &Labeling) -> bool {
    if labeling.values().any(|&lab| lab == Label::Undec) {
        return false;
    }
    let mut attackers_of: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
    for &(from, to) in &af.attacks {
        attackers_of.entry(to).or_default().push(from);
    }
    af.arguments.iter().all(
        |&arg| match labeling.get(&arg).copied().unwrap_or(Label::Undec) {
            Label::Out => attackers_of
                .get(&arg)
                .map(|atts| {
                    atts.iter()
                        .any(|a| labeling.get(a).copied() == Some(Label::In))
                })
                .unwrap_or(false),
            _ => true,
        },
    )
}

fn search_algorithm(used_backtracking: bool) -> Algorithm {
    if used_backtracking {
        Algorithm::SccWithBacktracking
    } else {
        Algorithm::SccPropagationOnly
    }
}

fn in_set(labeling: &Labeling) -> HashSet<crate::ArgumentId> {
    labeling
        .iter()
        .filter_map(|(&id, &lab)| (lab == Label::In).then_some(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArgumentId;
    use crate::dung::grounded_extension;

    fn af(n: usize, attacks: &[(usize, usize)]) -> ArgumentationFramework {
        ArgumentationFramework {
            arguments: (0..n).map(ArgumentId).collect(),
            attacks: attacks
                .iter()
                .map(|&(f, t)| (ArgumentId(f), ArgumentId(t)))
                .collect(),
        }
    }

    fn in_args(labeling: &Labeling) -> Vec<ArgumentId> {
        let mut ids: Vec<_> = labeling
            .iter()
            .filter_map(|(&id, &lab)| (lab == Label::In).then_some(id))
            .collect();
        ids.sort_by_key(|a| a.0);
        ids
    }

    #[test]
    fn preferred_on_chain_has_one_labelling() {
        // a→b→c : preferred IN = {a,c}
        let af = af(3, &[(0, 1), (1, 2)]);
        let r = solve(&af, Semantics::Preferred);
        assert_eq!(r.algorithm, Algorithm::SccPropagationOnly);
        assert_eq!(
            in_args(&r.labellings[0]),
            vec![ArgumentId(0), ArgumentId(2)]
        );
    }

    #[test]
    fn stable_on_odd_cycle_is_empty() {
        // 3-cycle: no stable extension
        let af = af(3, &[(0, 1), (1, 2), (2, 0)]);
        let r = solve(&af, Semantics::Stable);
        assert!(r.labellings.is_empty());
    }

    #[test]
    fn grounded_via_solve_matches_grounded_extension() {
        let af = af(3, &[(0, 1), (1, 2)]);
        let r = solve(&af, Semantics::Grounded);
        assert_eq!(r.algorithm, Algorithm::GroundedFixpoint);
        let g = grounded_extension(&af);
        assert_eq!(in_args(&r.labellings[0]), g.in_);
    }

    #[test]
    fn complete_on_two_cycle_has_multiple() {
        let af = af(2, &[(0, 1), (1, 0)]);
        let r = solve(&af, Semantics::Complete);
        assert!(r.labellings.len() >= 2);
        assert_eq!(r.algorithm, Algorithm::SccWithBacktracking);
    }
}
