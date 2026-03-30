use petgraph::algo::is_isomorphic_matching;
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{EpsilonNfa, State, Symbol};

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
        // ===== Compute epsilon closure for each state in the epsilon-NFA =====
        let ecloses: Vec<Vec<usize>> = (0..self.states.len()).map(|s| self.eclose(s)).collect();

        // STEP 2: Create New States for the NFA using Subset Construction
        // ===== Each NFA state represents a set of epsilon-NFA states =====
        let mut state_map: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut nfa_states: Vec<State> = Vec::new();
        let mut accept_states: Vec<usize> = Vec::new();
        let mut queue: VecDeque<Vec<usize>> = VecDeque::new();

        // Start with epsilon closure of the start state
        let mut start_set = ecloses[self.start].clone();
        start_set.sort();
        start_set.dedup();

        queue.push_back(start_set.clone());
        state_map.insert(start_set.clone(), 0);
        nfa_states.push(State {
            transitions: vec![],
        });

        // Check if start state is accepting
        if start_set.contains(&self.accept) {
            accept_states.push(0);
        }

        // STEP 3: Define the Transitions
        // ===== Build transitions by exploring reachable state sets =====
        // ===== FIX: Collect ALL reachable targets for each symbol, not just one =====
        while let Some(current_set) = queue.pop_front() {
            let current_idx = state_map[&current_set];

            // For each symbol, find all reachable epsilon-NFA states
            let mut symbol_targets: HashMap<u8, HashSet<usize>> = HashMap::new();

            for &state in &current_set {
                for (sym, target) in &self.states[state].transitions {
                    if let Symbol::Byte(b) = sym {
                        // Add all states in the epsilon closure of this target
                        symbol_targets.entry(*b).or_insert_with(HashSet::new);
                        for &s in &ecloses[*target] {
                            symbol_targets.get_mut(b).unwrap().insert(s);
                        }
                    }
                }
            }

            // Create transitions for each symbol
            for (b, target_set) in symbol_targets {
                let mut target_vec: Vec<usize> = target_set.into_iter().collect();
                target_vec.sort();
                target_vec.dedup();

                // Check if this target set already exists
                if !state_map.contains_key(&target_vec) {
                    let new_idx = nfa_states.len();
                    state_map.insert(target_vec.clone(), new_idx);
                    nfa_states.push(State {
                        transitions: vec![],
                    });
                    queue.push_back(target_vec.clone());

                    // STEP 4: Set Accepting States
                    // ===== Mark as accepting if set contains the accept state =====
                    if target_vec.contains(&self.accept) {
                        accept_states.push(new_idx);
                    }
                }

                let target_idx = state_map[&target_vec];
                nfa_states[current_idx]
                    .transitions
                    .push((Symbol::Byte(b), target_idx));
            }
        }

        Nfa {
            states: nfa_states,
            start: 0,
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
