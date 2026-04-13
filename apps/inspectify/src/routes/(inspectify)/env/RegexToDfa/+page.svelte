<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import Graphviz from '$lib/components/Graphviz.svelte';
  import CompactLabelsInput from '$lib/components/CompactLabelsInput.svelte';
  import CopyMinimizerButton from '$lib/components/CopyMinimizerButton.svelte';
  import { Io } from '$lib/io.svelte';
  import { parseAndCompactDot, parseAndRemoveUnreachableStates } from '$lib/utils/dotHandler';
  import { convertDotToMinimizer } from '$lib/utils/convertDotForMinimizer';

  const io = new Io('RegexToDfa', { regex: '' });
  let mode = $state<'none' | 'compact' | 'extreme'>('compact');

  function copyMinimizerFormat(dotString: string) {
    try {
      const minimizerFormat = convertDotToMinimizer(dotString);
      navigator.clipboard.writeText(minimizerFormat);
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  }
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToDfa" code="regex" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    {console.log(output.dot)}
    <div class="flex h-full flex-col gap-4">
      <div class="p-4">
        <CompactLabelsInput bind:mode />
      </div>
      <div class="relative flex-1">
        <div class="absolute inset-0 grid overflow-auto">
          <Graphviz
            dot={mode === 'none'
              ? output.dot || ''
              : mode === 'compact'
                ? parseAndCompactDot(output.dot || '')
                : parseAndRemoveUnreachableStates(output.dot || '')}
          />
        </div>
        <div class="absolute bottom-4 left-4 z-10">
          <CopyMinimizerButton
            label="Copy Minimizer Format"
            description="Export automaton in minimizer format"
            onclick={() => copyMinimizerFormat(output.dot || '')}
            showWhen={mode === 'none'}
          />
        </div>
      </div>
    </div>
  {/snippet}
</Env>
