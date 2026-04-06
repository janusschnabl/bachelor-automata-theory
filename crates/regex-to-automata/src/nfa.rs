use petgraph::algo::is_isomorphic_matching;
use petgraph::graph::Graph;
use std::collections::HashMap;

use crate::{EpsilonNfa, State, Symbol};

//PLEEAAASE
pub struct Nfa {
    pub states: Vec<State>,
    pub start: usize,
    pub accept: Vec<usize>,
}

impl EpsilonNfa {
    pub fn eclose(&self, state: usize) -> Vec<usize> {
        let mut visited = vec![false; self.states.len()];
        let mut stack = vec![state];
        visited[state] = true;

        while let Some(s) = stack.pop() {
            for (sym, target) in &self.states[s].transitions {
                if *sym == Symbol::Epsilon && !visited[*target] {
                    visited[*target] = true;
                    stack.push(*target);
                }
            }
        }

        visited
            .into_iter()
            .enumerate()
            .filter_map(|(i, v)| if v { Some(i) } else { None })
            .collect()
    }

    pub fn to_nfa(&self) -> Nfa {
        // STEP 1: Find the Epsilon Closure
        let ecloses: Vec<Vec<usize>> = (0..self.states.len()).map(|s| self.eclose(s)).collect();

        // STEP 2: Discover reachable states only
        let mut reachable: Vec<bool> = vec![false; self.states.len()];
        let mut queue: Vec<usize> = vec![self.start];
        reachable[self.start] = true;

        let mut i = 0;
        while i < queue.len() {
            let s = queue[i];
            i += 1;

            let eclose_s = &ecloses[s];
            for &e in eclose_s {
                for (sym, target) in &self.states[e].transitions {
                    if let Symbol::Byte(_b) = sym {
                        if !reachable[*target] {
                            reachable[*target] = true;
                            queue.push(*target);
                        }
                    }
                }
            }
        }

        let mut state_mapping: HashMap<usize, usize> = HashMap::new();
        let mut nfa_states: Vec<State> = Vec::new();
        let mut accept_states: Vec<usize> = Vec::new();

        for (old_idx, &is_reachable) in reachable.iter().enumerate() {
            if is_reachable {
                let new_idx = nfa_states.len();
                state_mapping.insert(old_idx, new_idx);
                nfa_states.push(State {
                    transitions: vec![],
                });

                // STEP 4: Mark accepting states
                if ecloses[old_idx].contains(&self.accept) {
                    accept_states.push(new_idx);
                }
            }
        }

        // STEP 3: Define the Transitions
        for (old_idx, &new_idx) in &state_mapping {
            let eclose_s = &ecloses[*old_idx];

            let mut symbol_targets: HashMap<u8, Vec<usize>> = HashMap::new();

            for &e in eclose_s {
                for (sym, target) in &self.states[e].transitions {
                    if let Symbol::Byte(b) = sym {
                        let targets = symbol_targets.entry(*b).or_insert_with(Vec::new);
                        for &t in &ecloses[*target] {
                            if !targets.contains(&t) {
                                targets.push(t);
                            }
                        }
                    }
                }
            }

            for (b, targets) in symbol_targets {
                for target in targets {
                    if let Some(&new_target) = state_mapping.get(&target) {
                        nfa_states[new_idx]
                            .transitions
                            .push((Symbol::Byte(b), new_target));
                    }
                }
            }
        }

        let new_start = state_mapping.get(&self.start).copied().unwrap_or(0);

        Nfa {
            states: nfa_states,
            start: new_start,
            accept: accept_states,
        }
    }
}

impl Nfa {
    pub fn is_isomorphic_to(&self, other: &Nfa) -> bool {
        if self.states.len() != other.states.len() {
            return false;
        }
        isomorphic_nfa(self, other)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct NfaNodeAttr {
    start: bool,
    accept: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct NfaEdgeAttr {
    byte: u8,
}

fn to_nfa_graph(nfa: &Nfa) -> Graph<NfaNodeAttr, NfaEdgeAttr> {
    let mut g = Graph::new();
    let mut nodes = Vec::new();

    for i in 0..nfa.states.len() {
        let is_accept = nfa.accept.contains(&i);
        nodes.push(g.add_node(NfaNodeAttr {
            start: i == nfa.start,
            accept: is_accept,
        }));
    }

    for (i, state) in nfa.states.iter().enumerate() {
        for (sym, target) in &state.transitions {
            if let Symbol::Byte(b) = sym {
                g.add_edge(nodes[i], nodes[*target], NfaEdgeAttr { byte: *b });
            }
        }
    }

    g
}

fn isomorphic_nfa(a: &Nfa, b: &Nfa) -> bool {
    let ga = to_nfa_graph(a);
    let gb = to_nfa_graph(b);

    is_isomorphic_matching(&ga, &gb, |na, nb| na == nb, |ea, eb| ea == eb)
}
