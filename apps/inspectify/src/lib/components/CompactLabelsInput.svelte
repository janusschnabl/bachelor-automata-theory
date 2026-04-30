<script lang="ts">
  import InputOption from './InputOption.svelte';

  let { mode = $bindable(), availableModes = ['none', 'compact', 'extreme'] } = $props<{
    mode: 'none' | 'compact' | 'extreme';
    availableModes?: ('none' | 'compact' | 'extreme')[];
  }>();

  const allOptions = [
    {
      label: 'None',
      value: 'none' as const,
      description: 'Show the original automaton without any modifications',
    },
    {
      label: 'Combine edges',
      value: 'compact' as const,
      description: 'Combine edges with the same source and destination',
    },
    {
      label: 'Remove dead states',
      value: 'extreme' as const,
      description: 'Combine edges and remove dead states, page 67 in the book for more info',
    },
  ];

  const options = allOptions.filter((opt) => availableModes.includes(opt.value));

  let hoveredOption: string | null = $state(null);
  let tooltipTimeout: ReturnType<typeof setTimeout> | null = null;

  function onMouseEnter(value: string) {
    hoveredOption = null;
    tooltipTimeout = setTimeout(() => {
      hoveredOption = value;
    }, 800);
  }

  function onMouseLeave() {
    if (tooltipTimeout) {
      clearTimeout(tooltipTimeout);
      tooltipTimeout = null;
    }
    hoveredOption = null;
  }
</script>

<InputOption title="Compaction options">
  <div class="relative">
    <div class="flex w-full gap-2">
      {#each options as option}
        <button
          class="flex-1 cursor-pointer rounded px-4 py-2 font-mono text-sm transition {mode ===
          option.value
            ? 'bg-slate-500 hover:bg-slate-400'
            : 'bg-slate-800 hover:bg-slate-700'}"
          onclick={() => (mode = option.value)}
          onmouseenter={() => onMouseEnter(option.value)}
          onmouseleave={onMouseLeave}
        >
          {option.label}
        </button>
      {/each}
    </div>
    {#if hoveredOption}
      <div
        class="absolute bottom-full left-1/2 z-50 mb-2 -translate-x-1/2 rounded bg-slate-900 px-3 py-1 text-xs whitespace-nowrap text-slate-200 shadow-lg"
      >
        {allOptions.find((opt) => opt.value === hoveredOption)?.description}
        <div
          class="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-slate-900"
        ></div>
      </div>
    {/if}
  </div>
</InputOption>
