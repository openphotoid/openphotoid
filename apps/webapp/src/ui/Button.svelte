<script>
  import Icon from "./Icon.svelte";

  let {
    variant = "primary", // primary | secondary | ghost | danger
    size = "md", // sm | md | lg
    icon = "",
    loading = false,
    disabled = false,
    full = false,
    type = "button",
    onclick = undefined,
    children,
    ...rest
  } = $props();

  const off = $derived(disabled || loading);
  const iconSize = $derived(size === "sm" ? 14 : size === "lg" ? 19 : 16);
</script>

<button
  {type}
  class="btn"
  class:full
  data-variant={variant}
  data-size={size}
  disabled={off}
  {onclick}
  {...rest}
>
  {#if loading}
    <Icon name="loader" size={iconSize} style="animation:oa-spin 900ms linear infinite" />
  {:else if icon}
    <Icon name={icon} size={iconSize} />
  {/if}
  {@render children?.()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5em;
    border: var(--border-width) solid transparent;
    border-radius: var(--radius-md);
    font-weight: var(--weight-medium);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-standard),
      color var(--duration-fast) var(--ease-standard),
      border-color var(--duration-fast) var(--ease-standard);
    /* Never below the platform minimum touch target: this app is used
       one-handed on a phone by people who are not enjoying the task. */
    min-height: 44px;
  }
  .btn.full {
    width: 100%;
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .btn[data-size="sm"] {
    min-height: 34px;
    padding: 0 var(--space-3);
    font-size: 0.86rem;
  }
  .btn[data-size="md"] {
    padding: 0 var(--space-4);
    font-size: 0.95rem;
  }
  .btn[data-size="lg"] {
    min-height: 54px;
    padding: 0 var(--space-6);
    font-size: 1.05rem;
  }
  .btn[data-variant="primary"] {
    /* Green means action. This was --gray-950 on --gray-0, layer-1 greys that
       do not flip with the theme: on a dark ground the app's main button was a
       near-black slab on near-black, the least visible thing on the screen. */
    background: var(--brand);
    color: var(--brand-contrast);
  }
  .btn[data-variant="primary"]:hover:not(:disabled) {
    background: var(--brand-strong);
  }
  .btn[data-variant="secondary"] {
    background: var(--surface-card);
    color: var(--text-strong);
    border-color: var(--border-strong);
  }
  .btn[data-variant="secondary"]:hover:not(:disabled) {
    background: var(--bg-sunken);
  }
  .btn[data-variant="ghost"] {
    background: transparent;
    color: var(--text-muted);
  }
  .btn[data-variant="ghost"]:hover:not(:disabled) {
    color: var(--text-strong);
    background: var(--bg-sunken);
  }
  .btn[data-variant="danger"] {
    background: transparent;
    color: var(--red);
    border-color: color-mix(in srgb, var(--red) 40%, transparent);
  }
</style>
