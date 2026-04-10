use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, Nfa, Result};
use std::collections::{BTreeSet, HashSet, BTreeMap};

#[derive(Debug, Clone, Default)]
pub struct Dfa {
    pub states: Vec<State<Symbol>>,
    pub start: usize,
    pub accept: Vec<usize>,
    pub alphabet: HashSet<u8>,
}
impl Nfa {
    pub fn to_dfa(&self) -> Dfa {
        Dfa::subset_construction(self)
    }
}
// TODO: Move algorithm to its own file and add tests showing we get the correct construction modulo state names
impl Dfa {
    fn subset_construction(nfa: &Nfa) -> Dfa {
        //TODO: refactor this to make more clear and maybe see if it can become more effecient.
        let mut dfa: BTreeMap<BTreeSet<usize>, BTreeMap<u8, BTreeSet<usize>>> = BTreeMap::new();   
        let mut worklist= HashSet::new();
        let alphabet = nfa.alphabet().clone();
        
        worklist.insert(BTreeSet::from([nfa.start]));

        while let Some(current) = worklist.iter().next().cloned() {
            worklist.remove(&current);
            if dfa.contains_key(&current) {
                continue;
            }

            let mut transitions: BTreeMap<u8, BTreeSet<usize>> = BTreeMap::new();
            for byte in &alphabet {  
                let mut next_states = BTreeSet::new();
                for state in &current {
                    for next_state in nfa.next_states(*state, *byte) {
                        next_states.insert(next_state);
                    }
                }
                //TODO: maybe use an explicit "seen" set to avoid inserting states repeatedly even though it is asymptotically the same.
                worklist.insert(next_states.clone());
                transitions.insert(*byte, next_states);
            }

            dfa.insert(current, transitions);
        }

        let initial_state = BTreeSet::from([nfa.start]);
        let mut accepting_state: HashSet<BTreeSet<usize>> = HashSet::new();
        for (state_set,_) in &dfa {
            if state_set.iter().any(|s| nfa.accept_states().contains(s)) {
                accepting_state.insert(state_set.clone());
            }
        }

        Dfa::build_dfa_from_subsets(dfa, initial_state, accepting_state, alphabet)

    }

    fn build_dfa_from_subsets(dfa: BTreeMap<BTreeSet<usize>, BTreeMap<u8, BTreeSet<usize>>>, initial_state: BTreeSet<usize>, accepting_states: HashSet<BTreeSet<usize>>, alphabet: HashSet<u8>) -> Dfa {        
        let mut state_indices: BTreeMap<BTreeSet<usize>, usize> = BTreeMap::new();
        let mut states: Vec<State<Symbol>> = vec![];
        let mut accept_states: Vec<usize> = vec![];

        for (i, state_set) in dfa.keys().enumerate() {
            state_indices.insert(state_set.clone(), i);
            states.push(State::new());
            if accepting_states.contains(state_set) {
                accept_states.push(i);
            }
        }

        for (state_set, transitions) in dfa {
            let from_index = state_indices[&state_set];
            for (byte, next_set) in transitions {
                let to_index = state_indices[&next_set];
                states[from_index].transitions.push((Symbol::Byte(byte), to_index));
            }
        }


        Dfa {
            states: states,
            start: state_indices[&initial_state],
            accept: accept_states,
            alphabet: alphabet,
        }
    }

}

impl Automaton for Dfa {
    type Label = Symbol;

    fn start_state(&self) -> usize {
        self.start
    }

    fn accept_states(&self) -> HashSet<usize> {
        self.accept.iter().copied().collect()
    }

    fn alphabet(&self) -> &HashSet<u8> {
        &self.alphabet
    }

    fn get_states(&self) -> &Vec<crate::automaton::State<Self::Label>> {
        &self.states
    }

    fn get_states_mut(&mut self) -> &mut Vec<crate::automaton::State<Self::Label>> {
        &mut self.states
    }

    fn encode_label(label: &Symbol) -> String {
        // Genbrug EpsilonNfa's implementation
        crate::epsilon_nfa::EpsilonNfa::encode_label(label)
    }

    fn decode_label(label: &str) -> Result<Symbol> {
        crate::epsilon_nfa::EpsilonNfa::decode_label(label)
    }

    fn next_states(&self, state: usize, byte: u8) -> HashSet<usize> {
        let mut next = HashSet::new();
        for (symbol, target) in &self.states[state].transitions {
            if let Symbol::Byte(b) = symbol {
                if *b == byte {
                    next.insert(*target);
                    return next;
                }
            }
        }
        next
    }

    fn set_start(&mut self, state: usize) {
        self.start = state;
    }

    fn set_accept_states(&mut self, states: HashSet<usize>) -> Result<()> {
        self.accept = states.into_iter().collect();
        Ok(())
    }
}