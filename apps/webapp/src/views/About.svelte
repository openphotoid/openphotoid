<script>
  import { t } from "$lib/i18n.js";
  import Icon from "$ui/Icon.svelte";

  /*
    The pricing page for a product with no prices.

    It exists because the competitive research found the single most
    common complaint in this whole category is being asked for money
    *after* doing the work — so the answer has to be somewhere obvious and
    stated plainly, not inferred from the absence of a Buy button.

    The premium list is the same set of features every competitor gates,
    each with a sentence about how it actually works here. Saying "AI" and
    leaving it there would be the same vagueness this page exists to
    reject.
  */
  const premium = [
    { icon: "sparkles", key: "about.f1" },
    { icon: "files", key: "about.f2" },
    { icon: "palette", key: "about.f3" },
    { icon: "image", key: "about.f4" },
    { icon: "shirt", key: "about.f5" },
  ];

</script>

<div class="stack-lg" style="padding-top:var(--space-5)">
  <header>
    <h1>{$t("about.title")}</h1>
    <p class="big">{$t("about.free")}</p>
    <p class="oa-lead">{$t("about.body")}</p>
  </header>

  <section class="stack">
    <h2>{$t("about.premium.title")}</h2>
    <p class="muted">{$t("about.premium.body")}</p>
    <ul class="premium">
      {#each premium as f (f.title.en)}
        <li>
          <Icon name={f.icon} size={18} />
          <div>
            <strong>
              {$t(`${f.key}.title`)}
              <span class="free-tag">{$t("common.free")}</span>
            </strong>
            <p class="tiny">{$t(`${f.key}.body`)}</p>
          </div>
        </li>
      {/each}
    </ul>
  </section>

  <section class="card">
    <h2>{$t("about.privacy.title")}</h2>
    <p class="muted">{$t("about.privacy.body")}</p>
  </section>

  <section class="card">
    <h2>{$t("about.source.title")}</h2>
    <p class="muted">{$t("about.source.body")}</p>
  </section>
</div>

<style>
  .big {
    font: var(--type-h3);
    color: var(--text-strong);
    margin: var(--space-3) 0;
  }
  h2 {
    font: var(--type-h4);
    margin-bottom: var(--space-2);
  }
  .premium {
    list-style: none;
    padding: 0;
    display: grid;
    gap: var(--space-5);
  }
  .premium li {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    color: var(--text-muted);
  }
  .premium strong {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
    color: var(--text-strong);
    font-weight: var(--weight-medium);
    margin-bottom: 3px;
  }
</style>
