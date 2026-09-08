<script>
  // No slider primitive exists in the kit (checked components/core and
  // forms/) — styled here to match the system's control conventions:
  // hairline track, card-surface thumb, brand-accent hover, standard
  // focus ring.
  let { value = $bindable(), min = 0, max = 100, step = 1, oninput = undefined, ...rest } = $props();
</script>

<input class="oa-slider" type="range" {min} {max} {step} bind:value {oninput} {...rest} />

<style>
  .oa-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    margin: 6px 0;
    border-radius: var(--radius-full);
    background: var(--bg-sunken);
    outline: none;
    cursor: pointer;
  }
  .oa-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-full);
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-strong);
    box-shadow: var(--shadow-xs);
    transition: var(--transition-control);
  }
  .oa-slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: var(--radius-full);
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-strong);
    box-shadow: var(--shadow-xs);
    transition: var(--transition-control);
  }
  .oa-slider::-moz-range-track {
    height: 4px;
    border-radius: var(--radius-full);
    background: var(--bg-sunken);
  }
  .oa-slider:hover::-webkit-slider-thumb {
    border-color: var(--app-frame);
  }
  .oa-slider:hover::-moz-range-thumb {
    border-color: var(--app-frame);
  }
  .oa-slider:focus-visible {
    box-shadow: var(--focus-ring);
    border-radius: var(--radius-full);
  }
</style>
