<script>
  let {
    padding = "var(--space-5)",
    interactive = false,
    elevated = false,
    tone = "default", // default | subtle
    style = "",
    children,
    ...rest
  } = $props();
</script>

<div
  class="oa-card"
  class:interactive
  class:elevated
  class:subtle={tone === "subtle"}
  style="padding:{padding};{style}"
  {...rest}
>
  {@render children?.()}
</div>

<style>
  .oa-card {
    display: block;
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-lg);
    transition: var(--transition-control);
  }
  .oa-card.subtle {
    background: var(--bg-subtle);
  }
  .oa-card.elevated {
    box-shadow: var(--shadow-md);
  }
  .oa-card.interactive {
    cursor: pointer;
  }
  .oa-card.interactive:hover {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-sm);
  }
</style>
