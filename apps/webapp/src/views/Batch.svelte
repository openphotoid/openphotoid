<script>
  import { t, lang } from "$lib/i18n.js";
  import { settings } from "$lib/settings.js";
  import { Photo, defaultOptions, loadModels } from "$lib/pipeline.js";
  import { backgroundRgb, documentLabel, countryName, sizeWindow, matches } from "$lib/specs.js";
  import { save, readFile, objectUrl, filename } from "$lib/download.js";
  import Button from "$ui/Button.svelte";
  import Icon from "$ui/Icon.svelte";

  let { specs = [], go } = $props();

  /*
    Batch is free and uncapped, which is the point: every competitor in
    the research either limits the free tier to a handful of photos or
    sells monthly export quotas. Doing the work on the device makes a cap
    meaningless — there is no per-photo cost to recover.

    Photos are processed one at a time on purpose. Running them
    concurrently would multiply peak wasm heap by the concurrency and take
    a mobile browser down on a large set; sequential keeps memory flat and
    the progress count honest.
  */
  let specId = $state($settings.recentSpecs[0] ?? "");
  let files = $state([]);
  let results = $state([]);
  let running = $state(false);
  let done = $state(0);
  let query = $state("");
  let input;

  const spec = $derived(specs.find((s) => s.id === specId) ?? null);
  const options = $derived(
    spec ? { ...defaultOptions(), backdrop: { kind: "solid", rgb: backgroundRgb(spec) }, retouch: $settings.retouch } : null,
  );
  const filteredSpecs = $derived(specs.filter((s) => matches(s, query, $lang)).slice(0, 40));

  function choose(e) {
    files = [...(e.currentTarget.files ?? [])];
    results = [];
    done = 0;
    e.currentTarget.value = "";
  }

  async function run() {
    if (!spec || !files.length) return;
    running = true;
    done = 0;
    results = [];
    try {
      await loadModels();
      for (const file of files) {
        let photo = null;
        try {
          photo = await Photo.open(await readFile(file), { quality: $settings.quality });
          if (!photo.faceFound) throw new Error($t("studio.noface"));
          const { report } = await photo.render(spec.id, options, {}, { enhance: $settings.sharpen });
          const win = sizeWindow(spec);
          const bytes = win
            ? photo.outputJpegWithin(win.min ?? 0, win.max ?? 10_000)
            : photo.outputJpeg(95);
          results.push({
            name: file.name,
            url: objectUrl(bytes),
            bytes,
            overall: report.overall,
            checks: report.checks,
          });
        } catch (e) {
          results.push({ name: file.name, error: e.message ?? String(e) });
        } finally {
          photo?.free();
          done += 1;
          // Yield so the count and thumbnails actually paint between
          // photos rather than all at once when the loop ends.
          await new Promise((r) => setTimeout(r, 0));
        }
      }
    } finally {
      running = false;
    }
  }

  function saveAll() {
    results.forEach((r, i) => {
      if (r.bytes) save(r.bytes, filename(spec.id, "jpg", i), "image/jpeg");
    });
  }

  /**
   * The same per-item CSV the desktop batch writes, so a school or an
   * agency processing a group has a record of which photos passed.
   */
  function saveReport() {
    const esc = (v) => `"${String(v).replaceAll('"', '""')}"`;
    const names = results.find((r) => r.checks)?.checks.map((c) => c.name) ?? [];
    const rows = [
      ["file", "overall", "detail", ...names].map(esc).join(","),
      ...results.map((r) =>
        [
          r.name,
          r.error ? "error" : r.overall,
          r.error ?? "",
          // A row that never got as far as being checked has no per-check
          // result. Leaving those blank keeps the columns meaning what
          // their headers say; repeating the failure message across all
          // eleven of them makes the file unreadable in a spreadsheet.
          ...names.map((n) => r.checks?.find((c) => c.name === n)?.status ?? ""),
        ]
          .map(esc)
          .join(","),
      ),
    ];
    save(new TextEncoder().encode(rows.join("\n")), `${spec.id}-report.csv`, "text/csv");
  }

  function clear() {
    results.forEach((r) => r.url && URL.revokeObjectURL(r.url));
    files = [];
    results = [];
    done = 0;
  }
</script>

<div class="stack" style="padding-top:var(--space-5)">
  <header>
    <h1>{$t("batch.title")}</h1>
    <p class="oa-lead">{$t("batch.hint")}</p>
  </header>

  <div class="card stack">
    <label class="search">
      <Icon name="globe" size={16} />
      <input type="search" bind:value={query} placeholder={$t("picker.search")} />
    </label>
    <label class="pickspec">
      <span class="sr-only">{$t("picker.title")}</span>
      <select bind:value={specId}>
        <option value="" disabled>{$t("picker.title")}</option>
        {#each filteredSpecs as s (s.id)}
          <option value={s.id}>
            {countryName(s.country, $lang)} — {documentLabel(s)}
          </option>
        {/each}
      </select>
    </label>
  </div>

  <input bind:this={input} type="file" accept="image/*" multiple class="sr-only" onchange={choose} />

  <div class="stack">
    <Button variant="secondary" icon="files" full onclick={() => input.click()}>
      {$t("batch.choose")}
    </Button>
    {#if files.length}
      <Button
        full
        icon="layers"
        loading={running}
        disabled={!spec || running}
        onclick={run}
      >
        {running ? $t("batch.running", { done, total: files.length }) : $t("batch.run", { n: files.length })}
      </Button>
    {/if}
  </div>

  {#if results.length}
    <ul class="results">
      {#each results as r, i (r.name + i)}
        <li class:failed={!!r.error}>
          {#if r.url}
            <img src={r.url} alt={r.name} />
          {:else}
            <div class="ph checkerboard"></div>
          {/if}
          <div class="info">
            <strong>{r.name}</strong>
            {#if r.error}
              <span class="tiny status-fail">{$t("batch.failed")} — {r.error}</span>
            {:else}
              <span class="tiny status-{r.overall}">
                {r.overall === "pass"
                  ? $t("studio.checks.pass")
                  : r.overall === "warn"
                    ? $t("studio.checks.warn")
                    : $t("studio.checks.fail")}
              </span>
            {/if}
          </div>
          {#if r.bytes}
            <Button
              size="sm"
              variant="ghost"
              icon="download"
              onclick={() => save(r.bytes, filename(spec.id, "jpg", i), "image/jpeg")}
            >
              {$t("batch.downloadOne")}
            </Button>
          {/if}
        </li>
      {/each}
    </ul>

    <div class="actions">
      <Button icon="download" onclick={saveAll}>{$t("batch.download")}</Button>
      <Button variant="secondary" onclick={saveReport}>{$t("batch.report")}</Button>
      <Button variant="ghost" onclick={clear}>{$t("batch.clear")}</Button>
    </div>
  {/if}
</div>

<style>
  .search,
  .pickspec select {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-4);
    min-height: 46px;
    width: 100%;
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-strong);
    background: var(--surface-card);
    color: var(--text-body);
  }
  .search input {
    flex: 1;
    border: 0;
    background: none;
    outline: none;
    min-width: 0;
  }
  .results {
    list-style: none;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }
  .results li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-md);
    background: var(--surface-card);
  }
  .results li.failed {
    border-color: color-mix(in srgb, var(--red) 30%, transparent);
  }
  .results img,
  .ph {
    width: 44px;
    height: 56px;
    object-fit: cover;
    border-radius: var(--radius-sm);
    flex: 0 0 auto;
  }
  .info {
    flex: 1;
    min-width: 0;
    display: grid;
    gap: 1px;
  }
  .info strong {
    font-weight: var(--weight-medium);
    font-size: 0.9rem;
    color: var(--text-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
