<script>
  import Icon from "./Icon.svelte";

  let {
    label = "",
    hint = "",
    value = $bindable(),
    options = [], // [{ value, label }]
    size = "md", // sm | md | lg
    disabled = false,
    onchange = undefined,
  } = $props();
</script>

<label class="oa-select" data-size={size}>
  {#if label}<span class="oa-select-label">{label}</span>{/if}
  <span class="oa-select-control" class:disabled>
    <select bind:value {disabled} {onchange}>
      {#each options as o (o.value)}
        <option value={o.value}>{o.label}</option>
      {/each}
    </select>
    <Icon
      name="chevron-down"
      size={14}
      style="position:absolute;right:9px;color:var(--text-faint);pointer-events:none"
    />
  </span>
  {#if hint}<span class="oa-select-hint">{hint}</span>{/if}
</label>

<style>
  .oa-select {
    display: grid;
    gap: 6px;
    width: 100%;
  }
  .oa-select-label {
    font: var(--type-ui);
    color: var(--text-body);
  }
  .oa-select-control {
    position: relative;
    display: flex;
    align-items: center;
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-xs);
    transition: var(--transition-control);
  }
  .oa-select[data-size="sm"] .oa-select-control {
    height: var(--control-h-sm);
  }
  .oa-select[data-size="md"] .oa-select-control {
    height: var(--control-h-md);
  }
  .oa-select[data-size="lg"] .oa-select-control {
    height: var(--control-h-lg);
  }
  .oa-select-control:focus-within {
    border-color: var(--border-focus);
    box-shadow: var(--focus-ring);
  }
  .oa-select-control.disabled {
    background: var(--bg-subtle);
  }
  .oa-select-control select {
    appearance: none;
    width: 100%;
    height: 100%;
    padding: 0 30px 0 10px;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--weight-regular) var(--text-sm) / 1 var(--font-sans);
    color: var(--text-strong);
    cursor: pointer;
  }
  .oa-select-control.disabled select {
    cursor: not-allowed;
  }
  .oa-select-hint {
    font: var(--type-caption);
    color: var(--text-faint);
  }
</style>
