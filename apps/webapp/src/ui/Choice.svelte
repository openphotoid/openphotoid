<script>
  /** A row of mutually exclusive pills — a segmented control that wraps. */
  let { value = $bindable(), options = [], onchange = undefined } = $props();
</script>

<div class="choices" role="radiogroup">
  {#each options as o (o.value)}
    <button
      class="choice"
      class:on={value === o.value}
      role="radio"
      aria-checked={value === o.value}
      onclick={() => {
        value = o.value;
        onchange?.(o.value);
      }}
    >
      {#if o.swatch}<span class="swatch" style="background:{o.swatch}"></span>{/if}
      {o.label}
    </button>
  {/each}
</div>

<style>
  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .choice {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 38px;
    padding: 0 var(--space-3);
    border-radius: var(--radius-full);
    border: var(--border-width) solid var(--border-hairline);
    background: var(--surface-card);
    color: var(--text-muted);
    font-size: 0.9rem;
    cursor: pointer;
  }
  .choice.on {
    border-color: var(--gray-950);
    color: var(--text-strong);
    background: var(--bg-sunken);
    font-weight: var(--weight-medium);
  }
  .swatch {
    width: 15px;
    height: 15px;
    border-radius: var(--radius-full);
    border: var(--border-width) solid var(--border-hairline);
  }
</style>
