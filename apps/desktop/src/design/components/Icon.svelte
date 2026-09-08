<script>
  // Lucide glyph as a CSS mask so it inherits currentColor. Loaded from the
  // CDN per the design system's own iconography rule (readme.md) — this is
  // a substitution (no icon assets were provided), flagged there for
  // swapping if OpenApps ever ships its own set.
  const BASE = "https://cdn.jsdelivr.net/npm/lucide-static@latest/icons/";

  let { name = "circle", size = 16, color = "", style = "" } = $props();
  const url = $derived(`url("${BASE}${name}.svg")`);
</script>

<span
  aria-hidden="true"
  class="oa-icon"
  style="width:{size}px;height:{size}px;background:{color || 'currentColor'};-webkit-mask-image:{url};mask-image:{url};{style}"
></span>

<style>
  .oa-icon {
    display: inline-block;
    flex: 0 0 auto;
    -webkit-mask-size: contain;
    mask-size: contain;
    -webkit-mask-repeat: no-repeat;
    mask-repeat: no-repeat;
    -webkit-mask-position: center;
    mask-position: center;
  }
</style>
