//! Strongly connected component analysis for Dung AFs (Tarjan).

use crate::ArgumentId;
use super::ArgumentationFramework;

/// Metadata derived from structural analysis of an argumentation framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfMetadata {
    pub argument_count: usize,
    pub attack_count: usize,
    pub is_acyclic: bool,
    pub has_self_attacks: bool,
    pub strongly_connected_components: Vec<Vec<ArgumentId>>,
    pub isolated_arguments: Vec<ArgumentId>,
}

/// Analyze an AF: counts, acyclicity, self-attacks, SCCs, and isolated nodes.
pub fn analyze_af(af: &ArgumentationFramework) -> AfMetadata {
    let argument_count = af.arguments.len();
    let attack_count = af.attacks.len();
    let has_self_attacks = af
        .attacks
        .iter()
        .any(|&(from, to)| from == to);

    let strongly_connected_components = if argument_count == 0 {
        Vec::new()
    } else {
        tarjan_scc(af)
    };

    let is_acyclic = !has_self_attacks
        && strongly_connected_components
            .iter()
            .all(|scc| scc.len() == 1);

    let mut incident: std::collections::HashSet<ArgumentId> = std::collections::HashSet::new();
    for &(from, to) in &af.attacks {
        incident.insert(from);
        incident.insert(to);
    }
    let isolated_arguments = af
        .arguments
        .iter()
        .copied()
        .filter(|a| !incident.contains(a))
        .collect();

    AfMetadata {
        argument_count,
        attack_count,
        is_acyclic,
        has_self_attacks,
        strongly_connected_components,
        isolated_arguments,
    }
}

fn tarjan_scc(af: &ArgumentationFramework) -> Vec<Vec<ArgumentId>> {
    use std::collections::{HashMap, HashSet};

    let mut adjacency: HashMap<ArgumentId, Vec<ArgumentId>> = HashMap::new();
    for &arg in &af.arguments {
        adjacency.entry(arg).or_default();
    }
    for &(from, to) in &af.attacks {
        if adjacency.contains_key(&from) && adjacency.contains_key(&to) {
            adjacency.get_mut(&from).unwrap().push(to);
        }
    }

    struct Tarjan<'a> {
        adjacency: &'a HashMap<ArgumentId, Vec<ArgumentId>>,
        index: usize,
        stack: Vec<ArgumentId>,
        on_stack: HashSet<ArgumentId>,
        indices: HashMap<ArgumentId, usize>,
        lowlink: HashMap<ArgumentId, usize>,
        components: Vec<Vec<ArgumentId>>,
    }

    impl Tarjan<'_> {
        fn strongconnect(&mut self, v: ArgumentId) {
            self.indices.insert(v, self.index);
            self.lowlink.insert(v, self.index);
            self.index += 1;
            self.stack.push(v);
            self.on_stack.insert(v);

            for &w in self.adjacency.get(&v).into_iter().flat_map(|n| n.iter()) {
                if !self.indices.contains_key(&w) {
                    self.strongconnect(w);
                    let w_low = self.lowlink[&w];
                    let v_low = self.lowlink.get_mut(&v).unwrap();
                    *v_low = (*v_low).min(w_low);
                } else if self.on_stack.contains(&w) {
                    let w_idx = self.indices[&w];
                    let v_low = self.lowlink.get_mut(&v).unwrap();
                    *v_low = (*v_low).min(w_idx);
                }
            }

            if self.lowlink[&v] == self.indices[&v] {
                let mut component = Vec::new();
                loop {
                    let w = self.stack.pop().unwrap();
                    self.on_stack.remove(&w);
                    component.push(w);
                    if w == v {
                        break;
                    }
                }
                self.components.push(component);
            }
        }
    }

    let mut tarjan = Tarjan {
        adjacency: &adjacency,
        index: 0,
        stack: Vec::new(),
        on_stack: HashSet::new(),
        indices: HashMap::new(),
        lowlink: HashMap::new(),
        components: Vec::new(),
    };

    for &arg in &af.arguments {
        if !tarjan.indices.contains_key(&arg) {
            tarjan.strongconnect(arg);
        }
    }

    tarjan.components
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
    fn empty_af_is_acyclic_with_no_sccs() {
        let meta = analyze_af(&af(0, &[]));
        assert_eq!(meta.argument_count, 0);
        assert_eq!(meta.attack_count, 0);
        assert!(meta.is_acyclic);
        assert!(!meta.has_self_attacks);
        assert!(meta.strongly_connected_components.is_empty());
        assert!(meta.isolated_arguments.is_empty());
    }

    #[test]
    fn chain_is_acyclic_with_three_singleton_sccs() {
        let meta = analyze_af(&af(3, &[(0, 1), (1, 2)]));
        assert!(meta.is_acyclic);
        assert!(!meta.has_self_attacks);
        assert_eq!(meta.strongly_connected_components.len(), 3);
        for scc in &meta.strongly_connected_components {
            assert_eq!(scc.len(), 1);
        }
        assert!(meta.isolated_arguments.is_empty());
    }

    #[test]
    fn two_cycle_is_not_acyclic_with_one_scc() {
        let meta = analyze_af(&af(2, &[(0, 1), (1, 0)]));
        assert!(!meta.is_acyclic);
        assert!(!meta.has_self_attacks);
        assert_eq!(meta.strongly_connected_components, vec![vec![ArgumentId(1), ArgumentId(0)]]);
        assert!(meta.isolated_arguments.is_empty());
    }

    #[test]
    fn isolated_node_is_listed_when_no_edges_touch_it() {
        let meta = analyze_af(&af(3, &[(0, 1)]));
        assert_eq!(meta.isolated_arguments, vec![ArgumentId(2)]);
    }
}
