<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import Graphviz from '$lib/components/Graphviz.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('RegexToDfa', { regex: '' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToDfa" code="regex" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    {console.log(output.dot)}
    <div class="relative">
      <div class="absolute inset-0 grid overflow-auto">
        <Graphviz dot={output.dot || ''} />
      </div>
    </div>
  {/snippet}
</Env>
