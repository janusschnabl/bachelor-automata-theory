use crate::{EpsilonNfa, Symbol, State};
use crate::errors::{Error, Result};

use graphviz_rust::dot_structures::{EdgeTy, Graph, Id, Stmt, Vertex};

//This entire file is literally just made by chat, but i don't care too much for writing simple formatting stuff by hand and it seems to work.
impl EpsilonNfa {
    pub fn to_dot(&self) -> String {
        let mut s = String::new();

        s.push_str("digraph NFA {\n");
        s.push_str("  rankdir=LR;\n");

        for (i, _) in self.states.iter().enumerate() {
            let mut attrs = Vec::new();

            if i == self.start {
                attrs.push("isInitial=true");
            }
            if i == self.accept {
                attrs.push("isAccepting=true");
            }

            if attrs.is_empty() {
                s.push_str(&format!("  {};\n", i));
            } else {
                s.push_str(&format!("  {} [{}];\n", i, attrs.join(", ")));
            }
        }

        s.push('\n');

        for (from, state) in self.states.iter().enumerate() {
            for (symbol, to) in &state.transitions {
                s.push_str(&format!("  {} -> {} [label=\"{}\"];\n", from, to, symbol));
            }
        }

        s.push_str("}\n");

        s
    }

    pub fn from_dot(dot: &str) -> Result<Self> {
        let parsed = graphviz_rust::parse(dot)
            .map_err(|e| Error::InvalidInput(format!("dot parse error: {e}")))?;

        let mut nfa = EpsilonNfa::new();

        let mut start = None;
        let mut accept = None;

        let stmts = match parsed {
            Graph::DiGraph { stmts, .. } => stmts,
            _ => return Err(Error::InvalidInput("expected digraph".into())),
        };

        for stmt in stmts {
            match stmt {

                Stmt::Node(n) => {
                    let id = id_to_usize(&n.id.0)?;

                    ensure_state(&mut nfa, id);

                    for attr in n.attributes {
                        if let (Id::Plain(key), Id::Plain(val)) = (&attr.0, &attr.1) {
                            match (key.as_str(), val.as_str()) {
                                ("isInitial", "true") => {
                                    if start.replace(id).is_some() {
                                        return Err(Error::InvalidInput("multiple start states".into()));
                                    }
                                }
                                ("isAccepting", "true") => {
                                    if accept.replace(id).is_some() {
                                        return Err(Error::InvalidInput("multiple accept states".into()));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                Stmt::Edge(e) => {
                    if let EdgeTy::Pair(a, b) = e.ty {
                        if let (Vertex::N(a), Vertex::N(b)) = (a, b) {

                            let from = id_to_usize(&a.0)?;
                            let to = id_to_usize(&b.0)?;

                            ensure_state(&mut nfa, from);
                            ensure_state(&mut nfa, to);

                            let label = e.attributes.iter().find_map(|attr| {
                                if let (Id::Plain(key), Id::Escaped(val)) = (&attr.0, &attr.1) {
                                    if key == "label" {
                                        return Some(val.trim_matches('"').to_string());
                                    }
                                }
                                None
                            }).ok_or_else(|| Error::InvalidInput("edge missing label".into()))?;

                            let symbol = if label == "ε" {
                                Symbol::Epsilon
                            } else if label.len() == 1 {
                                Symbol::Byte(label.as_bytes()[0])
                            } else {
                                return Err(Error::InvalidInput(format!("invalid label: {label}")));
                            };

                            nfa.add_transition(from, symbol, to);
                        }
                    }
                }

                _ => {}
            }
        }

        nfa.start = start.ok_or_else(|| Error::InvalidInput("missing start state".into()))?;
        nfa.accept = accept.ok_or_else(|| Error::InvalidInput("missing accept state".into()))?;

        Ok(nfa)
    }
}


fn ensure_state(nfa: &mut EpsilonNfa, id: usize) {
    while nfa.states.len() <= id {
        nfa.states.push(State { transitions: vec![] });
    }
}


fn id_to_usize(id: &Id) -> Result<usize> {
    let s = match id {
        Id::Plain(s) | Id::Escaped(s) | Id::Html(s) => s,
        Id::Anonymous(_) => {
            return Err(Error::InvalidInput(
                "anonymous node ids are not supported".into(),
            ))
        }
    };

    s.parse()
        .map_err(|_| Error::InvalidInput("node id must be integer".into()))
}