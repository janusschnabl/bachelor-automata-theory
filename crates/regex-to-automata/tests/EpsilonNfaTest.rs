use regex_to_automata::{EpsilonNfa, Symbol};

fn count_eps(nfa: &EpsilonNfa) -> usize {
    nfa.states
        .iter()
        .flat_map(|s| &s.transitions)
        .filter(|(sym, _)| *sym == Symbol::Epsilon)
        .count()
}

fn count_symbol(nfa: &EpsilonNfa, b: u8) -> usize {
    nfa.states
        .iter()
        .flat_map(|s| &s.transitions)
        .filter(|(sym, _)| *sym == Symbol::Byte(b))
        .count()
}

#[test]
fn literal_single() {
    let nfa = EpsilonNfa::from_regex("a").unwrap();

    assert_eq!(nfa.states.len(), 2);
    assert_eq!(count_symbol(&nfa, b'a'), 1);
    assert_eq!(count_eps(&nfa), 0);

    assert_eq!(nfa.start, 0);
    assert_eq!(nfa.accept, 1);
}

#[test]
fn concat_two_literals() {
    let nfa = EpsilonNfa::from_regex("ab").unwrap();

    // Thompson: a (2 states) + b (2 states)
    assert_eq!(nfa.states.len(), 4);

    assert_eq!(count_symbol(&nfa, b'a'), 1);
    assert_eq!(count_symbol(&nfa, b'b'), 1);

    // epsilon connecting them
    assert_eq!(count_eps(&nfa), 1);
}

#[test]
fn alternation_structure() {
    let nfa = EpsilonNfa::from_regex("a|b").unwrap();

    // Thompson alternation introduces start + accept
    assert_eq!(nfa.states.len(), 6);

    assert_eq!(count_symbol(&nfa, b'a'), 1);
    assert_eq!(count_symbol(&nfa, b'b'), 1);

    // start->branches + branches->accept
    assert_eq!(count_eps(&nfa), 4);
}

#[test]
fn kleene_star_structure() {
    let nfa = EpsilonNfa::from_regex("a*").unwrap();

    // literal (2) + wrapper (2)
    assert_eq!(nfa.states.len(), 4);

    assert_eq!(count_symbol(&nfa, b'a'), 1);

    // Thompson star uses 4 epsilons
    assert_eq!(count_eps(&nfa), 4);
}

#[test]
fn plus_structure() {
    let nfa = EpsilonNfa::from_regex("a+").unwrap();

    assert_eq!(nfa.states.len(), 4);

    assert_eq!(count_symbol(&nfa, b'a'), 1);

    // start->sub, loop, exit
    assert_eq!(count_eps(&nfa), 3);
}

#[test]
fn grouping_does_not_change_structure() {
    let a = EpsilonNfa::from_regex("ab").unwrap();
    let b = EpsilonNfa::from_regex("(ab)").unwrap();

    assert_eq!(a.states.len(), b.states.len());
}

#[test]
fn nested_expression() {
    let nfa = EpsilonNfa::from_regex("(a|b)*").unwrap();

    assert_eq!(count_symbol(&nfa, b'a'), 1);
    assert_eq!(count_symbol(&nfa, b'b'), 1);

    // alternation + star wrapper
    assert!(nfa.states.len() >= 8);
}