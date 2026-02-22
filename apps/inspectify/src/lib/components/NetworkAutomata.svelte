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
  let selectedNodes = $state(new Set<string>());

  let redraw = $derived(async () => {
    let preDot = dot;
    const vis = await import('vis-network/esnext');
    if (preDot != dot) return;

    const data = vis.parseDOTNetwork(dot);

    data.nodes.forEach((node: any) => {
      const outerR = 15;
      const isAccepting = node.isAccepting;
      const innerR = 11;
      const bg = mirage.ui.fg.hex();
      const highlightBg = mirage.ui.fg.brighten(1).hex();
      const borderColor = isAccepting ? '#85CC95' : mirage.ui.fg.hex();
      const label = node.label ?? '';
      const nodeId = node.id;

      // Make the double circles by hand
      node.shape = 'custom';
      node.ctxRenderer = ({ ctx, x, y }: any) => {
        return {
          drawNode() {
            const selected = selectedNodes.has(String(nodeId));
            const currentBg = selected ? highlightBg : bg;
            const currentBorder = selected ? '#ffffff' : borderColor;

            if (isAccepting) {
              ctx.shadowColor = '#85CC95';
              ctx.shadowBlur = selected ? 18 : 10;
            }

            ctx.beginPath();
            ctx.arc(x, y, outerR, 0, Math.PI * 2);
            ctx.fillStyle = currentBg;
            ctx.fill();
            ctx.strokeStyle = currentBorder;
            ctx.lineWidth = selected ? 2.5 : 1;
            ctx.stroke();

            ctx.shadowBlur = 0;

            if (isAccepting) {
              ctx.beginPath();
              ctx.arc(x, y, innerR, 0, Math.PI * 2);
              ctx.strokeStyle = currentBorder;
              ctx.lineWidth = selected ? 1.5 : 1;
              ctx.stroke();
            }

            if (label) {
              ctx.fillStyle = 'white';
              ctx.font = `${selected ? 'bold ' : ''}14px Menlo, Monaco, "Courier New", monospace`;
              ctx.textAlign = 'center';
              ctx.textBaseline = 'middle';
              ctx.fillText(label, x, y);
            }
          },
          nodeDimensions: { width: outerR * 2, height: outerR * 2 },
        };
      };
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

      network.on('selectNode', (e) => {
        selectedNodes = new Set(e.nodes.map(String));
        network?.redraw();
      });

      network.on('deselectNode', () => {
        selectedNodes = new Set();
        network?.redraw();
      });

      network.on('dragStart', (e) => {
        if (e.nodes.length > 0) {
          selectedNodes = new Set(e.nodes.map(String));
          network?.redraw();
        }
      });

      network.on('dragEnd', (e) => {
        if (e.nodes.length === 0) {
          selectedNodes = new Set();
          network?.redraw();
        }
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
    return () => {
      network?.destroy();
      network = undefined;
    };
  });

  $effect(() => {
    dot && network && redraw();
  });
</script>

<div class="relative h-full w-full">
  <div class="absolute inset-0" bind:this={container}></div>
</div>