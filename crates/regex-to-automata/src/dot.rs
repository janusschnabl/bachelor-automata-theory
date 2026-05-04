use crate::automaton::Automaton;
use crate::errors::{Error, Result};
use std::collections::{HashMap, HashSet};

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
    let mut node_id_map: HashMap<String, usize> = HashMap::new();

    let stmts = match parsed {
        Graph::DiGraph { stmts, .. } => stmts,
        _ => return Err(Error::InvalidInput("expected digraph".into())),
    };

    for stmt in stmts {
        match stmt {
            Stmt::Node(n) => {
                let node_name = parse_dot_node_name(&n.id.0)?;
                let id = get_or_create_node_id(&mut node_id_map, node_name);

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
                        let from_name = parse_dot_node_name(&a.0)?;
                        let to_name = parse_dot_node_name(&b.0)?;

                        let from = get_or_create_node_id(&mut node_id_map, from_name);
                        let to = get_or_create_node_id(&mut node_id_map, to_name);

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

fn parse_dot_node_name(id: &Id) -> Result<String> {
    let s = match id {
        Id::Plain(s) | Id::Escaped(s) | Id::Html(s) => s,
        Id::Anonymous(_) => {
            return Err(Error::InvalidInput(
                "anonymous node ids are not supported".into(),
            ));
        }
    };

    let name = if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    Ok(name.to_string())
}

fn get_or_create_node_id(map: &mut HashMap<String, usize>, name: String) -> usize {
    let len = map.len();
    *map.entry(name).or_insert(len)
}

fn format_node_name(state: usize, start_state: usize) -> String {
    if state == start_state {
        "q0".to_string()
    } else {
        format!("q{}", state)
    }
}

/// Generic helper for converting any Automaton to DOT format
pub(crate) fn automaton_to_dot_impl<A: Automaton>(automaton: &A) -> String {
    let mut s = String::new();
    s.push_str("digraph NFA {\n");
    s.push_str("  rankdir=LR;\n");

    let accept_states = automaton.accept_states();
    let start = automaton.start_state();

    for i in 0..automaton.state_count() {
        let mut attrs = Vec::new();

        if i == start {
            attrs.push("isInitial=true");
        }
        if accept_states.contains(&i) {
            attrs.push("isAccepting=true");
        }

        let node_name = format_node_name(i, start);

        if attrs.is_empty() {
            s.push_str(&format!("  \"{}\";\n", node_name));
        } else {
            s.push_str(&format!("  \"{}\" [{}];\n", node_name, attrs.join(", ")));
        }
    }

    s.push('\n');

    for from in 0..automaton.state_count() {
        for (label, to) in automaton.transitions_from(from) {
            let encoded = A::encode_label(&label);
            let from_name = format_node_name(from, start);
            let to_name = format_node_name(to, start);
            s.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                from_name, to_name, encoded
            ));
        }
    }

    s.push_str("}\n");
    s
}
