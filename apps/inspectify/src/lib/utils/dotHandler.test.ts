import { describe, expect, it } from 'vitest';

import { convertDotToMinimizer } from './convertDotForMinimizer';
import { parseAndCompactDot, parseAndRemoveUnreachableStates, parseNodesAndEdges } from './dotHandler';

const sampleDot = `
digraph DFA {
  rankdir=LR;
  q0[label="q0"];
  q1[label="q1"];

  q0 -> q1[label="a"];
  q0 -> q1[label="b"];
  q0 -> q1[label="c"];
  q0 -> q1[label="d"];
  q0 -> q1[label="e"];
  q1 -> q2[label="f"];
}
`;

const interleavedDot = `
digraph G {
  q0[label="q0"]; q0 -> q1[label="a"]; q1[label="q1"];
  q0[label="q0"]; q0 -> q1[label="b"]; q1[label="q1"];
  q0[label="q0"]; q0 -> q1[label="c"]; q1[label="q1"];
  q0[label="q0"]; q0 -> q1[label="d"]; q1[label="q1"];
  q0[label="q0"]; q0 -> q1[label="e"]; q1[label="q1"];
  q1[label="q1"]; q1 -> q2[label="f"]; q2[label="q2"];
}
`;

const finalSampleDot = `
digraph G {
  start[label="start"];
  accept[label="accept"];
  sink[label="sink"];
  start -> accept[label="a,b"];
  start -> accept[label="c"];
  start -> accept[label="d,e"];
  accept -> sink[label="f"];
}
`;

describe('parseNodesAndEdges', () => {
  it('extracts nodes, attributes, and edges from DOT input', () => {
    const parsed = parseNodesAndEdges(sampleDot);

    expect(parsed.nodes).toEqual([
      { id: 'q0', isInitial: false, isAccepting: false, additionalAttrs: 'label=q0' },
      { id: 'q1', isInitial: false, isAccepting: false, additionalAttrs: 'label=q1' },
      { id: 'q2', isInitial: false, isAccepting: false, additionalAttrs: '' },
    ]);

    expect(parsed.edges).toEqual([
      { from: 'q0', to: 'q1', label: 'a' },
      { from: 'q0', to: 'q1', label: 'b' },
      { from: 'q0', to: 'q1', label: 'c' },
      { from: 'q0', to: 'q1', label: 'd' },
      { from: 'q0', to: 'q1', label: 'e' },
      { from: 'q1', to: 'q2', label: 'f' },
    ]);
  });

  it('handles interleaved node and edge statements', () => {
    const parsed = parseNodesAndEdges(interleavedDot);

    expect(parsed.nodes.map((node) => node.id)).toEqual(['q0', 'q1', 'q2']);

    expect(parsed.edges).toEqual([
      { from: 'q0', to: 'q1', label: 'a' },
      { from: 'q0', to: 'q1', label: 'b' },
      { from: 'q0', to: 'q1', label: 'c' },
      { from: 'q0', to: 'q1', label: 'd' },
      { from: 'q0', to: 'q1', label: 'e' },
      { from: 'q1', to: 'q2', label: 'f' },
    ]);
  });
});

describe('parseAndCompactDot', () => {
  it('compacts parallel edges and preserves node attributes', () => {
    const compacted = parseAndCompactDot(sampleDot);

    expect(compacted).toBe(`digraph DFA {
  rankdir=LR
  "q0" [label=q0];
  "q1" [label=q1];
  "q2";

  "q0" -> "q1" [label="a,b,c,d,e"];
  "q1" -> "q2" [label="f"];
}
`);
  });

  it('compacts comma-separated labels on repeated transitions to the same node', () => {
    const compacted = parseAndCompactDot(finalSampleDot);

    expect(compacted).toBe(`digraph DFA {
  rankdir=LR
  "accept" [label=accept];
  "sink" [label=sink];
  "start" [label=start];

  "start" -> "accept" [label="a,b,c,d,e"];
  "accept" -> "sink" [label="f"];
}
`);
  });
});

describe('parseAndRemoveUnreachableStates', () => {
  it('removes states that cannot reach an accepting state', () => {
    const dot = `
digraph DFA {
  rankdir=LR;
  "0" [isInitial=true];
  "1";
  "2" [isAccepting=true];
  "3";
  "4";

  "0" -> "1" [label="a"];
  "1" -> "2" [label="b"];
  "3" -> "4" [label="c"];
}
`;

    const cleaned = parseAndRemoveUnreachableStates(dot);

    expect(cleaned).toBe(`digraph DFA {
  rankdir=LR
  "0" [isInitial=true];
  "1";
  "2" [isAccepting=true];

  "0" -> "1" [label="a"];
  "1" -> "2" [label="b"];
}
`);
  });
});

describe('convertDotToMinimizer', () => {
  it('converts DOT to the minimizer format', () => {
    const converted = convertDotToMinimizer(`
digraph DFA {
  rankdir=LR;
  "1" [isInitial=true];
  "2" [isAccepting=true];
  "3";

  "1" -> "2" [label="b"];
  "1" -> "3" [label="a"];
  "2" -> "3" [label="c,d"];
}
`);

    expect(converted).toBe(`states: 1 2 3
initial: 1
alphabet: a b c d
accepting: 2
transitions:
1, b -> 2
1, a -> 3
2, c,d -> 3
`);
  });

  it('keeps comma-separated labels intact while splitting the alphabet', () => {
    const converted = convertDotToMinimizer(`
digraph G {
  start[label="start"]; start [isInitial=true];
  accept[label="accept"]; accept [isAccepting=true];

  start -> accept[label="a,b,c,d,e"];
  accept -> sink[label="f"];
}
`);

    expect(converted).toBe(`states: accept sink start
initial: start
alphabet: a b c d e f
accepting: accept
transitions:
accept, f -> sink
start, a,b,c,d,e -> accept
`);
  });

  it('throws when the initial state is missing', () => {
    expect(() =>
      convertDotToMinimizer(`
digraph DFA {
  rankdir=LR;
  "1" [isAccepting=true];
}
`)
    ).toThrow('No initial state found');
  });

  it('throws when there are no accepting states', () => {
    expect(() =>
      convertDotToMinimizer(`
digraph DFA {
  rankdir=LR;
  "1" [isInitial=true];
}
`)
    ).toThrow('No accepting states found');
  });
});