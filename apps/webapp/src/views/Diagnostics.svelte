<script>
  /*
    A page for a phone in your hand: open it, and it runs the whole pipeline
    on a sample portrait shipped with the app and prints what this browser
    did with it — which inference provider, how many threads, whether the
    page is cross-origin isolated, the time each stage took, and the eleven
    checks. Nothing to pick, nothing to type; the numbers on the test guide
    come from the same measures.
  */
  import { t } from "$lib/i18n.js";
  import { Photo, loadModels } from "$lib/pipeline.js";
  import Button from "$ui/Button.svelte";

  // `#/diagnostics/run` starts without a tap, so a device can be tested by
  // opening one URL and reading the screen.
  let { go, autorun = false } = $props();

  let stage = $state("idle"); // idle | loading | running | done | failed
  let progress = $state(0);
  let error = $state("");
  let result = $state(null);
  let resultUrl = $state("");

  const env = {
    ua: navigator.userAgent,
    isolated: globalThis.crossOriginIsolated === true,
    cores: navigator.hardwareConcurrency ?? null,
    webgpu: typeof navigator !== "undefined" && "gpu" in navigator,
    memoryGb: navigator.deviceMemory ?? null,
    standalone: matchMedia("(display-mode: standalone)").matches || navigator.standalone === true,
    viewport: `${innerWidth}×${innerHeight} @${devicePixelRatio}x`,
  };

  $effect(() => {
    if (autorun && stage === "idle") run();
  });

  async function run() {
    stage = "loading";
    error = "";
    result = null;
    const t0 = performance.now();
    try {
      const { provider } = await loadModels(({ fraction }) => (progress = fraction));
      stage = "running";
      const res = await fetch(new URL("./sample-portrait.jpg", document.baseURI));
      const bytes = new Uint8Array(await res.arrayBuffer());
      const photo = await Photo.open(bytes, { quality: true });
      if (!photo.faceFound) throw new Error($t("studio.noface"));
      const { report } = await photo.render("us-passport", { retouch: 0, backdrop: { kind: "solid", rgb: [255, 255, 255] }, garment: null }, {}, { enhance: true });
      if (resultUrl) URL.revokeObjectURL(resultUrl);
      resultUrl = URL.createObjectURL(new Blob([await photo.outputJpeg(90)], { type: "image/jpeg" }));
      const timings = Object.fromEntries(
        performance.getEntriesByType("measure")
          .filter((m) => m.name.startsWith("openphotoid:"))
          .map((m) => [m.name.slice("openphotoid:".length), Math.round(m.duration)]),
      );
      result = {
        provider,
        threads: globalThis.__openphotoid?.threads ?? 1,
        notes: globalThis.__openphotoid?.notes ?? [],
        wall: Math.round(performance.now() - t0),
        timings,
        checks: report.checks,
        overall: report.overall,
        source: `${photo.width}×${photo.height}`,
      };
      photo.free();
      stage = "done";
    } catch (e) {
      error = e?.message ?? String(e);
      stage = "failed";
    }
  }
</script>

<div class="stack" style="padding-top:var(--space-5)">
  <h1>{$t("diag.title")}</h1>
  <p class="oa-lead">{$t("diag.lede")}</p>

  <div class="card">
    <dl class="kv">
      <dt>{$t("diag.browser")}</dt><dd class="mono small">{env.ua}</dd>
      <dt>{$t("diag.viewport")}</dt><dd class="mono">{env.viewport}{env.standalone ? " · installed" : ""}</dd>
      <dt>WebGPU</dt><dd class="mono">{env.webgpu ? "available" : "not available"}</dd>
      <dt>{$t("diag.isolated")}</dt><dd class="mono">{env.isolated ? "yes" : "no"}{env.cores ? ` · ${env.cores} cores` : ""}{env.memoryGb ? ` · ${env.memoryGb} GB` : ""}</dd>
    </dl>
  </div>

  {#if stage === "idle" || stage === "failed"}
    <Button size="lg" full icon="sparkles" onclick={run}>{$t("diag.run")}</Button>
  {:else if stage === "loading"}
    <p class="muted">{$t("common.downloading", { pct: Math.round(progress * 100) })}</p>
  {:else if stage === "running"}
    <p class="muted">{$t("studio.working")}</p>
  {/if}

  {#if error}
    <div class="card notice">
      {#if /Can't create a session|Failed to find args|OrtValue/.test(error)}
        <!-- Seen on iOS 17.0 Safari: the runtime's WebAssembly is mis-executed
             at session creation, at every optimisation level and with one
             thread. iOS 26 runs it. Say so before the raw text. -->
        <p><strong>{$t("diag.noSession")}</strong></p>
      {/if}
      <p class="raw">{error}</p>
    </div>
  {/if}

  {#if result}
    <div class="card stack" data-testid="diag-result">
      <div class="preview-row">
        {#if resultUrl}<img src={resultUrl} alt={$t("studio.after")} />{/if}
        <dl class="kv">
          <dt>{$t("diag.provider")}</dt><dd class="mono">{result.provider === "native" ? "native · tract" : result.provider}{result.provider !== "webgpu" ? ` · ${result.threads} thread${result.threads === 1 ? "" : "s"}` : ""}</dd>
          <dt>{$t("diag.source")}</dt><dd class="mono">{result.source}</dd>
          <dt>{$t("diag.total")}</dt><dd class="mono">{result.wall} ms</dd>
          {#each result.notes as n, i (i)}
            <dt class="tiny">note</dt><dd class="tiny">{n}</dd>
          {/each}
          {#each Object.entries(result.timings) as [k, v] (k)}
            <dt class="tiny">{k}</dt><dd class="mono tiny">{v} ms</dd>
          {/each}
        </dl>
      </div>
      <ul class="checks">
        {#each result.checks as c (c.name)}
          <li data-status={c.status}>
            <span class="cname">{c.name.replaceAll("_", " ")}</span>
            <span class="cdetail tiny">{c.status} · {c.detail}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <nav class="footer">
    <button class="inline" onclick={() => go("home")}>{$t("nav.home")}</button>
  </nav>
</div>

<style>
  .kv { display: grid; grid-template-columns: max-content 1fr; gap: var(--space-1) var(--space-4); margin: 0; }
  .kv dt { color: var(--text-muted); font-size: 0.85rem; }
  .kv dd { margin: 0; word-break: break-word; }
  .mono { font-family: var(--font-mono); font-size: 0.9rem; }
  .small { font-size: 0.75rem; }
  /* Runtime errors are one long token; without this the page widens past the phone. */
  .raw { font-family: var(--font-mono); font-size: 0.75rem; overflow-wrap: anywhere; word-break: break-word; color: var(--text-muted); }
  .notice p { overflow-wrap: anywhere; }
  .preview-row { display: grid; grid-template-columns: 120px 1fr; gap: var(--space-4); align-items: start; }
  .preview-row img { width: 120px; border-radius: var(--radius-sm); border: var(--border-width) solid var(--border-hairline); }
  .checks { list-style: none; padding: 0; margin: 0; display: grid; gap: var(--space-2); }
  .checks li { display: grid; gap: 2px; }
  .cname { text-transform: capitalize; font-weight: var(--weight-medium); }
  .footer { display: flex; justify-content: center; padding-top: var(--space-4); border-top: var(--border-width) solid var(--border-hairline); }
</style>
