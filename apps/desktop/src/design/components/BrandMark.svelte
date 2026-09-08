<script>
  // The OpenApps mark, and the mark of any product in the family — see
  // assets/logo/README.md. "Open" sets muted, the product word full
  // strength, the trailing period carries the product's accent hue.
  import Icon from "./Icon.svelte";

  let {
    variant = "wordmark", // "wordmark" | "monogram" | "lockup"
    size = 24,
    name = "OpenApps",
    icon = "",
    accent = "",
    dot = true,
  } = $props();

  const isProduct = $derived(name !== "OpenApps" && name.startsWith("Open"));
  const prefix = $derived(isProduct ? "Open" : null);
  const word = $derived(isProduct ? name.slice(4) : name);
  const dotColor = $derived(accent || "var(--logo-dot)");
  const wordSize = $derived(variant === "lockup" ? size * 0.56 : size);
  const gap = $derived(variant === "lockup" ? Math.round(size * 0.28) : 0);
</script>

<span role="img" aria-label={name} class="oa-brandmark" style="gap:{gap}px">
  {#if variant !== "wordmark"}
    <span
      class="oa-brandmark-tile"
      style="width:{size}px;height:{size}px;font-size:{size}px;background:{accent || 'var(--logo-tile-bg)'};color:{accent ? 'var(--white)' : 'var(--logo-tile-fg)'};"
    >
      {#if icon}
        <Icon name={icon} size={Math.round(size * 0.5)} />
      {:else}
        <span style="font:var(--weight-medium) {size * 0.625}px/1 var(--font-display);color:inherit;">O</span>
      {/if}
    </span>
  {/if}
  {#if variant !== "monogram"}
    <span class="oa-brandmark-word" style="font-size:{wordSize}px">
      {#if prefix}<span class="oa-brandmark-prefix">{prefix}</span>{/if}{word}{#if dot}<span
          class="oa-brandmark-dot"
          style="color:{dotColor}">.</span
        >{/if}
    </span>
  {/if}
</span>

<style>
  .oa-brandmark {
    display: inline-flex;
    align-items: center;
  }
  .oa-brandmark-tile {
    display: grid;
    place-items: center;
    flex: 0 0 auto;
    border-radius: var(--logo-tile-radius);
  }
  .oa-brandmark-word {
    font: var(--weight-medium) 1em/1 var(--font-display);
    letter-spacing: var(--logo-tracking);
    color: var(--logo-fg);
    white-space: nowrap;
  }
  .oa-brandmark-prefix {
    color: var(--text-muted);
  }
</style>
