<script lang="ts">
  import InputOption from './InputOption.svelte';

  interface Props {
    label?: string;
    description?: string;
    onclick: () => void;
    showWhen?: boolean;
  }

  let { label = 'Copy', description = '', onclick, showWhen = true }: Props = $props();

  let hoveredOption: boolean = $state(false);
  let tooltipTimeout: ReturnType<typeof setTimeout> | null = null;

  function onMouseEnter() {
    hoveredOption = false;
    tooltipTimeout = setTimeout(() => {
      hoveredOption = true;
    }, 800);
  }

  function onMouseLeave() {
    if (tooltipTimeout) {
      clearTimeout(tooltipTimeout);
      tooltipTimeout = null;
    }
    hoveredOption = false;
  }
</script>

{#if showWhen}
  <InputOption title="">
    <div class="relative">
      <button
        class="w-full rounded bg-slate-500 px-4 py-2 font-mono text-sm transition hover:bg-slate-400 cursor-pointer" 
        {onclick}
      >
        {label}
      </button>
    </div>
  </InputOption>
{/if}
