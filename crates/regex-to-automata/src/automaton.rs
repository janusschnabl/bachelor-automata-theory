use std::collections::HashSet;
use std::collections::{BTreeMap, BTreeSet};
use crate::errors::Result;
use petgraph::graph::Graph;
use petgraph::algo::is_isomorphic_matching;

/// Generic state representation for any automaton
#[derive(Debug, Clone)]
pub struct State<L> {
    pub transitions: Vec<(L, usize)>,
}

impl<L> State<L> {
    pub fn new() -> Self {
        State {
            transitions: vec![],
        }
    }
}

impl<L> Default for State<L> {
    fn default() -> Self {
        State::new()
    }
}

/// Core trait for automata types (EpsilonNfa, Nfa, Dfa)
/// with custom label types and shared generic logic.
pub trait Automaton: Sized + Default {
    /// The label type used for edges
    type Label: Clone;

    /// Returns the number of states in the automaton
    fn state_count(&self) -> usize {
        self.get_states().len()
    }

    /// Returns the start state index
    fn start_state(&self) -> usize;

    /// Returns set of accept state indices
    fn accept_states(&self) -> HashSet<usize>;

    /// Returns transitions from a given state as (label, target_state) pairs
    fn transitions_from(&self, state: usize) -> Vec<(Self::Label, usize)> {
        self.get_states()[state].transitions.clone()
    }

    /// Returns the alphabet (set of symbols that can appear on edges)
    fn alphabet(&self) -> &HashSet<u8>;

    /// Returns immutable reference to all states as a vector
    fn get_states(&self) -> &Vec<State<Self::Label>>;

    /// Returns mutable reference to all states as a vector
    fn get_states_mut(&mut self) -> &mut Vec<State<Self::Label>>;

    /// Encodes a label to a string for serialization (to_dot)
    /// E.g., Symbol::Epsilon -> "ε", Symbol::Byte(65) -> "A"
    fn encode_label(label: &Self::Label) -> String;

    /// Decodes a string label back to the label type (from_dot)
    /// E.g., "ε" -> Symbol::Epsilon, "A" -> Symbol::Byte(65)
    fn decode_label(label: &str) -> Result<Self::Label>;

    /// Returns all states reachable from a given state via a specific byte
    /// Handles epsilon transitions and label matching internally
    /// - DFA: returns singleton set if transition exists, empty otherwise
    /// - NFA: returns all states with matching byte transitions
    /// - EpsilonNfa: returns epsilon-closure of all states with matching byte transitions
    fn next_states(&self, state: usize, byte: u8) -> HashSet<usize>;

    /// Simulates the automaton on a word; returns true if accepted
    fn accepts(&self, word: &str) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(self.start_state());
        accepts_from_states(self, &current_states, word)
    }

    /// Generates a DOT representation of the automaton
    /// Default implementation uses encode_label for edge labels
    fn to_dot(&self) -> String {
        automaton_to_dot_impl(self)
    }

    /// Parses a DOT representation and reconstructs the automaton
    /// Default implementation parses via crate::dot::parse_dot_into_automaton
    fn from_dot(dot: &str) -> Result<Self> {
        from_dot_default(dot)
    }

    /// Checks if two automata are isomorphic (structurally equivalent)
    /// Default implementation uses generic graph isomorphism checking
    fn is_isomorphic_to(&self, other: &Self) -> bool {
        automaton_isomorphic(self, other)
    }

    /// Adds a new state to the automaton and returns its index
    fn add_state(&mut self) -> usize {
        let id = self.get_states().len();
        self.get_states_mut().push(State::new());
        id
    }

    /// Adds a transition from one state to another with the given label
    fn add_transition(&mut self, from: usize, label: Self::Label, to: usize) {
        self.get_states_mut()[from].transitions.push((label, to));
    }

    /// Sets the start state of the automaton
    fn set_start(&mut self, state: usize);

    /// Sets the accept states of the automaton (implementation-specific behavior)
    /// Returns an error if the states are invalid for this automaton type
    fn set_accept_states(&mut self, states: HashSet<usize>) -> Result<()>;
}

/// Helper: simulates from a given set of initial states
/// Used to customize the initial state set (e.g., epsilon closure for EpsilonNfa)
pub(crate) fn accepts_from_states<A: Automaton>(
    automaton: &A,
    initial_states: &HashSet<usize>,
    word: &str,
) -> bool {
    let bytes = word.as_bytes();
    let mut current_states = initial_states.clone();

    for &byte in bytes {
        let mut next = HashSet::new();
        for state in current_states {
            next.extend(automaton.next_states(state, byte));
        }
        current_states = next;

        if current_states.is_empty() {
            return false;
        }
    }

    current_states.into_iter().any(|state| automaton.accept_states().contains(&state))
}

/// Default implementation of from_dot parsing for any Automaton type
/// Parse DOT, populate states, and apply start/accept configuration
/// Can be called by type-specific overrides for validation
pub(crate) fn from_dot_default<A: Automaton>(dot: &str) -> Result<A> {
    let mut automaton = A::default();
    let (start, accept_states) = crate::dot::parse_dot_into_automaton(&mut automaton, dot)?;
    automaton.set_start(start);
    automaton.set_accept_states(accept_states)?;
    Ok(automaton)
}



/// Generic helper for converting any Automaton to DOT format
fn automaton_to_dot_impl<A: Automaton>(automaton: &A) -> String {
    let mut s = String::new();
    s.push_str("digraph NFA {\n");
    s.push_str("  rankdir=LR;\n");

    let accept_states = automaton.accept_states();
    let start = automaton.start_state();

    // Declare all nodes with attributes
    for i in 0..automaton.state_count() {
        let mut attrs = Vec::new();

        if i == start {
            attrs.push("isInitial=true");
        }
        if accept_states.contains(&i) {
            attrs.push("isAccepting=true");
        }

        if attrs.is_empty() {
            s.push_str(&format!("  {};\n", i));
        } else {
            s.push_str(&format!("  {} [{}];\n", i, attrs.join(", ")));
        }
    }

    s.push('\n');

    // Add edges
    for from in 0..automaton.state_count() {
        for (label, to) in automaton.transitions_from(from) {
            let encoded = A::encode_label(&label);
            s.push_str(&format!("  {} -> {} [label=\"{}\"];\n", from, to, encoded));
        }
    }

    s.push_str("}\n");
    s
}

/// Node attributes for graph representation (used in isomorphism checking)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeAttr {
    pub start: bool,
    pub accept: bool,
}

/// Generic helper to convert any Automaton to a (non multigraph) graph representation
pub(crate) fn automaton_to_graph_impl<A: Automaton>(automaton: &A) -> Graph<NodeAttr, BTreeSet<String>> {
    let mut g = Graph::new();
    let mut nodes = Vec::new();

    let accept_states = automaton.accept_states();
    let start = automaton.start_state();

    for i in 0..automaton.state_count() {
        nodes.push(g.add_node(NodeAttr {
            start: i == start,
            accept: accept_states.contains(&i),
        }));
    }

    let mut edge_labels: BTreeMap<(usize, usize), BTreeSet<String>> = BTreeMap::new();

    for from in 0..automaton.state_count() {
        for (label, to) in automaton.transitions_from(from) {
            edge_labels
                .entry((from, to))
                .or_default()
                .insert(A::encode_label(&label));
        }
    }

    for ((from, to), labels) in edge_labels {
        g.add_edge(nodes[from], nodes[to], labels);
    }

    g
}

pub fn automaton_isomorphic<A: Automaton>(a: &A, b: &A) -> bool {
    if a.state_count() != b.state_count() {
        return false;
    }

    let ga = automaton_to_graph_impl(a);
    let gb = automaton_to_graph_impl(b);

    is_isomorphic_matching(
        &ga,
        &gb,
        |na, nb| na == nb,
        |ea, eb| ea == eb,
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfa::Nfa;

    #[test]
    //so we don't pass multigraphs to the isomorphism checker.
    fn automaton_to_graph_collapses_parallel_edges() {
        let mut nfa = Nfa::default();

        let s0 = nfa.add_state();
        let s1 = nfa.add_state();

        nfa.set_start(s0);
        nfa.set_accept_states([s1].into()).unwrap();

        nfa.add_transition(s0, b'a', s1);
        nfa.add_transition(s0, b'b', s1);

        let graph = automaton_to_graph_impl(&nfa);

        let edges: Vec<_> = graph.edge_references().collect();

        assert_eq!(edges.len(), 1);

        let labels = edges[0].weight();

        assert!(labels.contains("a"));
        assert!(labels.contains("b"));
        assert_eq!(labels.len(), 2);
    }
}