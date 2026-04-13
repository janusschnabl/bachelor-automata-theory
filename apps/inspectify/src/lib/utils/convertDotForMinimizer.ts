import type { EdgeData, NodeData } from './dotHandler';
import { parseNodesAndEdges } from './dotHandler';

export function convertDotToMinimizer(dotString: string): string {
  const { nodes, edges } = parseNodesAndEdges(dotString);

  // Find initial state
  const initialState = nodes.find(node => node.isInitial);
  if (!initialState) {
    throw new Error('No initial state found');
  }

  // Find accepting states
  const acceptingStates = nodes.filter(node => node.isAccepting);
  if (acceptingStates.length === 0) {
    throw new Error('No accepting states found');
  }

  // Extract alphabet from edge labels
  const alphabetSet = new Set<string>();
  for (const edge of edges) {
    const labels = edge.label.split(',');
    for (const label of labels) {
      alphabetSet.add(label.trim());
    }
  }
  const alphabet = Array.from(alphabetSet).sort();

  // Sort states by ID (numeric part)
  const sortedStates = nodes.sort((a, b) => parseInt(a.id) - parseInt(b.id));

  // Build output
  let result = '';

  // States line
  result += `states: ${sortedStates.map(node => node.id).join(' ')}\n`;

  // Initial state
  result += `initial: ${initialState.id}\n`;

  // Alphabet
  result += `alphabet: ${alphabet.join(' ')}\n`;

  // Accepting states
  result += `accepting: ${acceptingStates.map(node => node.id).join(' ')}\n`;

  // Transitions
  result += 'transitions:\n';

  // Sort transitions by from state, then by to state
  const sortedEdges = edges.sort((a, b) => {
    const aFrom = parseInt(a.from);
    const bFrom = parseInt(b.from);
    if (aFrom !== bFrom) return aFrom - bFrom;
    return parseInt(a.to) - parseInt(b.to);
  });

  for (const edge of sortedEdges) {
    result += `${edge.from}, ${edge.label} -> ${edge.to}\n`;
  }

  return result;
}
