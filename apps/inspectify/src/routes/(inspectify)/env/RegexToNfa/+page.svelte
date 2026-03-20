<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import Network from '$lib/components/NetworkAutomata.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('RegexToNfa', { regex: '' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToNfa" {io} code="regex" />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    {console.log(output.dot)}
    <div class="relative">
      <div class="absolute inset-0 grid overflow-auto">
        <Network dot={output.dot || ''} />
      </div>
    </div>
  {/snippet}
</Env>
