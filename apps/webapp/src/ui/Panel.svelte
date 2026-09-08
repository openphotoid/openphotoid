<script>
  import Icon from "./Icon.svelte";

  let { title = "", icon = "", open = $bindable(false), hint = "", children } = $props();
</script>

<section class="panel" class:open>
  <button class="head" onclick={() => (open = !open)} aria-expanded={open}>
    {#if icon}<Icon name={icon} size={17} />{/if}
    <span class="title">{title}</span>
    <Icon name="down" size={16} style="transition:transform 160ms;{open ? 'transform:rotate(180deg)' : ''}" />
  </button>
  {#if open}
    <div class="body">
      {#if hint}<p class="hint">{hint}</p>{/if}
      {@render children?.()}
    </div>
  {/if}
</section>

<style>
  .panel {
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-lg);
    background: var(--surface-card);
    overflow: hidden;
  }
  .head {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-4);
    background: none;
    border: 0;
    cursor: pointer;
    text-align: left;
    color: var(--text-strong);
  }
  .title {
    flex: 1;
    font-weight: var(--weight-medium);
  }
  .body {
    padding: 0 var(--space-4) var(--space-4);
    display: grid;
    gap: var(--space-4);
    animation: oa-fade-up 160ms var(--ease-standard);
  }
  .hint {
    font: var(--type-caption);
    color: var(--text-muted);
    line-height: 1.5;
  }
</style>
