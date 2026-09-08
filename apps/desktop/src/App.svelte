<script>
  import { invoke } from "@tauri-apps/api/core";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { writeFile } from "@tauri-apps/plugin-fs";
  import CanvasSpike from "./CanvasSpike.svelte";
  import BrandMark from "./design/components/BrandMark.svelte";
  import Button from "./design/components/Button.svelte";
  import Card from "./design/components/Card.svelte";
  import Badge from "./design/components/Badge.svelte";
  import Select from "./design/components/Select.svelte";
  import Slider from "./design/components/Slider.svelte";
  import Icon from "./design/components/Icon.svelte";

  let tab = $state("photo"); // "photo" | "canvas-spike"

  let specs = $state([]);
  let specId = $state("");
  let sourceUrl = $state("");
  let sourceBytes = $state(null); // base64 (no data: prefix)
  let outputUrl = $state("");
  let checks = $state([]);
  let overall = $state("");
  let status = $state("Drop or choose a photo to begin.");
  let busy = $state(false);
  let dragOver = $state(false);
  // Current crop adjustment + the bounds the sliders are allowed to move
  // within. The bounds keep the percentage-band geometry checks (head
  // height, eye line, centering) satisfiable, but checks that depend on
  // the source photo's actual pixel detail (inter-eye distance, blur) can
  // still fail near the edges — see CropAdjustment's doc comment in
  // crates/frame-compliance/src/crop.rs. That's why every adjustment
  // re-validates and updates the checklist rather than assuming pass.
  // null until a photo's been processed at least once.
  let adjustment = $state(null);
  let adjusting = $state(false);
  let adjustTimer = null;

  async function loadSpecs() {
    specs = await invoke("list_specs");
    if (specs.length && !specId) specId = specs[0].id;
  }
  loadSpecs();

  function readFile(file) {
    if (!file) return;
    status = `Loaded ${file.name}`;
    outputUrl = "";
    checks = [];
    overall = "";
    adjustment = null;
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result;
      sourceUrl = dataUrl;
      sourceBytes = dataUrl.split(",").pop();
    };
    reader.readAsDataURL(file);
  }

  function onFileInput(e) {
    readFile(e.target.files?.[0]);
  }
  function onDrop(e) {
    e.preventDefault();
    dragOver = false;
    readFile(e.dataTransfer?.files?.[0]);
  }

  async function process() {
    if (!sourceBytes || !specId) return;
    busy = true;
    status = "Processing…";
    try {
      const result = await invoke("process_photo", {
        imageBase64: sourceBytes,
        specId,
      });
      outputUrl = result.output_jpeg_base64;
      checks = result.checks;
      overall = result.overall;
      adjustment = result.adjustment;
      status = "Done.";
    } catch (err) {
      status = `Error: ${err}`;
    } finally {
      busy = false;
    }
  }

  // Called on every slider drag. Debounced — adjust_crop is cheap (no ONNX
  // inference, see pipeline.rs's `finalize`) but still an IPC round trip,
  // so this avoids flooding it on every pixel of drag movement.
  function onAdjustInput() {
    clearTimeout(adjustTimer);
    adjustTimer = setTimeout(applyAdjustment, 150);
  }

  async function applyAdjustment() {
    if (!adjustment) return;
    adjusting = true;
    try {
      const result = await invoke("adjust_crop", {
        headPct: adjustment.head_pct,
        eyeFromBottomPct: adjustment.eye_from_bottom_pct,
        centerOffsetPct: adjustment.center_offset_pct,
      });
      outputUrl = result.output_jpeg_base64;
      checks = result.checks;
      overall = result.overall;
      adjustment = result.adjustment;
    } catch (err) {
      status = `Adjustment error: ${err}`;
    } finally {
      adjusting = false;
    }
  }

  function resetAdjustment() {
    if (!adjustment) return;
    adjustment = {
      ...adjustment,
      head_pct: (adjustment.head_min_pct + adjustment.head_max_pct) / 2,
      eye_from_bottom_pct: (adjustment.eye_min_pct + adjustment.eye_max_pct) / 2,
      center_offset_pct: 0,
    };
    applyAdjustment();
  }

  // The <a download> browser trick is unreliable inside a native webview
  // (WKWebView/WebView2/WebKitGTK don't consistently honor it, especially
  // for data: URLs) — use the platform's real save flow instead: a native
  // save dialog, then write the bytes directly.
  async function download() {
    if (!outputUrl) return;
    try {
      const path = await saveDialog({
        defaultPath: `${specId}.jpg`,
        filters: [{ name: "JPEG image", extensions: ["jpg", "jpeg"] }],
      });
      if (!path) return; // user cancelled the dialog
      const base64 = outputUrl.split(",").pop();
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await writeFile(path, bytes);
      status = `Saved to ${path}`;
    } catch (err) {
      status = `Save failed: ${err}`;
    }
  }

  // Badge tone carries the pass/warn/fail signal; the text label inside
  // the badge (not color alone) is what satisfies WCAG 1.4.1.
  const statusTone = { pass: "success", warn: "warning", fail: "danger", not_checked: "neutral" };
  const statusText = { pass: "PASS", warn: "WARN", fail: "FAIL", not_checked: "—" };

  const specOptions = $derived(
    specs.map((s) => ({
      value: s.id,
      label: `${s.id} (${s.country}/${s.document}, ${s.output_w}x${s.output_h})`,
    })),
  );
</script>

<main>
  <header>
    <BrandMark variant="lockup" size={26} name="OpenPhotoId" icon="frame" accent="var(--app-frame)" />
    {#if import.meta.env.DEV}
      <nav>
        <button class:active={tab === "photo"} onclick={() => (tab = "photo")}>Photo</button>
        <button class:active={tab === "canvas-spike"} onclick={() => (tab = "canvas-spike")}>
          Canvas perf spike (M0)
        </button>
      </nav>
    {/if}
    <span class="spacer"></span>
  </header>

  {#if import.meta.env.DEV && tab === "canvas-spike"}
    <CanvasSpike />
  {:else}
    <div class="photo-tab">
      <Card>
        <section class="controls">
          <label
            class="dropzone"
            class:drag={dragOver}
            ondragover={(e) => {
              e.preventDefault();
              dragOver = true;
            }}
            ondragleave={() => (dragOver = false)}
            ondrop={onDrop}
          >
            <input type="file" accept="image/*" onchange={onFileInput} />
            {#if sourceUrl}
              <img src={sourceUrl} alt="source" />
            {:else}
              <Icon name="image-plus" size={22} color="var(--text-faint)" />
              <span>Drop a photo here, or click to choose</span>
            {/if}
          </label>

          <Select label="Compliance spec" bind:value={specId} options={specOptions} />

          <Button fullWidth onclick={process} disabled={!sourceBytes || busy} loading={busy}>
            {busy ? "Processing…" : "Process"}
          </Button>
          <p class="status">{status}</p>
        </section>
      </Card>

      <Card>
        <section class="result">
          {#if outputUrl}
            <img src={outputUrl} alt="compliant output" class="output" class:adjusting />
            <Button variant="secondary" iconLeft="download" onclick={download}>Download JPEG</Button>
          {:else}
            <div class="placeholder">
              <Icon name="image" size={28} color="var(--text-faint)" />
              <span>Output appears here</span>
            </div>
          {/if}

          {#if adjustment}
            <Card tone="subtle" padding="var(--space-4)" style="width:100%">
              <div class="adjust-header">
                <h2>Adjust crop</h2>
                <Button variant="ghost" size="sm" onclick={resetAdjustment}>Reset to auto</Button>
              </div>
              <label class="adjust-row">
                <span>Zoom (head size): {(adjustment.head_pct * 100).toFixed(0)}%</span>
                <Slider
                  min={adjustment.head_min_pct}
                  max={adjustment.head_max_pct}
                  step="0.005"
                  bind:value={adjustment.head_pct}
                  oninput={onAdjustInput}
                  aria-label="Zoom, adjusts how much of the frame the head fills"
                />
              </label>
              <label class="adjust-row">
                <span>Vertical position: {(adjustment.eye_from_bottom_pct * 100).toFixed(0)}% from bottom</span>
                <Slider
                  min={adjustment.eye_min_pct}
                  max={adjustment.eye_max_pct}
                  step="0.005"
                  bind:value={adjustment.eye_from_bottom_pct}
                  oninput={onAdjustInput}
                  aria-label="Vertical position, moves the face up or down within the crop"
                />
              </label>
              <label class="adjust-row">
                <span>Horizontal position: {adjustment.center_offset_pct.toFixed(1)}%</span>
                <Slider
                  min={-adjustment.center_offset_max_pct}
                  max={adjustment.center_offset_max_pct}
                  step="0.2"
                  bind:value={adjustment.center_offset_pct}
                  oninput={onAdjustInput}
                  aria-label="Horizontal position, shifts the face left or right within the crop"
                />
              </label>
              <p class="adjust-hint">
                Sliders are bounded to what {specId} allows geometrically — but the checklist can
                still change as you adjust (e.g. zooming out too far can fail eye-distance on
                lower-detail photos), so keep an eye on it.
              </p>
            </Card>
          {/if}
        </section>
      </Card>

      <Card>
        <section class="checklist">
          <div class="checklist-header">
            <h2>Validation</h2>
            {#if overall}<Badge tone={statusTone[overall] ?? "neutral"}>{statusText[overall] ?? overall}</Badge>{/if}
          </div>
          <ul>
            {#each checks as c}
              <li>
                <Badge size="sm" tone={statusTone[c.status] ?? "neutral"}>{statusText[c.status] ?? "?"}</Badge>
                <span class="check-text"><strong>{c.name}</strong>: {c.detail}</span>
              </li>
            {/each}
          </ul>
        </section>
      </Card>
    </div>
  {/if}
</main>

<style>
  :global(html, body) {
    height: 100%;
  }
  :global(body) {
    margin: 0;
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-page);
  }
  header {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: var(--space-3) var(--space-5);
    background: var(--surface-card);
    border-bottom: var(--border-width) solid var(--border-hairline);
  }
  nav {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  nav button {
    background: none;
    border: none;
    font: var(--weight-medium) var(--text-sm) / 1 var(--font-sans);
    color: var(--text-muted);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition-control);
  }
  nav button:hover {
    background: var(--surface-hover);
  }
  nav button.active {
    background: var(--surface-active);
    color: var(--text-strong);
  }
  .spacer {
    flex: 1;
  }
  .photo-tab {
    flex: 1;
    display: grid;
    grid-template-columns: 320px 1fr 320px;
    gap: var(--space-5);
    padding: var(--space-5);
    overflow: auto;
    align-items: start;
  }
  .controls,
  .checklist {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  h2 {
    font: var(--type-h4);
    margin: 0;
  }
  .dropzone {
    border: var(--border-width-strong) dashed var(--border-strong);
    border-radius: var(--radius-lg);
    min-height: 220px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    cursor: pointer;
    position: relative;
    overflow: hidden;
    color: var(--text-faint);
    font: var(--type-caption);
    text-align: center;
    transition: var(--transition-control);
  }
  .dropzone:hover {
    background: var(--surface-hover);
  }
  .dropzone.drag {
    border-color: var(--app-frame);
    background: var(--surface-hover);
  }
  .dropzone input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .dropzone img {
    max-width: 100%;
    max-height: 220px;
    object-fit: contain;
  }
  .status {
    font: var(--type-caption);
    color: var(--text-faint);
    margin: 0;
  }
  .result {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
  }
  .result img.output {
    max-width: 100%;
    max-height: 55vh;
    border-radius: var(--radius-xl);
    border: var(--border-width) solid var(--border-hairline);
    transition: opacity var(--duration-fast) var(--ease-standard);
  }
  .result img.output.adjusting {
    opacity: 0.6;
  }
  .adjust-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }
  .adjust-header h2 {
    font: var(--type-ui);
  }
  .adjust-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font: var(--type-caption);
    color: var(--text-body);
    margin-bottom: var(--space-3);
  }
  .adjust-hint {
    font: var(--type-caption);
    color: var(--text-faint);
    margin: var(--space-2) 0 0;
  }
  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-faint);
    font: var(--type-caption);
    border: var(--border-width) dashed var(--border-hairline);
    border-radius: var(--radius-lg);
    padding: var(--space-16) var(--space-6);
    width: 100%;
  }
  .checklist-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .checklist ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .checklist li {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }
  .check-text {
    font: var(--type-caption);
    color: var(--text-body);
  }
  .check-text strong {
    color: var(--text-strong);
    font-weight: var(--weight-medium);
  }
</style>
