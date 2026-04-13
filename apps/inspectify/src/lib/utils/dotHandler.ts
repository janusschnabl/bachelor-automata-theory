type NodeData = {
  id: string;
  isInitial: boolean;
  isAccepting: boolean;
};

type EdgeData = {
  from: string;
  to: string;
  label: string;
};

type ParsedAutomaton = {
  nodes: NodeData[];
  edges: EdgeData[];
};

// Helper function to parse nodes and edges from dot format
function parseNodesAndEdges(dotString: string): ParsedAutomaton {
  const nodeRegex = /^\s*(\w+)\s*(?:\[(.*?)\])?;/gm;
  const nodesMap = new Map<string, NodeData>();
  let nodeMatch;

  while ((nodeMatch = nodeRegex.exec(dotString)) !== null) {
    const id = nodeMatch[1];
    const attrs = nodeMatch[2] || '';

    // Skip non-numeric nodes like rankdir, __start, etc.
    if (!/^\d+$/.test(id)) continue;

    nodesMap.set(id, {
      id,
      isInitial: attrs.includes('isInitial=true'),
      isAccepting: attrs.includes('isAccepting=true'),
    });
  }

  // Parse edges and collect any nodes that appear in edges but weren't declared
  const edgeRegex = /(\d+)\s*->\s*(\d+)\s*\[label="([^"]+)"\]/g;
  const edges: EdgeData[] = [];
  let edgeMatch;

  while ((edgeMatch = edgeRegex.exec(dotString)) !== null) {
    const from = edgeMatch[1];
    const to = edgeMatch[2];
    const label = edgeMatch[3];

    // Ensure nodes exist in map (preserve existing attributes)
    if (!nodesMap.has(from)) {
      nodesMap.set(from, { id: from, isInitial: false, isAccepting: false });
    }
    if (!nodesMap.has(to)) {
      nodesMap.set(to, { id: to, isInitial: false, isAccepting: false });
    }

    edges.push({ from, to, label });
  }

  return {
    nodes: Array.from(nodesMap.values()),
    edges,
  };
}

export function parseAndCompactDot(dotString: string): string {
  const { nodes, edges } = parseNodesAndEdges(dotString);

  // Group edges by from->to key and collect all labels
  const edgesMap = new Map<string, Set<string>>();
  for (const edge of edges) {
    const key = `${edge.from}->${edge.to}`;
    if (!edgesMap.has(key)) {
      edgesMap.set(key, new Set());
    }
    edgesMap.get(key)!.add(edge.label);
  }

  // Build compacted dot
  let result = 'digraph DFA {\n  rankdir=LR\n';

  // Add nodes with attributes, preserving initial and accepting states
  // Sort nodes by ID to maintain consistent order
  const sortedNodes = nodes.sort((a, b) => 
    parseInt(a.id) - parseInt(b.id)
  );

  for (const node of sortedNodes) {
    if (node.isInitial || node.isAccepting) {
      const attrs: string[] = [];
      if (node.isInitial) attrs.push('isInitial=true');
      if (node.isAccepting) attrs.push('isAccepting=true');
      result += `  ${node.id} [${attrs.join(', ')}];\n`;
    } else {
      result += `  ${node.id};\n`;
    }
  }

  result += '\n';

  // Add compacted edges
  for (const [key, labels] of edgesMap.entries()) {
    const [from, to] = key.split('->');
    const sortedLabels = Array.from(labels).sort();
    result += `  ${from} -> ${to} [label="${sortedLabels.join(',')}"];\n`;
  }

  result += '}\n';

  return result;
}

export function parseAndRemoveUnreachableStates(dotString: string): string {
  const { nodes, edges } = parseNodesAndEdges(dotString);

  // Find all accepting states
  const acceptingStates = new Set(
    nodes
      .filter(node => node.isAccepting)
      .map(node => node.id)
  );

  // If no accepting states, return original
  if (acceptingStates.size === 0) {
    return dotString;
  }

  // Build reverse adjacency list (to -> from)
  const reverseGraph = new Map<string, Set<string>>();
  for (const edge of edges) {
    if (!reverseGraph.has(edge.to)) {
      reverseGraph.set(edge.to, new Set());
    }
    reverseGraph.get(edge.to)!.add(edge.from);
  }

  // Find all states that can reach an accepting state (backwards search)
  const reachableStates = new Set<string>(acceptingStates);
  const queue = Array.from(acceptingStates);

  while (queue.length > 0) {
    const state = queue.shift()!;
    const predecessors = reverseGraph.get(state);

    if (predecessors) {
      for (const pred of predecessors) {
        if (!reachableStates.has(pred)) {
          reachableStates.add(pred);
          queue.push(pred);
        }
      }
    }
  }

  // Build cleaned dot
  let result = 'digraph DFA {\n  rankdir=LR\n';

  // Add only reachable nodes, preserving their attributes
  const sortedNodes = nodes
    .filter(node => reachableStates.has(node.id))
    .sort((a, b) => parseInt(a.id) - parseInt(b.id));

  for (const node of sortedNodes) {
    if (node.isInitial || node.isAccepting) {
      const attrs: string[] = [];
      if (node.isInitial) attrs.push('isInitial=true');
      if (node.isAccepting) attrs.push('isAccepting=true');
      result += `  ${node.id} [${attrs.join(', ')}];\n`;
    } else {
      result += `  ${node.id};\n`;
    }
  }

  result += '\n';

  // Add only edges between reachable states
  for (const edge of edges) {
    if (reachableStates.has(edge.from) && reachableStates.has(edge.to)) {
      result += `  ${edge.from} -> ${edge.to} [label="${edge.label}"];\n`;
    }
  }

  result += '}\n';

  return result;
}
