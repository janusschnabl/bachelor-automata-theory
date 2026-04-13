<script lang="ts">
  import { Graphviz } from '@hpcc-js/wasm-graphviz';

  interface Props {
    dot: string;
  }
  let { dot }: Props = $props();

  let graphviz: Graphviz | null = $state(null);
  let svg = $state('');

  function enrichDot(dotStr: string): string {
    let enriched = dotStr;
    let initialNode: string | null = null;

    // Extract and convert accepting states to double circles
    enriched = enriched.replace(/(\w+)\s*\[(.*?isAccepting=true.*?)\];/g, (match, node, attrs) => {
      let cleanAttrs = attrs.replace(/isAccepting=true/g, '').trim();
      cleanAttrs = cleanAttrs.replace(/^,\s*/, '').replace(/\s*,$/, '').trim();

      if (cleanAttrs) {
        return `${node} [${cleanAttrs}, shape=doublecircle];`;
      } else {
        return `${node} [shape=doublecircle];`;
      }
    });

    // Extract initial node and add __start node with arrow
    enriched = enriched.replace(/(\w+)\s*\[(.*?isInitial=true.*?)\];/g, (match, node, attrs) => {
      initialNode = node;
      let cleanAttrs = attrs.replace(/isInitial=true/g, '').trim();
      cleanAttrs = cleanAttrs.replace(/^,\s*/, '').replace(/\s*,$/, '').trim();

      if (cleanAttrs) {
        return `${node} [${cleanAttrs}];`;
      } else {
        return `${node} [];`;
      }
    });

    // Add __start node and arrow if we found an initial state
    if (initialNode) {
      enriched = enriched.replace(
        /(digraph[^{]*{)/,
        `$1\n  rankdir=LR\n  __start [label="", shape=none]\n  __start -> ${initialNode}`,
      );
    }

    return enriched;
  }

  // Load graphviz once
  $effect(() => {
    Graphviz.load().then((g) => {
      graphviz = g;
    });
  });

  // Only re-render when dot changes
  $effect(() => {
    const currentDot = dot;
    if (graphviz && currentDot) {
      svg = graphviz.dot(enrichDot(currentDot));
    } else {
      svg = '';
    }
  });
</script>

<div class="flex h-full w-full items-center justify-center">{@html svg}</div>

<style>
  div :global(svg) {
    max-width: 100%;
    max-height: 100%;
    height: auto;
    width: auto;
    background: transparent;
  }

  div :global(polygon[fill='black']) {
    fill: white;
    stroke: none;
  }

  div :global(ellipse),
  div :global(path) {
    stroke: white;
  }

  div :global(text) {
    fill: white;
  }

  div :global(polygon[fill='white']),
  div :global(circle[fill='white']) {
    fill: transparent;
  }
</style>
