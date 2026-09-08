<script>
  import Icon from "./Icon.svelte";

  let {
    variant = "primary", // primary | secondary | ghost | inverse | danger
    size = "md", // sm | md | lg
    iconLeft = "",
    iconRight = "",
    loading = false,
    disabled = false,
    fullWidth = false,
    type = "button",
    onclick = undefined,
    children,
    ...rest
  } = $props();

  const off = $derived(disabled || loading);
  const iconSize = $derived(size === "sm" ? 14 : size === "lg" ? 18 : 16);
</script>

<button
  {type}
  class="oa-btn"
  class:full={fullWidth}
  data-variant={variant}
  data-size={size}
  disabled={off}
  {onclick}
  {...rest}
>
  {#if loading}
    <Icon name="loader-circle" size={iconSize} style="animation:oa-spin 700ms linear infinite" />
  {:else if iconLeft}
    <Icon name={iconLeft} size={iconSize} />
  {/if}
  {@render children?.()}
  {#if iconRight}
    <Icon name={iconRight} size={iconSize} />
  {/if}
</button>

<style>
  .oa-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border-radius: var(--radius-md);
    border: var(--border-width) solid transparent;
    font: var(--weight-medium) 14px/1 var(--font-sans);
    letter-spacing: var(--tracking-body);
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    transition: var(--transition-control);
    text-decoration: none;
  }
  .oa-btn.full {
    display: flex;
    width: 100%;
  }
  .oa-btn[data-size="sm"] {
    height: var(--control-h-sm);
    padding: 0 10px;
    font-size: 13px;
    gap: 6px;
  }
  .oa-btn[data-size="md"] {
    height: var(--control-h-md);
    padding: 0 14px;
  }
  .oa-btn[data-size="lg"] {
    height: var(--control-h-lg);
    padding: 0 20px;
    font-size: 15px;
  }

  .oa-btn[data-variant="primary"] {
    background: var(--brand);
    color: var(--brand-contrast);
  }
  .oa-btn[data-variant="primary"]:hover:not(:disabled) {
    background: var(--green-400);
  }
  .oa-btn[data-variant="primary"]:active:not(:disabled) {
    background: var(--green-600);
    transform: translateY(0.5px);
  }

  .oa-btn[data-variant="secondary"] {
    background: var(--surface-card);
    color: var(--text-strong);
    border-color: var(--border-strong);
    box-shadow: var(--shadow-xs);
  }
  .oa-btn[data-variant="secondary"]:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .oa-btn[data-variant="secondary"]:active:not(:disabled) {
    background: var(--surface-active);
    transform: translateY(0.5px);
  }

  .oa-btn[data-variant="ghost"] {
    background: transparent;
    color: var(--text-body);
  }
  .oa-btn[data-variant="ghost"]:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .oa-btn[data-variant="ghost"]:active:not(:disabled) {
    background: var(--surface-active);
    transform: translateY(0.5px);
  }

  .oa-btn[data-variant="inverse"] {
    background: var(--gray-950);
    color: var(--gray-0);
  }
  .oa-btn[data-variant="inverse"]:hover:not(:disabled) {
    background: var(--gray-800);
  }
  .oa-btn[data-variant="inverse"]:active:not(:disabled) {
    background: var(--gray-900);
    transform: translateY(0.5px);
  }

  .oa-btn[data-variant="danger"] {
    background: var(--red-500);
    color: var(--gray-0);
  }
  .oa-btn[data-variant="danger"]:hover:not(:disabled) {
    background: var(--red-600);
  }
  .oa-btn[data-variant="danger"]:active:not(:disabled) {
    background: var(--red-600);
    transform: translateY(0.5px);
  }

  .oa-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }
  .oa-btn:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }
</style>
