<script>
  import { t, lang, LANGUAGES, chooseLocale, pageHasTranslations } from "$lib/i18n.js";
  import { loadSpecs } from "$lib/pipeline.js";
  import Icon from "$ui/Icon.svelte";
  import Home from "./views/Home.svelte";
  import Picker from "./views/Picker.svelte";
  import Studio from "./views/Studio.svelte";
  import Batch from "./views/Batch.svelte";
  import Coverage from "./views/Coverage.svelte";
  import About from "./views/About.svelte";
  import Account from "./views/Account.svelte";
  import Diagnostics from "./views/Diagnostics.svelte";
  import { isSignInReturn, isConsumedSignInReturn } from "$lib/openapps.js";
  import { isNative, openAccountInBrowser } from "$lib/platform.js";

  /*
    A hash router in twenty lines rather than a routing library. The app
    has six screens and one of them holds all the state; anything more is
    a dependency that has to be kept current for no benefit here.
  */
  /*
    The sign-in return overrides the hash. A provider sends the browser back
    with the code in the fragment; this router would read `code=…` as a
    route name, match nothing, and render Home, where nothing account-related
    mounts and the code is never exchanged. Detected here, before a view is
    chosen, because by then it is too late.
  */
  let route = $state(isSignInReturn() ? { name: "account", arg: "" } : parse(location.hash));
  function parse(hash) {
    // Routes are `#/name`. A bare `#anchor` is a link into the page's own
    // marketing copy, which shares this document, so it means home.
    if (hash && !hash.startsWith("#/")) return { name: "home", arg: "" };
    const [name, arg] = hash.replace(/^#\/?/, "").split("/");
    return { name: name || "home", arg: arg ? decodeURIComponent(arg) : "" };
  }
  // The marketing copy is one document with the app, so it has to know
  // which screen is showing. Static in the HTML for a crawler, hidden by
  // CSS once someone is past the front door.
  $effect(() => {
    document.body.dataset.route = route.name;
  });

  function go(name, arg = "") {
    location.hash = arg ? `#/${name}/${encodeURIComponent(arg)}` : `#/${name}`;
  }

  // The SDK deletes the code from the fragment once exchanged, which fires
  // hashchange with an empty hash; the first such change after a return is
  // rewritten to the account route rather than followed to Home.
  let holdAccount = isSignInReturn();
  $effect(() => {
    const on = () => {
      if (isSignInReturn()) {
        holdAccount = true;
        route = { name: "account", arg: "" };
        return;
      }
      if (holdAccount && isConsumedSignInReturn()) {
        holdAccount = false;
        location.hash = "#/account";
        return;
      }
      holdAccount = false;
      route = parse(location.hash);
    };
    addEventListener("hashchange", on);
    return () => removeEventListener("hashchange", on);
  });

  // The spec dataset is small and every screen wants it, so it is loaded
  // once here rather than per view. This is also the wasm module's first
  // instantiation, which warms it before the user picks a photo.
  const specsPromise = loadSpecs();
  // A phone app launched with `-diag` (from a simulator or adb) goes
  // straight to the device test, so a build can be checked without a tap.
  if (isNative) {
    import("$lib/platform.js").then(({ nativeInfo }) => nativeInfo()).then((i) => {
      if (i?.autorun_diagnostics) location.hash = "#/diagnostics/run";
    }).catch(() => {});
  }
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

  // Installed to a home screen: no browser chrome, and the page around the
  // app hides its own, so this is the app alone on the screen.
  const standalone =
    typeof matchMedia !== "undefined" &&
    (matchMedia("(display-mode: standalone)").matches || navigator.standalone === true);

  /*
    Whether this app draws its own top bar.

    Everywhere but one place, yes: the phone and desktop builds have no page
    around them, and every app route on the web replaces the marketing copy.
    The exception is the web's landing route, where the page's own header is
    already at the top of the document and ours would be the second bar on it.

    Installed to a home screen is not that exception: the site's stylesheet
    hides its whole chrome there, slot included, so the controls we hand it
    would be hidden with it and the screen would have no account button and no
    way to change language at all.
  */
  const showAppBar = $derived(isNative || standalone || route.name !== "home");

  /*
    Who draws the language control.

    On openphotoid.com the page draws one: a globe that switches language by
    *navigating* -- to /de.html -- so the choice is shareable, crawlable, and
    present on the privacy page too, none of which a select inside the app can
    manage (APP-177). Two of them in one corner is one too many, so wherever
    the page publishes a translation ring, the app leaves language to it.

    Not everywhere, though: installed to a home screen the page's chrome is
    hidden by its own stylesheet, and the phone apps have no page around them
    at all. Both are the app alone on the screen, and it keeps its control.
  */
  const ownsLanguage = isNative || standalone || !pageHasTranslations();

  /*
    Move a node into an element outside this component's tree.

    Svelte has no portal, and the alternative — rendering the controls a second
    time inside the site header — would be two sets of components bound to one
    store, which drift the moment one of them is the one you click. Moving the
    node keeps it a single instance with its listeners intact.
  */
  function portal(node, selector) {
    const target = document.querySelector(selector);
    if (target) target.append(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }
</script>

<!--
  The account and language controls, defined once and rendered in one of two
  places: this app's own top bar, or — on the web's landing route — the site
  header that the page around us already draws.
-->
{#snippet appControls()}
  <!-- The account lives here and only here: a round, icon-only control, top
       right, reachable from every screen. It navigates to its own route
       rather than opening a panel in place. -->
  <button
    class="linkish account"
    class:on={route.name === "account"}
    onclick={() => (isNative ? openAccountInBrowser() : go("account"))}
    aria-label={$t("nav.account")}
    title={$t("nav.account")}
  >
    <Icon name="user" size={18} />
  </button>

  {#if ownsLanguage}
    <label class="lang">
      <Icon name="globe" size={15} />
      <span class="sr-only">Language</span>
      <select value={$lang} onchange={(e) => chooseLocale(e.currentTarget.value)}>
        {#each LANGUAGES as l (l.value)}
          <option value={l.value}>{l.label}</option>
        {/each}
      </select>
      <Icon name="down" size={14} />
    </label>
  {/if}
{/snippet}

{#if showAppBar}
  <header class="topbar">
    <button class="brand" onclick={() => go("home")}>
      <span class="of">Open</span>PhotoId<span class="dot">.</span>
    </button>

    {#if route.name !== "home"}
      <button class="linkish" onclick={() => go("home")} aria-label={$t("nav.back")}>
        <Icon name="left" size={18} />
      </button>
    {/if}

    {@render appControls()}
  </header>
{:else}
  <!--
    The landing route on the web. The page already has a header at the top of
    the document; drawing ours as well put a second bar — and a second
    "OpenPhotoId." — half a screen below the first, so the page opened with a
    headline and its navigation arrived after a scroll. Our two controls move
    up into that header instead; `use:portal` reparents this node, so they are
    the same live components with the same bindings, not a second copy.
  -->
  <div class="ported-controls" use:portal={"#app-controls-slot"}>
    {@render appControls()}
  </div>
{/if}

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
  {:else if route.name === "account"}
    <Account {go} />
  {:else if route.name === "diagnostics"}
    <Diagnostics {go} autorun={route.arg === "run"} />
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
  /* The account and language controls share one height with the header's
     button beside them, so the row reads as one bar rather than three
     components that disagree by a few pixels. */
  .account {
    width: var(--control-h-md);
    height: var(--control-h-md);
    align-items: center;
    justify-content: center;
    padding: 0;
    border-radius: var(--radius-full, 999px);
  }
  .account:hover,
  .account.on {
    color: var(--text-strong);
    background: var(--surface-hover);
  }
  .lang {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: var(--control-h-md);
    padding: 0 var(--space-2) 0 var(--space-3);
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-md);
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color var(--duration-fast) var(--ease-standard),
      color var(--duration-fast) var(--ease-standard);
  }
  .lang:hover,
  .lang:focus-within {
    border-color: var(--border-strong);
    color: var(--text-strong);
  }
  .lang select {
    appearance: none;
    -webkit-appearance: none;
    background: none;
    border: 0;
    color: inherit;
    font: var(--type-ui);
    cursor: pointer;
    padding: 0;
    height: 100%;
    /* As wide as the chosen language, not the longest one in the list. */
    field-sizing: content;
    outline: none;
  }
  .lang select option {
    color: var(--text-strong);
    background: var(--surface-raised);
  }
</style>
