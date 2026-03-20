<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('RegexToEnfa', { commands: 'skip' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="RegexToEnfa" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}{/snippet}
</Env>
