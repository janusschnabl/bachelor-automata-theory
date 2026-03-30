<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import Graphviz from '$lib/components/Graphviz.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('RegexToNfa', { regex: '' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToNfa" {io} code="regex" />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    <div class="relative">
      <div class="absolute inset-0 overflow-auto">
        <Graphviz dot={output.dot || ''} />
      </div>
    </div>
  {/snippet}
</Env>
