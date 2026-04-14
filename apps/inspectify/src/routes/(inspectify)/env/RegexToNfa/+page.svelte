<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import Graphviz from '$lib/components/Graphviz.svelte';
  import CompactLabelsInput from '$lib/components/CompactLabelsInput.svelte';
  import { Io } from '$lib/io.svelte';
  import { parseAndCompactDot } from '$lib/utils/dotHandler';

  const io = new Io('RegexToNfa', { regex: '' });
  let mode = $state<'none' | 'compact'>('none');
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToNfa" {io} code="regex" />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    <div class="flex h-full flex-col gap-4">
      <div class="p-4">
        <CompactLabelsInput bind:mode availableModes={['none', 'compact']} />
      </div>
      <div class="relative flex-1">
        <div class="absolute inset-0 grid overflow-auto">
          <Graphviz
            dot={mode === 'none' ? output.dot || '' : parseAndCompactDot(output.dot || '')}
          />
        </div>
      </div>
    </div>
  {/snippet}
</Env>
