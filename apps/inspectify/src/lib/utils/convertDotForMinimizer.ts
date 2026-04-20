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

  // Sort states by ID (numeric part if possible, otherwise lexicographic)
  const sortedStates = nodes.sort((a, b) => {
    const aNum = parseInt(a.id);
    const bNum = parseInt(b.id);
    if (!isNaN(aNum) && !isNaN(bNum)) {
      return aNum - bNum;
    }
    return a.id.localeCompare(b.id);
  });

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

  // Sort transitions by from state, then by to state (numeric if possible, otherwise lexicographic)
  const sortedEdges = edges.sort((a, b) => {
    const aFromNum = parseInt(a.from);
    const bFromNum = parseInt(b.from);
    if (!isNaN(aFromNum) && !isNaN(bFromNum)) {
      if (aFromNum !== bFromNum) return aFromNum - bFromNum;
      const aToNum = parseInt(a.to);
      const bToNum = parseInt(b.to);
      if (!isNaN(aToNum) && !isNaN(bToNum)) return aToNum - bToNum;
    }
    if (a.from !== b.from) return a.from.localeCompare(b.from);
    return a.to.localeCompare(b.to);
  });

  for (const edge of sortedEdges) {
    result += `${edge.from}, ${edge.label} -> ${edge.to}\n`;
  }

  return result;
}
