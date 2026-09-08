<script>
  /*
    The account page. It unlocks nothing, and says so first.

    Every feature here runs on the visitor's device, so there is nothing to
    meter and nothing to withhold; an account is a way to carry a balance to
    our apps where there *is* something to pay for. Someone arriving on a
    page headed "Account" in a product whose pricing page says "no account"
    is entitled to think the promise just broke, so the reason it has not
    comes before any button.

    Its own route, not a panel over the studio: <openapps-login>'s Google
    button is a full-page redirect out and back, which would throw away the
    photo being worked on. Here there is no state worth losing.
  */
  import { t } from "$lib/i18n.js";
  import Icon from "$ui/Icon.svelte";
  import { ensureConfigured, isSignedIn, onSessionChange, OPENAPPS_BASE_URL } from "$lib/openapps.js";

  let { go } = $props();
  let state = $state("loading"); // loading | ready | failed
  let signedIn = $state(false);

  $effect(() => {
    let cancelled = false;
    ensureConfigured()
      .then(() => !cancelled && (state = "ready"))
      .catch(() => !cancelled && (state = "failed"));
    return () => (cancelled = true);
  });

  // Watched rather than read once: the sign-in completes without a
  // navigation, so a value read at mount would still say "signed out".
  $effect(() => {
    let stop;
    let cancelled = false;
    const sync = () => isSignedIn().then((v) => !cancelled && (signedIn = v));
    sync();
    onSessionChange(sync).then((off) => (cancelled ? off?.() : (stop = off)));
    return () => {
      cancelled = true;
      stop?.();
    };
  });
</script>

<div class="stack-lg" style="padding-top:var(--space-5)">
  <div>
    <h1>{$t("account.title")}</h1>
    <p class="oa-lead">{$t("account.lede")}</p>
  </div>

  <div class="card stack">
    <section>
      <h3><Icon name="check" size={16} /> {$t("account.free.title")}</h3>
      <p class="tiny">{$t("account.free.body")}</p>
    </section>
    <section>
      <h3><Icon name="shield" size={16} /> {$t("account.private.title")}</h3>
      <p class="tiny">{$t("account.private.body")}</p>
    </section>
  </div>

  {#if state === "loading"}
    <p class="muted">{$t("account.loading")}</p>
  {:else if state === "failed"}
    <!-- The common cause is CORS, which a browser reports exactly like a dead
         server, so the message names both and does not guess. -->
    <div class="card">
      <h3>{$t("account.offline.title")}</h3>
      <p class="tiny">{$t("account.offline.body")}</p>
      <p class="tiny muted">{OPENAPPS_BASE_URL}</p>
    </div>
  {:else}
    <div class="card stack" data-testid="account-panel">
      <!-- The framing is ours: heading, description and mark are supported
           properties. The default heading names a service the visitor has
           never heard of, above a request for a password. No `return-to`:
           the login element declares none; the code comes back in the
           fragment and App.svelte routes it here. -->
      <openapps-login
        variant="panel"
        mark="▣"
        heading={$t("account.signin.title")}
        description={$t("account.signin.body")}
      ></openapps-login>
      <!-- Only once there is a session: signed out, each of these renders
           its own "Sign in to…" placeholder under a panel that already asks. -->
      {#if signedIn}
        <openapps-credits poll-seconds="30"></openapps-credits>
        <openapps-account></openapps-account>
        <openapps-buy></openapps-buy>
        <openapps-history></openapps-history>
        <openapps-signout></openapps-signout>
      {/if}
    </div>
  {/if}

  <nav class="footer">
    <button class="inline" onclick={() => go("home")}>{$t("nav.home")}</button>
    <button class="inline" onclick={() => go("about")}>{$t("nav.about")}</button>
  </nav>
</div>

<style>
  openapps-login, openapps-credits, openapps-account, openapps-buy, openapps-history, openapps-signout {
    display: block;
  }
  section h3 {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }
  .footer {
    display: flex;
    gap: var(--space-5);
    justify-content: center;
    padding-top: var(--space-4);
    border-top: var(--border-width) solid var(--border-hairline);
  }
</style>
