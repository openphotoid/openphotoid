<script>
  import { t, lang } from "$lib/i18n.js";
  import { byCountry, documentLabel, outputPx, sizeWindow } from "$lib/specs.js";

  let { specs = [] } = $props();
  const groups = $derived(byCountry(specs, $lang));
</script>

<div class="stack" style="padding-top:var(--space-5)">
  <h1>{$t("coverage.title")}</h1>
  <p class="oa-lead">{$t("coverage.intro")}</p>

  {#each groups as group (group.code)}
    <section class="card">
      <h2>{group.name}</h2>
      <table>
        <thead>
          <tr>
            <th>{$t("coverage.document")}</th>
            <th>px</th>
            <th>mm</th>
            <th>KB</th>
            <th>{$t("coverage.verified")}</th>
          </tr>
        </thead>
        <tbody>
          {#each group.specs as spec (spec.id)}
            {@const px = outputPx(spec)}
            {@const win = sizeWindow(spec)}
            <tr>
              <td>
                <span class="name">
                  <span class="dot" style="background:{spec.background.hex_render}"></span>
                  {documentLabel(spec)}
                </span>
                {#if spec.sources?.length}
                  <a href={spec.sources[0]} target="_blank" rel="noreferrer noopener" class="tiny">
                    {$t("coverage.source")}
                  </a>
                {/if}
                {#if spec.rules?.notes}
                  <span class="tiny notes">{spec.rules.notes}</span>
                {/if}
              </td>
              <td class="mono">{px[0]}×{px[1]}</td>
              <td class="mono">
                {spec.print ? `${spec.print.width_mm}×${spec.print.height_mm}` : "—"}
              </td>
              <td class="mono">
                {win ? `${win.min ?? 0}–${win.max ?? "∞"}` : "—"}
              </td>
              <td class="mono">{spec.last_verified ?? "—"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {/each}

  <p class="tiny">{$t("coverage.missing")}</p>
</div>

<style>
  .card {
    overflow-x: auto;
  }
  h2 {
    font: var(--type-h4);
    margin-bottom: var(--space-3);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.88rem;
  }
  th {
    text-align: left;
    font: var(--type-eyebrow);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--text-faint);
    padding-bottom: var(--space-2);
    white-space: nowrap;
  }
  td {
    padding: var(--space-3) var(--space-3) var(--space-3) 0;
    border-top: var(--border-width) solid var(--border-hairline);
    vertical-align: top;
    color: var(--text-muted);
  }
  td:first-child {
    min-width: 200px;
  }
  .name {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-strong);
    font-weight: var(--weight-medium);
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    border: var(--border-width) solid var(--border-hairline);
    flex: 0 0 auto;
  }
  .notes {
    display: block;
    margin-top: 2px;
    max-width: 42ch;
  }
  a {
    display: block;
    margin-top: 2px;
  }
</style>
