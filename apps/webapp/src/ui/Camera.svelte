<script>
  /*
    The webcam, in the page, for a computer.

    On a phone "Take a photo" is a file input with `capture="user"`, which
    hands over to the system camera — the better camera app there. Every
    desktop browser ignores `capture`, so on a computer that same input opened
    the file picker and the button looked broken (APP-97). This is the
    computer's version: a live preview, a shutter, a retake, and a JPEG handed
    to the same `onphoto` the file input feeds.

    The frames go from the camera to a <video> and a <canvas> in this tab and
    nowhere else. The stream is stopped whenever the dialog closes, however it
    closes, so the camera light never outlives the dialog.
  */
  import { t } from "$lib/i18n.js";
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";

  let { onphoto, onfallback } = $props();

  let dialog;
  let video = $state();
  let stream = null;
  // starting | live | taken | error
  let phase = $state("starting");
  let problem = $state("");
  let still = $state(null);

  export function open() {
    dialog.showModal();
    start();
  }

  async function start() {
    phase = "starting";
    problem = "";
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        audio: false,
        // As many pixels as the camera has: the checks want enough of them
        // between the eyes, and a 640×480 default often does not have them.
        video: { facingMode: "user", width: { ideal: 1920 }, height: { ideal: 1080 } },
      });
      // Closed while the browser was still asking for permission.
      if (!dialog.open) return stop();
      video.srcObject = stream;
      await video.play();
      phase = "live";
    } catch (e) {
      stop();
      problem =
        e?.name === "NotAllowedError" || e?.name === "SecurityError"
          ? "camera.denied"
          : e?.name === "NotFoundError" || e?.name === "OverconstrainedError"
            ? "camera.none"
            : e?.name === "NotReadableError"
              ? "camera.busy"
              : "camera.failed";
      phase = "error";
    }
  }

  function stop() {
    stream?.getTracks().forEach((track) => track.stop());
    stream = null;
    if (video) video.srcObject = null;
  }

  function capture() {
    const canvas = document.createElement("canvas");
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    // Drawn as the camera sees it, not as the mirrored preview shows it: a
    // passport photo is the face as other people see it.
    canvas.getContext("2d").drawImage(video, 0, 0);
    canvas.toBlob(
      (blob) => {
        if (!blob) return;
        still = { blob, url: URL.createObjectURL(blob) };
        video.pause();
        phase = "taken";
      },
      "image/jpeg",
      0.95,
    );
  }

  function retake() {
    drop();
    video.play();
    phase = "live";
  }

  function drop() {
    if (still) URL.revokeObjectURL(still.url);
    still = null;
  }

  function use() {
    const file = new File([still.blob], `camera-${Date.now()}.jpg`, { type: "image/jpeg" });
    shut();
    onphoto(file);
  }

  function fallback() {
    shut();
    onfallback();
  }

  // Every way out ends in `closed`, which is what makes "the camera is off
  // when the dialog is gone" true rather than hoped for. The ways out this
  // component owns call it directly, because the dialog's `close` event is
  // queued as a task of its own: the dialog is already gone from the screen
  // by then, with the camera still on. The event stays for Esc, which only
  // the browser sees.
  function shut() {
    closed();
    dialog.close();
  }

  function closed() {
    stop();
    drop();
    phase = "starting";
  }
</script>

<dialog bind:this={dialog} class="camera" aria-labelledby="camera-title" onclose={closed}>
  <header>
    <h2 id="camera-title">{$t("camera.title")}</h2>
    <button class="x" onclick={shut} aria-label={$t("camera.close")} title={$t("camera.close")}>
      <Icon name="x" size={18} />
    </button>
  </header>

  <div class="stage" class:off={phase === "error"}>
    <!-- svelte-ignore a11y_media_has_caption -->
    <video bind:this={video} playsinline muted hidden={phase === "taken"}></video>
    {#if still}
      <img src={still.url} alt={$t("camera.taken")} />
    {/if}
    {#if phase === "live"}
      <!-- Where the head goes. A guide, not a crop: the crop to the
           document's rules happens later, on the photo itself. -->
      <svg class="guide" viewBox="0 0 160 90" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
        <ellipse cx="80" cy="42" rx="19" ry="26" />
      </svg>
    {/if}
    {#if phase === "starting"}
      <p class="status"><Icon name="loader" size={18} style="animation:oa-spin 900ms linear infinite" /> {$t("camera.starting")}</p>
    {/if}
    {#if phase === "error"}
      <div class="status problem" role="alert">
        <Icon name="alert" size={20} />
        <p>{$t(problem)}</p>
      </div>
    {/if}
  </div>

  <p class="hint">{phase === "error" ? $t("camera.private") : `${$t("camera.hint")} ${$t("camera.private")}`}</p>

  <div class="actions">
    {#if phase === "taken"}
      <Button variant="secondary" size="lg" icon="left" onclick={retake}>{$t("camera.retake")}</Button>
      <Button size="lg" icon="check" onclick={use}>{$t("camera.use")}</Button>
    {:else if phase === "error"}
      <Button variant="secondary" size="lg" onclick={start}>{$t("camera.retry")}</Button>
      <Button size="lg" icon="image" onclick={fallback}>{$t("camera.fallback")}</Button>
    {:else}
      <Button size="lg" icon="camera" disabled={phase !== "live"} onclick={capture}>{$t("camera.capture")}</Button>
    {/if}
  </div>
</dialog>

<style>
  .camera {
    width: min(760px, calc(100vw - 2 * var(--space-4)));
    max-height: calc(100dvh - 2 * var(--space-4));
    padding: var(--space-5);
    border: var(--border-width) solid var(--border-hairline);
    border-radius: var(--radius-xl, var(--radius-lg));
    background: var(--surface-raised);
    color: var(--text-body);
    box-shadow: var(--shadow-lg, none);
  }
  .camera::backdrop {
    background: color-mix(in oklab, var(--black) 55%, transparent);
    backdrop-filter: blur(4px);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }
  h2 {
    font: var(--weight-medium) var(--text-lg) / 1.2 var(--font-display);
    color: var(--text-strong);
    margin: 0;
  }
  .x {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--control-h-md);
    height: var(--control-h-md);
    border: 0;
    border-radius: var(--radius-full, 999px);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
  }
  .x:hover {
    color: var(--text-strong);
    background: var(--surface-hover);
  }
  .stage {
    position: relative;
    aspect-ratio: 16 / 9;
    max-width: 100%;
    border-radius: var(--radius-lg);
    overflow: hidden;
    /* The camera's own dark, in both themes: a preview is a picture. */
    background: var(--black);
  }
  .stage.off {
    background: var(--bg-sunken);
  }
  video,
  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    /* A mirror, which is what a person expects to see of themselves. The
       photo itself is saved the right way round (see capture()). */
    transform: scaleX(-1);
  }
  video[hidden] {
    display: none;
  }
  .guide {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  .guide ellipse {
    fill: none;
    stroke: var(--white);
    stroke-opacity: 0.75;
    stroke-width: 0.6;
    stroke-dasharray: 2 1.6;
  }
  .status {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    margin: 0;
    padding: var(--space-6);
    color: var(--white);
    font: var(--type-ui);
    text-align: center;
  }
  .problem {
    flex-direction: column;
    color: var(--text-body);
  }
  .problem p {
    margin: 0;
    max-width: 44ch;
    font: var(--type-body);
  }
  .problem :global(svg) {
    color: var(--text-muted);
  }
  .hint {
    margin: var(--space-3) 0 0;
    font: var(--type-caption);
    color: var(--text-faint);
    text-align: center;
  }
  .actions {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-top: var(--space-5);
  }
  .actions :global(.btn) {
    min-width: 12rem;
  }
</style>
