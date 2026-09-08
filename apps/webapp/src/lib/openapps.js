/**
 * The account integration: where it talks to, and what it does not do.
 *
 * WHAT AN ACCOUNT IS FOR HERE, AND WHAT IT IS NOT FOR
 *
 * It gates nothing. Every feature on this site runs on the visitor's own
 * device, which costs us nothing per photo, so there is nothing to meter and
 * nothing to withhold — that is the product's whole argument for being
 * free, and an account does not change it. Signing in does exactly one
 * thing: it carries a balance that our other apps, where there *is*
 * server-side work to pay for, can spend.
 *
 * So there is no app key anywhere in this app, no `charge` call, and no
 * entitlement check. If a paid feature is ever added here it belongs behind
 * the gateway, and the copy on the website has to change with it.
 *
 * ONE PLACE FOR EVERY URL
 *
 * Both hostnames are defined here and imported everywhere else. A grep for
 * `openapps.network` anywhere else in `src/` should return nothing, and the
 * e2e suite asserts exactly that.
 */

/**
 * The shared account backend, under this product's own name. Same box, same
 * service, same ledger as the platform's own hostname; sessions are bearer
 * tokens, so a second hostname changes nothing functionally. It exists so
 * that signing in never shows a stranger's domain to someone who has only
 * ever heard of OpenPhotoId.
 *
 * What this does not hide: the Google sign-in visibly bounces through the
 * backend's own hostname on the OAuth callback hop until the redirect URI
 * for this host is registered with Google (see the deploy notes).
 */
export const OPENAPPS_BASE_URL = "https://auth.openphotoid.com";

/**
 * The gateway, which holds the app key that turns a user's token into a
 * charge. Unused by this app today — nothing here is chargeable — and named
 * anyway so that the day something is, the URL is already in the one place
 * URLs live.
 */
export const OPENAPPS_GATEWAY_URL = "https://gateway.openphotoid.com";

/**
 * Is this page load the tail end of a sign-in?
 *
 * The session is only ever picked up by the SDK's `completeRedirect()`,
 * which runs from the login component's `connectedCallback` and reads the
 * code from `location.hash` and nowhere else. This app is a hash router: it
 * would read `#code=…` as a route name, match nothing, and fall through to
 * Home — where no account component mounts and the code sits unread in the
 * address bar. So the return is detected *before* the router picks a view.
 *
 * `?account=1` is a second, unrelated entry point — a plain link to the
 * account screen that survives the server's "return_to must not contain a
 * fragment" rule.
 */
export function isSignInReturn(hash = location.hash, search = location.search) {
  if (new URLSearchParams(hash.replace(/^#/, "")).has("code")) return true;
  return new URLSearchParams(search).has("account");
}

/**
 * The SDK deletes the provider's code from the fragment once exchanged,
 * which empties the hash and fires `hashchange`. Callers use this to keep
 * the account view put across that one transition.
 */
export function isConsumedSignInReturn(hash = location.hash) {
  return hash === "" || hash === "#";
}

let configured = false;

/**
 * Point the shared client at our host. Idempotent. Importing the bundle is
 * what registers `<openapps-login>` and friends as custom elements; it is
 * loaded here rather than at start-up so a visitor who never opens the
 * account page never downloads it.
 */
export async function ensureConfigured() {
  if (configured) return;
  const { configure } = await import("../vendor/openapps/openapps-ui.js");
  configure({ baseUrl: OPENAPPS_BASE_URL });
  configured = true;
}

/** Subscribe to sign-in and sign-out. Returns an unsubscribe. */
export async function onSessionChange(fn) {
  await ensureConfigured();
  const { onChange } = await import("../vendor/openapps/openapps-ui.js");
  return onChange(fn);
}

/** Is there a session right now? */
export async function isSignedIn() {
  const c = await client();
  return c?.isLoggedIn ?? false;
}

/** The live client, or null before {@link ensureConfigured} has run. */
export async function client() {
  await ensureConfigured();
  const { getClient } = await import("../vendor/openapps/openapps-ui.js");
  return getClient();
}
