<script lang="ts">
  import { mirage } from 'ayu';
  import { onMount } from 'svelte';
  import type { Network } from 'vis-network/esnext';

  interface Props {
    dot: string;
  }

  let { dot }: Props = $props();
  let container: HTMLDivElement | undefined = $state();
  let network: Network | undefined = $state();

  let redraw = $derived(async () => {
    let preDot = dot;
    const vis = await import('vis-network/esnext');
    if (preDot != dot) return;

    const data = vis.parseDOTNetwork(dot);

    data.nodes.forEach((node: any) => {
      if (node.isAccepting) {
        const outerR = 15;
        const innerR = 11;
        const bg = mirage.ui.fg.hex();
        const borderColor = '#85CC95';
        const highlightBg = mirage.ui.fg.brighten(1).hex();
        const label = node.label ?? '';

        node.shape = 'custom';
        node.ctxRenderer = ({ ctx, x, y, selected, hover }: any) => {
          return {
            drawNode() {
              const currentBg = selected || hover ? highlightBg : bg;

              // Outer circle fill
              ctx.beginPath();
              ctx.arc(x, y, outerR, 0, Math.PI * 2);
              ctx.fillStyle = currentBg;
              ctx.fill();

              // Outer circle stroke
              ctx.strokeStyle = borderColor;
              ctx.lineWidth = selected ? 2 : 1;
              ctx.stroke();

              // Inner circle stroke (double ring effect)
              ctx.beginPath();
              ctx.arc(x, y, innerR, 0, Math.PI * 2);
              ctx.strokeStyle = borderColor;
              ctx.lineWidth = selected ? 1.5 : 1;
              ctx.stroke();

              // Label
              if (label) {
                ctx.fillStyle = 'white';
                ctx.font = '14px Menlo, Monaco, "Courier New", monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText(label, x, y);
              }
            },
            nodeDimensions: { width: outerR * 2, height: outerR * 2 },
          };
        };
      }
    });

    if (network) {
      network.setData(data);
    } else {
      if (!container) return;
      network = new vis.Network(container, data, {
        // interaction: { zoomView: false },
        nodes: {
          color: {
            background: mirage.ui.fg.hex(),
            border: mirage.ui.fg.hex(),
            highlight: mirage.ui.fg.brighten(1).hex(),
            // background: '#666666',
            // border: '#8080a0',
            // highlight: '#80a0ff',
          },
          font: {
            color: 'white',
          },
          borderWidth: 1,
          shape: 'circle',
          size: 30,
        },
        edges: {
          // color: '#D0D0FF',
          color: mirage.syntax.constant.hex(),
          font: {
            color: 'white',
            strokeColor: '#200020',
            face: 'Menlo, Monaco, "Courier New", monospace',
          },
        },
        autoResize: true,
      });
    }
  });

  onMount(() => {
    if (!container) return;
    const observer = new ResizeObserver(() => {
      requestAnimationFrame(() => {
        if (network) {
          network.fit({ animation: false, maxZoomLevel: 20 });
          network.redraw();
        }
      });
    });
    observer.observe(container);
    return () => observer?.disconnect();
  });

  onMount(() => {
    redraw();
  });

  $effect(() => {
    dot && network && redraw();
  });
</script>

<div class="relative h-full w-full">
  <div class="absolute inset-0" bind:this={container}></div>
</div>
