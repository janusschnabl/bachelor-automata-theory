<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import Network from '$lib/components/NetworkAutomata.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('RegexToDfa', { regex: 'skip' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToDfa" code="regex" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
  {console.log(output.dot)}
  <div class="relative">
      <pre>{output.dot || ''} </pre>
    </div>
    {/snippet}
</Env>
