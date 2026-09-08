<script>
  let {
    tone = "neutral", // neutral | brand | success | warning | danger | info
    dot = false,
    size = "md", // sm | md
    children,
    ...rest
  } = $props();
</script>

<span class="oa-badge" data-tone={tone} class:sm={size === "sm"} {...rest}>
  {#if dot}<span class="oa-badge-dot"></span>{/if}
  {@render children?.()}
</span>

<style>
  .oa-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 0 8px;
    font: var(--weight-medium) var(--text-xs) / 1 var(--font-sans);
    border-radius: var(--radius-full);
    white-space: nowrap;
  }
  .oa-badge.sm {
    height: 18px;
    padding: 0 6px;
    font-size: var(--text-2xs);
  }
  .oa-badge-dot {
    width: 5px;
    height: 5px;
    border-radius: var(--radius-full);
    background: currentColor;
    flex: 0 0 auto;
  }
  .oa-badge[data-tone="neutral"] {
    background: var(--bg-sunken);
    color: var(--text-muted);
  }
  .oa-badge[data-tone="brand"] {
    background: var(--brand-soft);
    color: var(--text-brand);
  }
  .oa-badge[data-tone="success"] {
    background: var(--success-bg);
    color: var(--success-fg);
  }
  .oa-badge[data-tone="warning"] {
    background: var(--warning-bg);
    color: var(--warning-fg);
  }
  .oa-badge[data-tone="danger"] {
    background: var(--danger-bg);
    color: var(--danger-fg);
  }
  .oa-badge[data-tone="info"] {
    background: var(--info-bg);
    color: var(--info-fg);
  }
</style>
