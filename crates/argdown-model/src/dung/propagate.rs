//! Shared label propagation for Dung AF semantics.

use crate::ArgumentId;
use super::ArgumentationFramework;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    In,
    Out,
    Undec,
}

pub type Labeling = HashMap<ArgumentId, Label>;

/// Initialise every argument in `args` to [`Label::Undec`].
pub fn empty_labeling(args: &[ArgumentId]) -> Labeling {
    args.iter().map(|&a| (a, Label::Undec)).collect()
}

/// One propagation pass: an undec argument becomes IN when all attackers are
/// OUT, or OUT when any attacker is IN. Returns `true` if any label changed.
pub fn propagate_once(af: &ArgumentationFramework, labeling: &mut Labeling) -> bool {
    let mut attackers_of: HashMap<ArgumentId, Vec<ArgumentId>> = HashMap::new();
    for &(from, to) in &af.attacks {
        attackers_of.entry(to).or_default().push(from);
    }

    let mut changed = false;
    for &arg in &af.arguments {
        if labeling.get(&arg).copied().unwrap_or(Label::Undec) != Label::Undec {
            continue;
        }
        let attackers = attackers_of.get(&arg).map(Vec::as_slice).unwrap_or(&[]);
        if attackers
            .iter()
            .all(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::Out)
        {
            labeling.insert(arg, Label::In);
            changed = true;
        } else if attackers
            .iter()
            .any(|a| labeling.get(a).copied().unwrap_or(Label::Undec) == Label::In)
        {
            labeling.insert(arg, Label::Out);
            changed = true;
        }
    }
    changed
}

/// Run propagation to a fixpoint; returns the final labelling.
pub fn grounded_fixpoint(af: &ArgumentationFramework) -> Labeling {
    let mut labeling = empty_labeling(&af.arguments);
    while propagate_once(af, &mut labeling) {}
    labeling
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dung::ArgumentationFramework;

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
    fn unattacked_argument_is_in() {
        let framework = af(1, &[]);
        let labeling = grounded_fixpoint(&framework);
        assert_eq!(labeling.get(&ArgumentId(0)), Some(&Label::In));
    }

    #[test]
    fn attacker_in_target_out() {
        let framework = af(2, &[(0, 1)]);
        let labeling = grounded_fixpoint(&framework);
        assert_eq!(labeling.get(&ArgumentId(0)), Some(&Label::In));
        assert_eq!(labeling.get(&ArgumentId(1)), Some(&Label::Out));
    }

    #[test]
    fn two_cycle_both_undec() {
        let framework = af(2, &[(0, 1), (1, 0)]);
        let labeling = grounded_fixpoint(&framework);
        assert_eq!(labeling.get(&ArgumentId(0)), Some(&Label::Undec));
        assert_eq!(labeling.get(&ArgumentId(1)), Some(&Label::Undec));
    }

    #[test]
    fn self_attack_is_undec() {
        let framework = af(1, &[(0, 0)]);
        let labeling = grounded_fixpoint(&framework);
        assert_eq!(labeling.get(&ArgumentId(0)), Some(&Label::Undec));
    }
}
