use petgraph::graph::DiGraph;
use petgraph::dot::{Dot, Config};
use serde::{Serialize, Deserialize};
use tapi::Tapi;
use ce_core::{Env, Generate, ValidationResult, EnvError};
use regex_syntax::Parser;

// Define Input and Output types
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Tapi)]
pub struct Input {
    pub regex: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Tapi)]
pub struct Output {
    pub dot: String,
}

// Define the RegexToDfaEnv type
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegexToDfaEnv;

impl Env for RegexToDfaEnv {
    type Input = Input;
    type Output = Output;
    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        // Parse the regex into an AST
        let ast = Parser::new()
            .parse(&input.regex)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?;

        // Build the NFA dynamically from the AST
        let mut graph = DiGraph::<String, String>::new();

        // Build NFA fragments for each regex subexpression
        fn build_fragment(
            hir: &regex_syntax::hir::Hir,
            graph: &mut DiGraph<String, String>,
        ) -> (petgraph::graph::NodeIndex, petgraph::graph::NodeIndex) {
            use regex_syntax::hir::{HirKind};

            match hir.kind() {
                HirKind::Literal(literal) => {
                    let mut current_start = graph.add_node("start".to_string());
                    let mut current_accept;

                    for c in String::from_utf8_lossy(&literal.0).chars() {
                        current_accept = graph.add_node("accept".to_string());
                        graph.add_edge(current_start, current_accept, c.to_string());
                        current_start = current_accept;
                    }

                    (graph.node_indices().next().unwrap(), current_start)
                }
                HirKind::Empty => {
                    let start = graph.add_node("start".to_string());
                    let accept = graph.add_node("accept".to_string());
                    graph.add_edge(start, accept, "ε".to_string());
                    (start, accept)
                }
                HirKind::Concat(asts) => {
                    let mut current_start = None;
                    let mut current_accept = None;

                    for sub_ast in asts {
                        let (sub_start, sub_accept) = build_fragment(sub_ast, graph);
                        if let Some(accept) = current_accept {
                            graph.add_edge(accept, sub_start, "ε".to_string());
                        } else {
                            current_start = Some(sub_start);
                        }
                        current_accept = Some(sub_accept);
                    }

                    (current_start.unwrap(), current_accept.unwrap())
                }
                HirKind::Alternation(asts) => {
                    let start = graph.add_node("start".to_string());
                    let accept = graph.add_node("accept".to_string());

                    for sub_ast in asts {
                        let (sub_start, sub_accept) = build_fragment(sub_ast, graph);
                        graph.add_edge(start, sub_start, "ε".to_string());
                        graph.add_edge(sub_accept, accept, "ε".to_string());
                    }

                    (start, accept)
                }
                HirKind::Repetition(rep) => {
                    let start = graph.add_node("start".to_string());
                    let accept = graph.add_node("accept".to_string());

                    let (sub_start, sub_accept) = build_fragment(&rep.sub, graph);

                    graph.add_edge(start, sub_start, "ε".to_string());
                    graph.add_edge(start, accept, "ε".to_string());
                    graph.add_edge(sub_accept, sub_start, "ε".to_string());
                    graph.add_edge(sub_accept, accept, "ε".to_string());

                    (start, accept)
                }
                HirKind::Capture(capture) => build_fragment(&capture.sub, graph),
                _ => panic!("Unsupported regex construct for Thompson construction"),
            }
        }

        // Replace the `traverse_ast` call with `build_fragment`
        let (start, accept) = build_fragment(&ast, &mut graph);

        // Annotate states with `isInitial` and `isAccepting`
        let dot_output = format!(
            "{}",
            Dot::with_attr_getters(
                &graph,
                &[Config::EdgeNoLabel],
                &|_, edge| format!("label=\"{}\"", edge.weight()),
                &|_, (node_idx, node)| {
                    if node_idx == start {
                        "label=start isInitial=true".to_string()
                    } else if node_idx == accept {
                        "label=accept isAccepting=true".to_string()
                    } else {
                        format!("label=\"{}\"", node)
                    }
                },
            )
        );

        // Return the DOT representation
        Ok(Output { dot: dot_output })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, _rng: &mut R) -> Self {
        Input::default()
    }
}
