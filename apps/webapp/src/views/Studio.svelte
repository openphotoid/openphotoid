<script>
  import { t, lang } from "$lib/i18n.js";
  import { settings, rememberSpec } from "$lib/settings.js";
  import { Photo, defaultOptions, loadModels } from "$lib/pipeline.js";
  import { backgroundRgb, documentLabel, countryName, outputPx, sizeWindow } from "$lib/specs.js";
  import { save, readFile, objectUrl, filename } from "$lib/download.js";
  import Button from "$ui/Button.svelte";
  import Icon from "$ui/Icon.svelte";
  import Panel from "$ui/Panel.svelte";
  import Range from "$ui/Range.svelte";
  import Choice from "$ui/Choice.svelte";

  let { specs = [], specId = "", file = null, go, setPicked } = $props();

  const spec = $derived(specs.find((s) => s.id === specId) ?? null);

  let photo = $state(null);
  /*
    The crop solution and the checklist live here rather than on the Photo
    instance, even though that is where they are produced.

    `$state` deep-proxies plain objects and arrays but not class
    instances, so `report = …` inside a method updates the object
    and notifies nobody: the pipeline ran correctly and the checklist
    simply never appeared. Holding them as component state makes the
    dependency explicit, and is why `render()` returns them.
  */
  let solve = $state(null);
  let report = $state(null);
  let stage = $state("");
  let progress = $state(0);
  let error = $state(null);
  let busy = $state(false);
  let resultUrl = $state("");
  let sourceUrl = $state("");
  let saved = $state("");

  // --- the settings that drive render() -------------------------------
  let options = $state(defaultOptions());
  let adjust = $state({ head: NaN, eye: NaN, center: NaN });
  let backdropKind = $state("solid");
  let backdropBlur = $state(8);
  let garmentStyle = $state("none");
  let garmentColors = $state({ jacket: [38, 41, 48], shirt: [244, 246, 249], tie: [30, 52, 92] });
  let sheet = $state("4x6");
  let sheetCount = $state(null);

  let fileInput;
  let backdropInput;

  const hex = (rgb) => "#" + rgb.map((v) => v.toString(16).padStart(2, "0")).join("");

  /*
    Load and run the two model stages. Split from render() because this is
    the expensive half: everything the user can drag afterwards re-runs
    render() alone, which touches no model.
  */
  async function open(f) {
    if (!f || !spec) return;
    busy = true;
    error = null;
    stage = "decoding";
    try {
      await loadModels(({ fraction }) => (progress = fraction));
      const bytes = await readFile(f);
      photo?.free();
      solve = null;
      report = null;
      photo = await Photo.open(bytes, {
        quality: $settings.quality,
        onStage: (s) => (stage = s),
      });
      if (sourceUrl) URL.revokeObjectURL(sourceUrl);
      sourceUrl = objectUrl(photo.sourceJpeg(80));
      if (!photo.faceFound) {
        error = $t("studio.noface");
        return;
      }
      // The document's own background colour is what the compliance path
      // needs, so it is the starting point rather than a choice to make.
      options.backdrop = { kind: "solid", rgb: backgroundRgb(spec) };
      backdropKind = "solid";
      rememberSpec(spec.id);
      await render();
    } catch (e) {
      error = e.message ?? String(e);
    } finally {
      busy = false;
      stage = "";
    }
  }

  /*
    The cheap half — re-run on every control change.

    A slider fires input events faster than a render completes, and each
    render awaits a face re-detection on the output. Without coalescing,
    a drag queued a render per event and the preview kept catching up for
    seconds after the thumb stopped. So: one render in flight at a time,
    and a change that arrives during it is folded into exactly one more.
  */
  let rendering = false;
  let renderAgain = false;
  async function render() {
    if (!photo || !spec || !photo.faceFound) return;
    if (rendering) {
      renderAgain = true;
      return;
    }
    rendering = true;
    busy = true;
    stage = "rendering";
    try {
      options.retouch = $settings.retouch;
      options.garment =
        garmentStyle === "none"
          ? null
          : {
              style: garmentStyle,
              jacket: garmentColors.jacket,
              shirt: garmentColors.shirt,
              tie: garmentColors.tie,
            };
      ({ solve, report } = await photo.render(spec.id, options, adjust, {
        enhance: $settings.sharpen,
      }));
      if (resultUrl) URL.revokeObjectURL(resultUrl);
      resultUrl = objectUrl(await photo.outputJpeg(92));
      sheetCount = spec.print ? (await photo.sheetInfo(sheet)).count : null;
      error = null;
    } catch (e) {
      error = e.message ?? String(e);
    } finally {
      busy = false;
      stage = "";
      rendering = false;
      if (renderAgain) {
        renderAgain = false;
        render();
      }
    }
  }

  // Kick off as soon as both a file and a spec exist.
  $effect(() => {
    if (file && spec && !photo) open(file);
  });

  $effect(() => () => {
    if (resultUrl) URL.revokeObjectURL(resultUrl);
    if (sourceUrl) URL.revokeObjectURL(sourceUrl);
    photo?.free();
  });

  function pickAnother(e) {
    const f = e.currentTarget.files?.[0];
    if (!f) return;
    e.currentTarget.value = "";
    setPicked(f);
    photo?.free();
    photo = null;
    open(f);
  }

  async function pickBackdrop(e) {
    const f = e.currentTarget.files?.[0];
    if (!f || !photo) return;
    e.currentTarget.value = "";
    await photo.setBackdropImage(await readFile(f));
    backdropKind = "image";
    options.backdrop = { kind: "image", blur: backdropBlur };
    await render();
  }

  function setBackdrop(kind) {
    backdropKind = kind;
    const doc = backgroundRgb(spec);
    options.backdrop =
      kind === "solid"
        ? { kind: "solid", rgb: doc }
        : kind === "gradient"
          ? { kind: "gradient", top: [235, 238, 242], bottom: [176, 185, 196] }
          : kind === "vignette"
            ? { kind: "vignette", center: [232, 234, 236], edge: [138, 145, 152], center_y: 0.35 }
            : kind === "image"
              ? { kind: "image", blur: backdropBlur }
              : { kind: "keep" };
    if (kind === "image") backdropInput.click();
    else render();
  }

  function resetAdjust() {
    adjust = { head: NaN, eye: NaN, center: NaN };
    render();
  }

  // --- saving ---------------------------------------------------------
  function flash(what) {
    saved = what;
    setTimeout(() => (saved = ""), 1800);
  }

  async function saveDigital() {
    const win = sizeWindow(spec);
    const bytes = win
      ? await photo.outputJpegWithin(win.min ?? 0, win.max ?? 10_000)
      : await photo.outputJpeg(95);
    save(bytes, filename(spec.id, "jpg"), "image/jpeg");
    flash("digital");
  }
  async function savePng() {
    save(await photo.outputPng(), filename(spec.id, "png"), "image/png");
    flash("png");
  }
  async function saveSheet() {
    const dpi = spec.print?.dpi_default ?? 300;
    save(await photo.sheetJpeg(sheet, dpi, true, 95), `${spec.id}-sheet-${sheet}.jpg`, "image/jpeg");
    flash("sheet");
  }

  const digitalHint = $derived.by(() => {
    if (!spec) return "";
    const [w, h] = outputPx(spec);
    const win = sizeWindow(spec);
    const size = !win
      ? ""
      : win.min != null && win.max != null
        ? $t("export.digital.window", { min: win.min, max: win.max })
        : win.max != null
          ? $t("export.digital.max", { max: win.max })
          : "";
    return $t("export.digital.hint", { w, h, size });
  });

  const overall = $derived(report?.overall ?? "");
  const SHEETS = [
    { value: "4x6", label: '4×6"' },
    { value: "5x7", label: '5×7"' },
    { value: "a4", label: "A4" },
    { value: "letter", label: "Letter" },
  ];
</script>

<input bind:this={fileInput} type="file" accept="image/*" class="sr-only" onchange={pickAnother} />
<input
  bind:this={backdropInput}
  type="file"
  accept="image/*"
  class="sr-only"
  onchange={pickBackdrop}
/>

{#if !spec}
  <div class="card" style="margin-top:var(--space-6)">
    <p class="muted">{$t("picker.none", { q: specId })}</p>
    <Button variant="secondary" onclick={() => go("pick")}>{$t("picker.title")}</Button>
  </div>
{:else}
  <div class="stack" style="padding-top:var(--space-5)">
    <header class="dochead">
      <div>
        <h1>{documentLabel(spec)}</h1>
        <p class="muted">
          {countryName(spec.country, $lang)} ·
          {$t("picker.background", { name: spec.background.name })}
        </p>
      </div>
      <Button size="sm" variant="ghost" onclick={() => go("pick")}>{$t("studio.changeDoc")}</Button>
    </header>

    <!-- Preview -->
    <div class="preview">
      {#if resultUrl}
        <img src={resultUrl} alt={$t("studio.after")} />
      {:else if sourceUrl}
        <img src={sourceUrl} alt={$t("studio.before")} class="dim" />
      {:else}
        <div class="placeholder checkerboard"></div>
      {/if}

      {#if busy}
        <div class="veil">
          <Icon name="loader" size={22} style="animation:oa-spin 900ms linear infinite" />
          <span>
            {stage ? $t(`studio.stage.${stage}`) : $t("studio.working")}
            {#if progress > 0 && progress < 1}
              — {$t("common.downloading", { pct: Math.round(progress * 100) })}
            {/if}
          </span>
        </div>
      {/if}
    </div>

    {#if error}
      <div class="card notice">
        <Icon name="alert" size={18} />
        <div>
          <p>{error}</p>
          <Button size="sm" variant="secondary" onclick={() => fileInput.click()}>
            {$t("studio.retry")}
          </Button>
        </div>
      </div>
    {/if}

    {#if solve?.padded}
      <p class="tiny padded"><Icon name="info" size={14} /> {$t("studio.padded")}</p>
    {/if}

    <!-- Checklist. Status is never carried by colour alone — a red dot and
         a green dot are the same dot to a colourblind user, and this list
         is the whole reason to trust the result. -->
    {#if report}
      <section class="card">
        <header class="checkhead">
          <h2>{$t("studio.checks")}</h2>
          <span class="status-{overall}">
            {overall === "pass"
              ? $t("studio.checks.pass")
              : overall === "warn"
                ? $t("studio.checks.warn")
                : $t("studio.checks.fail")}
          </span>
        </header>
        <ul class="checks">
          {#each report.checks as c (c.name)}
            <li data-status={c.status}>
              <Icon
                name={c.status === "pass" ? "check" : c.status === "fail" ? "x" : "alert"}
                size={15}
                style="color:var(--{c.status === 'pass'
                  ? 'green'
                  : c.status === 'fail'
                    ? 'red'
                    : 'orange'})"
              />
              <span class="cname">{c.name.replaceAll("_", " ")}</span>
              <span class="cdetail tiny">{c.detail}</span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if photo?.faceFound}
      <!-- Position -->
      <Panel title={$t("panel.adjust")} icon="sliders" hint={$t("panel.adjust.hint")}>
        {#if solve}
          <Range
            label={$t("panel.adjust.head")}
            min={solve.head_min_pct}
            max={solve.head_max_pct}
            step={0.001}
            value={solve.head_pct}
            format={(v) => `${Math.round(v * 100)}%`}
            oninput={(e) => {
              adjust.head = +e.currentTarget.value;
              render();
            }}
          />
          <Range
            label={$t("panel.adjust.eye")}
            min={solve.eye_min_pct}
            max={solve.eye_max_pct}
            step={0.001}
            value={solve.eye_from_bottom_pct}
            format={(v) => `${Math.round(v * 100)}%`}
            oninput={(e) => {
              adjust.eye = +e.currentTarget.value;
              render();
            }}
          />
          <Range
            label={$t("panel.adjust.center")}
            min={-solve.center_offset_max_pct}
            max={solve.center_offset_max_pct}
            step={0.1}
            value={solve.center_offset_pct}
            format={(v) => `${v > 0 ? "+" : ""}${v.toFixed(1)}%`}
            oninput={(e) => {
              adjust.center = +e.currentTarget.value;
              render();
            }}
          />
          <Button size="sm" variant="ghost" onclick={resetAdjust}>
            {$t("panel.adjust.reset")}
          </Button>
        {/if}
      </Panel>

      <!-- Background -->
      <Panel title={$t("panel.background")} icon="palette" hint={$t("panel.background.hint")}>
        <p class="tiny">
          {$t("panel.background.required", { name: spec.background.name })}
        </p>
        <Choice
          value={backdropKind}
          onchange={setBackdrop}
          options={[
            { value: "solid", label: $t("panel.background.solid"), swatch: spec.background.hex_render },
            { value: "gradient", label: $t("panel.background.gradient") },
            { value: "vignette", label: $t("panel.background.vignette") },
            { value: "image", label: $t("panel.background.image") },
            { value: "keep", label: $t("panel.background.keep") },
          ]}
        />
        {#if backdropKind === "solid"}
          <label class="colorrow">
            <span>{$t("panel.background.color")}</span>
            <input
              type="color"
              value={hex(options.backdrop.rgb ?? backgroundRgb(spec))}
              oninput={(e) => {
                const v = e.currentTarget.value;
                options.backdrop = {
                  kind: "solid",
                  rgb: [1, 3, 5].map((i) => parseInt(v.slice(i, i + 2), 16)),
                };
                render();
              }}
            />
          </label>
        {/if}
        {#if backdropKind === "image"}
          <Range
            label={$t("panel.background.blur")}
            min={0}
            max={24}
            step={1}
            bind:value={backdropBlur}
            format={(v) => `${v}px`}
            oninput={() => {
              options.backdrop = { kind: "image", blur: backdropBlur };
              render();
            }}
          />
          <Button size="sm" variant="secondary" onclick={() => backdropInput.click()}>
            {$t("panel.background.choose")}
          </Button>
        {/if}
      </Panel>

      <!-- Finishing -->
      <Panel title={$t("panel.finish")} icon="sparkles">
        <Range
          label={$t("panel.finish.retouch")}
          min={0}
          max={0.6}
          step={0.05}
          value={$settings.retouch}
          format={(v) => (v === 0 ? $t("panel.finish.retouch.off") : `${Math.round((v / 0.6) * 100)}%`)}
          oninput={(e) => {
            $settings.retouch = +e.currentTarget.value;
            render();
          }}
        />
        <p class="tiny">{$t("panel.finish.retouch.hint")}</p>

        <label class="toggle">
          <input
            type="checkbox"
            checked={$settings.sharpen}
            onchange={(e) => {
              $settings.sharpen = e.currentTarget.checked;
              render();
            }}
          />
          <span>
            {$t("panel.finish.sharpen")}
            <span class="free-tag">{$t("common.free")}</span>
          </span>
        </label>
        <p class="tiny">{$t("panel.finish.sharpen.hint")}</p>

        <label class="toggle">
          <input
            type="checkbox"
            checked={$settings.quality}
            onchange={(e) => {
              $settings.quality = e.currentTarget.checked;
              // The detail pass happens during matting, so this one does
              // need the expensive half re-run.
              photo?.free();
              photo = null;
              open(file);
            }}
          />
          <span>
            {$t("panel.finish.quality")}
            <span class="free-tag">{$t("common.free")}</span>
          </span>
        </label>
        <p class="tiny">{$t("panel.finish.quality.hint")}</p>
      </Panel>

      <!-- Formal wear -->
      <Panel title={$t("panel.garment")} icon="shirt" hint={$t("panel.garment.hint")}>
        <!-- bind:, not value=: the pill highlighted but the style never
             reached this component, so no jacket was ever drawn. -->
        <Choice
          bind:value={garmentStyle}
          onchange={render}
          options={[
            { value: "none", label: $t("panel.garment.none") },
            { value: "suit-tie", label: $t("panel.garment.suitTie") },
            { value: "suit-open", label: $t("panel.garment.suitOpen") },
            { value: "blouse", label: $t("panel.garment.blouse") },
            { value: "shirt", label: $t("panel.garment.shirt") },
          ]}
        />
        {#if garmentStyle !== "none"}
          <div class="colors">
            {#each [["jacket", "panel.garment.jacket"], ["shirt", "panel.garment.shirtColor"], ["tie", "panel.garment.tie"]] as [key, label] (key)}
              <label class="colorrow">
                <span>{$t(label)}</span>
                <input
                  type="color"
                  value={hex(garmentColors[key])}
                  oninput={(e) => {
                    const v = e.currentTarget.value;
                    garmentColors[key] = [1, 3, 5].map((i) => parseInt(v.slice(i, i + 2), 16));
                    render();
                  }}
                />
              </label>
            {/each}
          </div>
        {/if}
      </Panel>

      <!-- Save -->
      <section class="card stack">
        <h2>{$t("export.title")}</h2>

        <div class="saverow">
          <div>
            <strong>{$t("export.digital")}</strong>
            <p class="tiny">{digitalHint}</p>
          </div>
          <Button icon="download" onclick={saveDigital} disabled={!report}>
            {saved === "digital" ? $t("export.saved") : "JPEG"}
          </Button>
        </div>

        <div class="saverow">
          <div>
            <strong>PNG</strong>
            <p class="tiny">{$t("export.png")}</p>
          </div>
          <Button variant="secondary" icon="download" onclick={savePng} disabled={!report}>
            {saved === "png" ? $t("export.saved") : "PNG"}
          </Button>
        </div>

        <div class="saverow column">
          <div>
            <strong>{$t("export.sheet")}</strong>
            {#if spec.print}
              <p class="tiny">
                {$t("export.sheet.hint", {
                  n: sheetCount ?? "—",
                  sheet: SHEETS.find((s) => s.value === sheet)?.label ?? sheet,
                })}
              </p>
            {:else}
              <p class="tiny">{$t("export.sheet.none")}</p>
            {/if}
          </div>
          {#if spec.print}
            <Choice
              bind:value={sheet}
              options={SHEETS}
              onchange={() => (sheetCount = photo.sheetInfo(sheet).count)}
            />
            <Button variant="secondary" icon="printer" onclick={saveSheet}>
              {saved === "sheet" ? $t("export.saved") : $t("export.sheet")}
            </Button>
          {/if}
        </div>
      </section>

      <Button variant="ghost" full onclick={() => fileInput.click()}>
        {$t("studio.retry")}
      </Button>
    {/if}
  </div>
{/if}

<style>
  .dochead {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
  }
  .dochead h1 {
    font: var(--type-h2);
  }
  .dochead > div {
    flex: 1;
  }
  .preview {
    position: relative;
    display: grid;
    place-items: center;
    min-height: 260px;
    padding: var(--space-5);
    border-radius: var(--radius-lg);
    background: var(--bg-sunken);
    border: var(--border-width) solid var(--border-hairline);
  }
  .preview img {
    max-height: 44vh;
    width: auto;
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
  }
  .preview img.dim {
    opacity: 0.45;
    filter: grayscale(0.4);
  }
  .placeholder {
    width: 160px;
    height: 200px;
    border-radius: var(--radius-sm);
  }
  .veil {
    position: absolute;
    inset: 0;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    text-align: center;
    background: color-mix(in srgb, var(--bg-page) 78%, transparent);
    backdrop-filter: blur(3px);
    border-radius: var(--radius-lg);
    color: var(--text-muted);
    font-size: 0.9rem;
  }
  .notice {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    border-color: color-mix(in srgb, var(--red) 35%, transparent);
  }
  .notice p {
    margin-bottom: var(--space-3);
  }
  .padded {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
  }
  .checkhead {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }
  .checkhead h2 {
    font: var(--type-h4);
  }
  .checkhead span {
    font-size: 0.88rem;
    font-weight: var(--weight-medium);
  }
  .checks {
    list-style: none;
    padding: 0;
    display: grid;
    gap: var(--space-3);
  }
  .checks li {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px var(--space-3);
  }
  .cname {
    color: var(--text-strong);
    font-size: 0.9rem;
    text-transform: capitalize;
  }
  .cdetail {
    grid-column: 2;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    cursor: pointer;
  }
  .toggle input {
    width: 20px;
    height: 20px;
    accent-color: var(--gray-950);
  }
  .toggle span {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.95rem;
  }
  .colors {
    display: grid;
    gap: var(--space-2);
  }
  .colorrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    font-size: 0.92rem;
    color: var(--text-body);
  }
  .colorrow input {
    width: 46px;
    height: 30px;
    padding: 0;
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-sm);
    background: none;
    cursor: pointer;
  }
  .saverow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }
  .saverow.column {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-3);
  }
  .saverow strong {
    font-weight: var(--weight-medium);
    color: var(--text-strong);
  }
</style>
