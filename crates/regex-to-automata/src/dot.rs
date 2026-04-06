use crate::automaton::Automaton;
use crate::errors::{Error, Result};
use std::collections::HashSet;

use graphviz_rust::dot_structures::{EdgeTy, Graph, Id, Stmt, Vertex};

/// Generic helper for parsing DOT format into any Automaton type
/// Returns (start_state, accept_states) for the caller to apply via setters
pub(crate) fn parse_dot_into_automaton<A: Automaton>(
    automaton: &mut A,
    dot: &str,
) -> Result<(usize, HashSet<usize>)> {
    let parsed = graphviz_rust::parse(dot)
        .map_err(|e| Error::InvalidInput(format!("dot parse error: {e}")))?;

    let mut start = None;
    let mut accept_states = HashSet::new();

    let stmts = match parsed {
        Graph::DiGraph { stmts, .. } => stmts,
        _ => return Err(Error::InvalidInput("expected digraph".into())),
    };

    for stmt in stmts {
        match stmt {
            Stmt::Node(n) => {
                let id = parse_dot_id(&n.id.0)?;

                ensure_automaton_state(automaton, id);

                for attr in n.attributes {
                    if let (Id::Plain(key), Id::Plain(val)) = (&attr.0, &attr.1) {
                        match (key.as_str(), val.as_str()) {
                            ("isInitial", "true") => {
                                if start.replace(id).is_some() {
                                    return Err(Error::InvalidInput(
                                        "multiple start states".into(),
                                    ));
                                }
                            }
                            ("isAccepting", "true") => {
                                accept_states.insert(id);
                            }
                            _ => {}
                        }
                    }
                }
            }

            Stmt::Edge(e) => {
                if let EdgeTy::Pair(a, b) = e.ty {
                    if let (Vertex::N(a), Vertex::N(b)) = (a, b) {
                        let from = parse_dot_id(&a.0)?;
                        let to = parse_dot_id(&b.0)?;

                        ensure_automaton_state(automaton, from);
                        ensure_automaton_state(automaton, to);

                        let label_str = e
                            .attributes
                            .iter()
                            .find_map(|attr| {
                                if let (Id::Plain(key), Id::Escaped(val)) = (&attr.0, &attr.1) {
                                    if key == "label" {
                                        return Some(val.trim_matches('"').to_string());
                                    }
                                }
                                None
                            })
                            .ok_or_else(|| Error::InvalidInput("edge missing label".into()))?;

                        let label = A::decode_label(&label_str)?;
                        automaton.add_transition(from, label, to);
                    }
                }
            }

            _ => {}
        }
    }

    let start_state = start.ok_or_else(|| Error::InvalidInput("missing start state".into()))?;
    if accept_states.is_empty() {
        return Err(Error::InvalidInput("missing accept states".into()));
    }

    Ok((start_state, accept_states))
}

fn ensure_automaton_state<A: Automaton>(automaton: &mut A, id: usize) {
    while automaton.get_states().len() <= id {
        automaton.add_state();
    }
}

fn parse_dot_id(id: &Id) -> Result<usize> {
    let s = match id {
        Id::Plain(s) | Id::Escaped(s) | Id::Html(s) => s,
        Id::Anonymous(_) => {
            return Err(Error::InvalidInput(
                "anonymous node ids are not supported".into(),
            ));
        }
    };

    s.parse()
        .map_err(|_| Error::InvalidInput("node id must be integer".into()))
}
