//! Grounded extension via shared fixpoint propagation.

use super::{ArgumentationFramework, GroundedLabelling};
use super::propagate::{grounded_fixpoint, Label};

/// Compute the unique grounded labelling of an AF via the characteristic-
/// function least-fixpoint. Total.
pub fn grounded_extension(af: &ArgumentationFramework) -> GroundedLabelling {
    let labeling = grounded_fixpoint(af);

    let mut labelling = GroundedLabelling::default();
    for &arg in &af.arguments {
        match labeling.get(&arg).copied().unwrap_or(Label::Undec) {
            Label::In => labelling.in_.push(arg),
            Label::Out => labelling.out.push(arg),
            Label::Undec => labelling.undec.push(arg),
        }
    }
    labelling
}
