<script>
  // M0 canvas spike: 40 MP synthetic image, zoom/pan via CSS transform,
  // before/after clip-path slider, FPS meter. This is the go/no-go test for
  // the webview canvas route (PLAN.md §6 M0) — kept as a standalone tab so
  // it stays runnable independent of the real photo pipeline.
  let imgUrl = $state("");
  let status = $state("generating 40 MP test image…");
  let zoom = $state(0.08);
  let ox = $state(0);
  let oy = $state(0);
  let split = $state(50);
  let fps = $state(0);

  let dragging = false;
  let lx = 0;
  let ly = 0;

  let frames = 0;
  let last = performance.now();
  function tick(t) {
    frames++;
    if (t - last >= 1000) {
      fps = frames;
      frames = 0;
      last = t;
    }
    requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);

  async function generate() {
    const W = 8000;
    const H = 5000; // 40 MP
    const t0 = performance.now();
    const c = new OffscreenCanvas(W, H);
    const ctx = c.getContext("2d");
    const g = ctx.createLinearGradient(0, 0, W, H);
    g.addColorStop(0, "#1e3a8a");
    g.addColorStop(1, "#f59e0b");
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, W, H);
    for (let i = 0; i < 5000; i++) {
      ctx.fillStyle = `hsl(${(i * 7) % 360} 70% 60% / 0.5)`;
      ctx.beginPath();
      ctx.arc(Math.random() * W, Math.random() * H, 5 + Math.random() * 120, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.fillStyle = "#fff";
    ctx.font = "200px sans-serif";
    ctx.fillText("OpenPhotoId 40 MP canvas spike", 200, 400);
    const blob = await c.convertToBlob({ type: "image/jpeg", quality: 0.8 });
    imgUrl = URL.createObjectURL(blob);
    status = `40 MP (${W}×${H}) ready in ${Math.round(performance.now() - t0)} ms — drag to pan, wheel to zoom`;
  }
  generate();

  function onWheel(e) {
    e.preventDefault();
    const f = e.deltaY < 0 ? 1.1 : 1 / 1.1;
    zoom = Math.min(8, Math.max(0.02, zoom * f));
  }
  function onDown(e) {
    dragging = true;
    lx = e.clientX;
    ly = e.clientY;
  }
  function onMove(e) {
    if (!dragging) return;
    ox += e.clientX - lx;
    oy += e.clientY - ly;
    lx = e.clientX;
    ly = e.clientY;
  }
  function onUp() {
    dragging = false;
  }

  const PAN_STEP = 60;
  const ZOOM_STEP = 1.1;
  function onKeydown(e) {
    switch (e.key) {
      case "ArrowLeft":
        ox += PAN_STEP;
        break;
      case "ArrowRight":
        ox -= PAN_STEP;
        break;
      case "ArrowUp":
        oy += PAN_STEP;
        break;
      case "ArrowDown":
        oy -= PAN_STEP;
        break;
      case "+":
      case "=":
        zoom = Math.min(8, zoom * ZOOM_STEP);
        break;
      case "-":
        zoom = Math.max(0.02, zoom / ZOOM_STEP);
        break;
      default:
        return;
    }
    e.preventDefault();
  }
</script>

<svelte:window onmousemove={onMove} onmouseup={onUp} />

<div class="spike">
  <header>
    <span>{status}</span>
    <span class="fps">{fps} fps</span>
    <label>
      before/after
      <input type="range" min="0" max="100" bind:value={split} />
    </label>
  </header>
  <!--
    A custom pan/zoom viewport has no better-fitting native or ARIA role:
    role="application" would suppress normal screen-reader navigation of
    everything inside it (a recognized anti-pattern), and no interactive
    role models "draggable/zoomable region" accurately. Real keyboard
    equivalents (arrows to pan, +/- to zoom) and a visible focus ring are
    provided below as the actual accessibility improvement; the two
    svelte-ignore comments suppress the linter's "pick a real interactive
    role" suggestion, which doesn't have a good answer here.
  -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="viewport"
    role="img"
    tabindex="0"
    aria-label="40 megapixel zoom and pan test image. Use arrow keys to pan, plus and minus to zoom, or drag and scroll with a mouse."
    onwheel={onWheel}
    onmousedown={onDown}
    onkeydown={onKeydown}
  >
    {#if imgUrl}
      <div class="stack" style="transform: translate({ox}px, {oy}px) scale({zoom});">
        <img src={imgUrl} alt="" draggable="false" />
        <img
          src={imgUrl}
          alt=""
          class="after"
          draggable="false"
          style="clip-path: inset(0 0 0 {split}%);"
        />
      </div>
    {/if}
  </div>
</div>

<style>
  .spike {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background: #1c1c1e;
    font-size: 0.85rem;
  }
  .fps {
    color: #4ade80;
    min-width: 4rem;
  }
  .viewport {
    flex: 1;
    overflow: hidden;
    cursor: grab;
    position: relative;
  }
  .viewport:focus-visible {
    outline: 2px solid #4ade80;
    outline-offset: -2px;
  }
  .viewport:active {
    cursor: grabbing;
  }
  .stack {
    position: absolute;
    transform-origin: 0 0;
    will-change: transform;
  }
  .stack img {
    display: block;
    user-select: none;
    -webkit-user-select: none;
  }
  .stack .after {
    position: absolute;
    inset: 0;
    filter: saturate(0.15) brightness(1.15);
  }
</style>
