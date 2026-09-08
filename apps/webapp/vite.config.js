import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

// Relative base: the build is a folder of static files that has to work
// from a subpath, a file host, or a CDN without being rebuilt for each.
export default defineConfig({
  base: "./",
  plugins: [svelte()],
  resolve: {
    alias: {
      // The OpenApps design tokens, shared with the desktop app rather
      // than copied — the two drift the moment there are two of them.
      // Only the CSS is shared; the kit's Svelte components are not,
      // because its Icon pulls glyphs from a CDN and this build has to
      // work with no network at all. See src/ui/Icon.svelte.
      $design: fileURLToPath(new URL("../desktop/src/design", import.meta.url)),
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
      $ui: fileURLToPath(new URL("./src/ui", import.meta.url)),

      // Point at onnxruntime-web's *non-bundled* builds.
      //
      // The default export condition resolves to `ort.*.bundle.min.mjs`,
      // which reaches its WebAssembly through `new URL(..., import.meta.url)`.
      // Rollup sees that, emits its own content-hashed copies of both
      // runtimes into assets/ — 40 MB — and then nothing loads them,
      // because src/lib/ort.js points `wasmPaths` at ./ort/ instead.
      // The non-bundled builds fetch both the glue and the binary from
      // `wasmPaths` at runtime, so exactly one runtime is downloaded and
      // the bundler never sees a wasm import at all.
      "onnxruntime-web/webgpu": fileURLToPath(
        new URL("./node_modules/onnxruntime-web/dist/ort.webgpu.min.mjs", import.meta.url),
      ),
      "onnxruntime-web/wasm": fileURLToPath(
        new URL("./node_modules/onnxruntime-web/dist/ort.wasm.min.mjs", import.meta.url),
      ),
    },
  },
  build: {
    target: "es2022",
    // The two ONNX runtimes are large and mutually exclusive; keeping
    // them out of the main chunk is what lets a visitor download only
    // the one their browser can use.
    chunkSizeWarningLimit: 2048,
  },
  worker: { format: "es" },
  server: { port: 8082 },
});
