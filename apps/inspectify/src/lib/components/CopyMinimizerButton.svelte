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
        class="w-full rounded bg-slate-500 px-4 py-2 font-mono text-sm transition hover:bg-slate-400"
        {onclick}
        onmouseenter={onMouseEnter}
        onmouseleave={onMouseLeave}
      >
        {label}
      </button>
      {#if hoveredOption && description}
        <div
          class="absolute bottom-full left-8 z-50 mb-2 rounded bg-slate-900 px-3 py-1 text-xs whitespace-nowrap text-slate-200 shadow-lg"
        >
          {description}
          <div
            class="absolute top-full left-0 border-4 border-transparent border-t-slate-900"
          ></div>
        </div>
      {/if}
    </div>
  </InputOption>
{/if}
