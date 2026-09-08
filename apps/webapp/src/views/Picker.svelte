<script>
  import { t, lang } from "$lib/i18n.js";
  import { settings, rememberSpec } from "$lib/settings.js";
  import { byCountry, matches, documentLabel, outputPx, sizeWindow } from "$lib/specs.js";
  import Icon from "$ui/Icon.svelte";

  let { specs = [], onpick } = $props();

  let query = $state("");

  const filtered = $derived(specs.filter((s) => matches(s, query, $lang)));
  const groups = $derived(byCountry(filtered, $lang));
  const recent = $derived(
    $settings.recentSpecs.map((id) => specs.find((s) => s.id === id)).filter(Boolean),
  );

  function pick(spec) {
    rememberSpec(spec.id);
    onpick(spec.id);
  }

  function sizeLine(spec) {
    const [w, h] = outputPx(spec);
    const win = sizeWindow(spec);
    const kb = !win
      ? ""
      : win.min != null && win.max != null
        ? ` · ${win.min}–${win.max} KB`
        : win.max != null
          ? ` · ≤ ${win.max} KB`
          : "";
    const mm = spec.print ? ` · ${spec.print.width_mm}×${spec.print.height_mm} mm` : "";
    return `${w} × ${h} px${mm}${kb}`;
  }
</script>

<div class="stack" style="padding-top:var(--space-5)">
  <h1>{$t("picker.title")}</h1>

  <label class="search">
    <Icon name="globe" size={16} />
    <input
      type="search"
      bind:value={query}
      placeholder={$t("picker.search")}
      autocomplete="off"
      autocapitalize="off"
    />
  </label>

  {#if !query && recent.length}
    <section>
      <h2 class="grouphead">{$t("picker.recent")}</h2>
      <ul class="list">
        {#each recent as spec (spec.id)}
          <li>
            <button onclick={() => pick(spec)}>
              <span class="name">{documentLabel(spec)}</span>
              <span class="meta mono">{sizeLine(spec)}</span>
              <Icon name="right" size={16} />
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#each groups as group (group.code)}
    <section>
      <h2 class="grouphead">{group.name}</h2>
      <ul class="list">
        {#each group.specs as spec (spec.id)}
          <li>
            <button onclick={() => pick(spec)}>
              <span class="name">
                {documentLabel(spec)}
                <span
                  class="dot"
                  style="background:{spec.background.hex_render}"
                  title={spec.background.name}
                ></span>
              </span>
              <span class="meta mono">{sizeLine(spec)}</span>
              {#if spec.last_verified}
                <span class="verified tiny">
                  {$t("picker.verified", { date: spec.last_verified })}
                </span>
              {/if}
              <Icon name="right" size={16} />
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/each}

  {#if !groups.length}
    <p class="muted">{$t("picker.none", { q: query })}</p>
  {/if}
</div>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-4);
    min-height: 48px;
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-strong);
    background: var(--surface-card);
    color: var(--text-faint);
  }
  .search input {
    flex: 1;
    border: 0;
    background: none;
    outline: none;
    min-width: 0;
  }
  .grouphead {
    font: var(--type-eyebrow);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--text-faint);
    margin-bottom: var(--space-2);
  }
  .list {
    list-style: none;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }
  .list button {
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-areas: "name chev" "meta chev" "ver chev";
    align-items: center;
    gap: 2px var(--space-3);
    width: 100%;
    padding: var(--space-3) var(--space-4);
    text-align: left;
    background: var(--surface-card);
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-md);
    cursor: pointer;
    color: var(--text-body);
  }
  .list button:hover {
    border-color: var(--border-strong);
  }
  .name {
    grid-area: name;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-strong);
    font-weight: var(--weight-medium);
  }
  .meta {
    grid-area: meta;
    color: var(--text-muted);
  }
  .verified {
    grid-area: ver;
  }
  .list :global(svg) {
    grid-area: chev;
    color: var(--text-faint);
  }
  .dot {
    width: 11px;
    height: 11px;
    border-radius: var(--radius-full);
    border: var(--border-width) solid var(--border-hairline);
  }
</style>
