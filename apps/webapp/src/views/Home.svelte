<script>
  import { t } from "$lib/i18n.js";
  import { readyOffline } from "$lib/models.js";
  import Button from "$ui/Button.svelte";
  import Icon from "$ui/Icon.svelte";

  let { specs = [], go, setPicked } = $props();

  let fileInput;
  let cameraInput;
  let offlineReady = $state(false);
  readyOffline().then((v) => (offlineReady = v));

  const countries = $derived(new Set(specs.map((s) => s.country)).size);

  function choose(e) {
    const file = e.currentTarget.files?.[0];
    if (!file) return;
    setPicked(file);
    go("pick");
    // Reset, so picking the same file twice in a row still fires change.
    e.currentTarget.value = "";
  }
</script>

<div class="stack-lg" style="padding-top:var(--space-6)">
  <div class="hero">
    <h1>{$t("app.tagline")}</h1>
    <p class="oa-lead">{$t("app.sub")}</p>
  </div>

  <div class="stack">
    <Button size="lg" full icon="image" onclick={() => fileInput.click()}>
      {$t("home.start")}
    </Button>
    <Button size="lg" full variant="secondary" icon="camera" onclick={() => cameraInput.click()}>
      {$t("home.camera")}
    </Button>
    <p class="tiny" style="text-align:center">
      {$t("home.count", { n: specs.length, c: countries })} ·
      <button class="inline" onclick={() => go("coverage")}>{$t("nav.coverage")}</button>
    </p>
  </div>

  <input
    bind:this={fileInput}
    type="file"
    accept="image/*"
    class="sr-only"
    onchange={choose}
  />
  <!-- `capture` asks a phone for the camera directly. Desktop browsers
       ignore it and show the normal picker, which is the right fallback. -->
  <input
    bind:this={cameraInput}
    type="file"
    accept="image/*"
    capture="user"
    class="sr-only"
    onchange={choose}
  />

  <ul class="promises">
    <li>
      <Icon name="check" size={17} />
      <div>
        <strong>{$t("promise.free.title")}</strong>
        <p class="tiny">{$t("promise.free.body")}</p>
      </div>
    </li>
    <li>
      <Icon name="shield" size={17} />
      <div>
        <strong>{$t("promise.private.title")}</strong>
        <p class="tiny">{$t("promise.private.body")}</p>
      </div>
    </li>
    <li>
      <Icon name="wifiOff" size={17} />
      <div>
        <strong>
          {$t("promise.offline.title")}
          {#if offlineReady}<span class="free-tag">{$t("promise.offline.ready")}</span>{/if}
        </strong>
        <p class="tiny">{$t("promise.offline.body")}</p>
      </div>
    </li>
  </ul>

  <div class="card">
    <h3 style="margin-bottom:var(--space-3)">{$t("home.tips.title")}</h3>
    <ol class="tips">
      <li>{$t("home.tips.1")}</li>
      <li>{$t("home.tips.2")}</li>
      <li>{$t("home.tips.3")}</li>
      <li>{$t("home.tips.4")}</li>
    </ol>
  </div>

  <nav class="footer">
    <button class="inline" onclick={() => go("batch")}>{$t("nav.batch")}</button>
    <button class="inline" onclick={() => go("coverage")}>{$t("nav.coverage")}</button>
    <button class="inline" onclick={() => go("about")}>{$t("nav.about")}</button>
  </nav>
</div>

<style>
  .hero h1 {
    text-wrap: balance;
    margin-bottom: var(--space-3);
  }
  .promises {
    list-style: none;
    padding: 0;
    display: grid;
    gap: var(--space-4);
  }
  .promises li {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    color: var(--text-muted);
  }
  .promises strong {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-strong);
    font-weight: var(--weight-medium);
    font-size: 0.95rem;
  }
  .tips {
    display: grid;
    gap: var(--space-2);
    color: var(--text-muted);
    font-size: 0.92rem;
    line-height: 1.5;
  }
  .inline {
    background: none;
    border: 0;
    padding: 0;
    color: var(--text-link);
    cursor: pointer;
    font: inherit;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .footer {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
    justify-content: center;
    padding-top: var(--space-4);
    border-top: var(--border-width) solid var(--border-hairline);
  }
</style>
