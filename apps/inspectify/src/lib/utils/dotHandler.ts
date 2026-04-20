import DotParser from 'dotparser';

export type NodeData = {
  id: string;
  isInitial: boolean;
  isAccepting: boolean;
  additionalAttrs: string; // Store any other attributes as a string
};

export type EdgeData = {
  from: string;
  to: string;
  label: string;
};

export type ParsedAutomaton = {
  nodes: NodeData[];
  edges: EdgeData[];
};

// Helper function to parse nodes and edges from dot format using dotparser library
export function parseNodesAndEdges(dotString: string): ParsedAutomaton {
  try {
    const ast = DotParser(dotString);
    const nodesMap = new Map<string, NodeData>();
    const edges: EdgeData[] = [];

    // The AST is an array with one graph object
    if (!Array.isArray(ast) || ast.length === 0) {
      throw new Error('Invalid DOT format');
    }

    const graph = ast[0];

    // Process the AST children to extract nodes and edges
    if (!Array.isArray(graph.children)) {
      throw new Error('Invalid graph structure');
    }

    for (const stmt of graph.children) {
      if (stmt.type === 'node_stmt') {
        const nodeId = String(stmt.node_id.id);

        let isInitial = false;
        let isAccepting = false;
        const otherAttrs: string[] = [];

        // Parse node attributes
        if (Array.isArray(stmt.attr_list)) {
          for (const attr of stmt.attr_list) {
            if (attr.id === 'isInitial' && attr.eq === 'true') {
              isInitial = true;
            } else if (attr.id === 'isAccepting' && attr.eq === 'true') {
              isAccepting = true;
            } else {
              // Keep other attributes
              otherAttrs.push(`${attr.id}=${attr.eq}`);
            }
          }
        }

        nodesMap.set(nodeId, {
          id: nodeId,
          isInitial,
          isAccepting,
          additionalAttrs: otherAttrs.join(', '),
        });
      } else if (stmt.type === 'edge_stmt') {
        // Extract node IDs from edge_list
        if (!Array.isArray(stmt.edge_list) || stmt.edge_list.length < 2) {
          continue;
        }

        // Get label if present
        let label = '';
        if (Array.isArray(stmt.attr_list)) {
          for (const attr of stmt.attr_list) {
            if (attr.id === 'label') {
              label = String(attr.eq);
              break;
            }
          }
        }

        // Process edges between consecutive nodes
        for (let i = 0; i < stmt.edge_list.length - 1; i++) {
          const fromNode = stmt.edge_list[i];
          const toNode = stmt.edge_list[i + 1];

          // Skip if nodes are not valid node_id type
          if (fromNode.type !== 'node_id' || toNode.type !== 'node_id') {
            continue;
          }

          const from = String(fromNode.id);
          const to = String(toNode.id);

          // Ensure nodes exist in map
          if (!nodesMap.has(from)) {
            nodesMap.set(from, { id: from, isInitial: false, isAccepting: false, additionalAttrs: '' });
          }
          if (!nodesMap.has(to)) {
            nodesMap.set(to, { id: to, isInitial: false, isAccepting: false, additionalAttrs: '' });
          }

          edges.push({ from, to, label });
        }
      }
    }

    return {
      nodes: Array.from(nodesMap.values()),
      edges,
    };
  } catch (error) {
    throw new Error(`Failed to parse DOT format: ${error instanceof Error ? error.message : String(error)}`);
  }
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
  // Sort nodes by ID (try numeric if possible, otherwise lexicographic)
  const sortedNodes = nodes.sort((a, b) => {
    const aNum = parseInt(a.id);
    const bNum = parseInt(b.id);
    if (!isNaN(aNum) && !isNaN(bNum)) {
      return aNum - bNum;
    }
    return a.id.localeCompare(b.id);
  });

  for (const node of sortedNodes) {
    if (node.isInitial || node.isAccepting || node.additionalAttrs) {
      const attrs: string[] = [];
      if (node.isInitial) attrs.push('isInitial=true');
      if (node.isAccepting) attrs.push('isAccepting=true');
      if (node.additionalAttrs) attrs.push(node.additionalAttrs);
      result += `  "${node.id}" [${attrs.join(', ')}];\n`;
    } else {
      result += `  "${node.id}";\n`;
    }
  }

  result += '\n';

  // Add compacted edges
  for (const [key, labels] of edgesMap.entries()) {
    const [from, to] = key.split('->');
    const sortedLabels = Array.from(labels).sort();
    result += `  "${from}" -> "${to}" [label="${sortedLabels.join(',')}"];\n`;
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
    .sort((a, b) => {
      const aNum = parseInt(a.id);
      const bNum = parseInt(b.id);
      if (!isNaN(aNum) && !isNaN(bNum)) {
        return aNum - bNum;
      }
      return a.id.localeCompare(b.id);
    });

  for (const node of sortedNodes) {
    if (node.isInitial || node.isAccepting || node.additionalAttrs) {
      const attrs: string[] = [];
      if (node.isInitial) attrs.push('isInitial=true');
      if (node.isAccepting) attrs.push('isAccepting=true');
      if (node.additionalAttrs) attrs.push(node.additionalAttrs);
      result += `  "${node.id}" [${attrs.join(', ')}];\n`;
    } else {
      result += `  "${node.id}";\n`;
    }
  }

  result += '\n';

  // Add only edges between reachable states
  for (const edge of edges) {
    if (reachableStates.has(edge.from) && reachableStates.has(edge.to)) {
      result += `  "${edge.from}" -> "${edge.to}" [label="${edge.label}"];\n`;
    }
  }

  result += '}\n';

  return result;
}
