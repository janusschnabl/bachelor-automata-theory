<script lang="ts">
  import { Graphviz } from '@hpcc-js/wasm-graphviz';

  interface Props { dot: string; }
  let { dot }: Props = $props();

  let graphviz: Graphviz | null = $state(null);
  let svg = $state('');

  function enrichDot(dotStr: string): string {
    // Make accepting states double circles with green outline
    let enriched = dotStr.replace(
      /(\d+)\s*\[(.*?isAccepting=true.*?)\];/g,
      (match, node, attrs) => {
        let cleanAttrs = attrs.replace(/isAccepting=true/g, '').trim();
        cleanAttrs = cleanAttrs.replace(/^,\s*/, '').replace(/\s*,$/, '').trim();
        
        if (cleanAttrs) {
          return `${node} [${cleanAttrs}, shape=doublecircle, color="#85CC95", penwidth=2];`;
        } else {
          return `${node} [shape=doublecircle, color="#85CC95", penwidth=2];`;
        }
      }
    );
    
    // Style initial states with default mirage color
    enriched = enriched.replace(
      /(\d+)\s*\[(.*?isInitial=true.*?)\];/g,
      (match, node, attrs) => {
        let cleanAttrs = attrs.replace(/isInitial=true/g, '').trim();
        cleanAttrs = cleanAttrs.replace(/^,\s*/, '').replace(/\s*,$/, '').trim();
        
        if (cleanAttrs) {
          return `${node} [${cleanAttrs}, color="#c9d1d9", penwidth=2];`;
        } else {
          return `${node} [color="#c9d1d9", penwidth=2];`;
        }
      }
    );

    return enriched;
  }

  // Load graphviz once
  $effect(() => {
    Graphviz.load().then(g => {
        graphviz = g;
    });
  });

  // Only re-render when dot changes
  $effect(() => {
    const currentDot = dot;
    if (graphviz && currentDot) {
      svg = graphviz.dot(enrichDot(currentDot));
    } else {
      svg = "";
    }
  });
</script>

<div class="h-full w-full flex items-center justify-center">{@html svg}</div>

<style>
  div :global(svg) {
    max-width: 100%;
    max-height: 100%;
    height: auto;
    width: auto;
    background: transparent;
  }

  div :global(polygon[fill="black"]) {
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

  div :global(polygon[fill="white"]),
  div :global(circle[fill="white"]) {
    fill: transparent;
  }
</style>