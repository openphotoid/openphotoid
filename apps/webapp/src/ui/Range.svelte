<script>
  let {
    label = "",
    value = $bindable(0),
    min = 0,
    max = 100,
    step = 1,
    format = (v) => String(v),
    oninput = undefined,
  } = $props();
</script>

<label class="range">
  <span class="row">
    <span class="label">{label}</span>
    <span class="value">{format(value)}</span>
  </span>
  <input type="range" {min} {max} {step} bind:value {oninput} />
</label>

<style>
  .range {
    display: grid;
    gap: var(--space-1);
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: var(--space-3);
  }
  .label {
    font-size: 0.92rem;
    color: var(--text-body);
  }
  .value {
    font: var(--type-mono);
    font-size: 0.82rem;
    color: var(--text-muted);
  }
  input {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: var(--radius-full);
    background: var(--bg-sunken);
    outline: none;
    cursor: pointer;
  }
  input::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-strong);
    box-shadow: var(--shadow-sm);
  }
  input::-moz-range-thumb {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-strong);
  }
</style>
