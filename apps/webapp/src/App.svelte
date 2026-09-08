<script>
  import { t, lang, LANGUAGES } from "$lib/i18n.js";
  import { loadSpecs } from "$lib/pipeline.js";
  import Icon from "$ui/Icon.svelte";
  import Home from "./views/Home.svelte";
  import Picker from "./views/Picker.svelte";
  import Studio from "./views/Studio.svelte";
  import Batch from "./views/Batch.svelte";
  import Coverage from "./views/Coverage.svelte";
  import About from "./views/About.svelte";

  /*
    A hash router in twenty lines rather than a routing library. The app
    has six screens and one of them holds all the state; anything more is
    a dependency that has to be kept current for no benefit here.
  */
  let route = $state(parse(location.hash));
  function parse(hash) {
    const [name, arg] = hash.replace(/^#\/?/, "").split("/");
    return { name: name || "home", arg: arg ? decodeURIComponent(arg) : "" };
  }
  function go(name, arg = "") {
    location.hash = arg ? `#/${name}/${encodeURIComponent(arg)}` : `#/${name}`;
  }

  $effect(() => {
    const on = () => (route = parse(location.hash));
    addEventListener("hashchange", on);
    return () => removeEventListener("hashchange", on);
  });

  // The spec dataset is small and every screen wants it, so it is loaded
  // once here rather than per view. This is also the wasm module's first
  // instantiation, which warms it before the user picks a photo.
  const specsPromise = loadSpecs();
  let specs = $state([]);
  let loadError = $state(null);
  specsPromise.then((s) => (specs = s)).catch((e) => (loadError = e));

  /*
    The picked file lives here, not in a view, so that choosing a photo and
    choosing a document are two independent steps the user can revisit in
    either order without losing the other.
  */
  let picked = $state(null);
  const setPicked = (file) => (picked = file);
</script>

<header class="topbar">
  <button class="brand" onclick={() => go("home")}>
    <span class="of">Open</span>PhotoId
  </button>

  {#if route.name !== "home"}
    <button class="linkish" onclick={() => go("home")} aria-label={$t("nav.back")}>
      <Icon name="left" size={18} />
    </button>
  {/if}

  <label class="lang">
    <Icon name="globe" size={15} />
    <span class="sr-only">Language</span>
    <select bind:value={$lang}>
      {#each LANGUAGES as l (l.value)}
        <option value={l.value}>{l.label}</option>
      {/each}
    </select>
  </label>
</header>

<main class="shell">
  {#if loadError}
    <div class="card">
      <h2>{$t("common.error")}</h2>
      <p class="muted">{loadError.message}</p>
    </div>
  {:else if route.name === "home"}
    <Home {specs} {go} {setPicked} />
  {:else if route.name === "pick"}
    <Picker {specs} onpick={(id) => go("studio", id)} />
  {:else if route.name === "studio"}
    <Studio {specs} specId={route.arg} file={picked} {go} {setPicked} />
  {:else if route.name === "batch"}
    <Batch {specs} {go} />
  {:else if route.name === "coverage"}
    <Coverage {specs} />
  {:else if route.name === "about"}
    <About />
  {:else}
    <Home {specs} {go} {setPicked} />
  {/if}
</main>

<style>
  .linkish {
    background: none;
    border: 0;
    color: var(--text-muted);
    cursor: pointer;
    padding: var(--space-1);
    display: inline-flex;
  }
  .lang {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--text-muted);
  }
  .lang select {
    background: none;
    border: 0;
    color: inherit;
    font-size: 0.85rem;
    cursor: pointer;
    padding: 4px 2px;
  }
</style>
