<script>
  import { t, lang } from "$lib/i18n.js";
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
    {
      icon: "sparkles",
      title: { en: "Hair-level edge detail", zh: "发丝级抠图" },
      body: {
        en: "Sold as an AI upgrade elsewhere. Here the separation runs a second time zoomed in on your head, so hair is resolved at the model's full resolution instead of a fraction of it.",
        zh: "别家作为 AI 升级功能收费。这里是对着头部再抠一次图，头发按模型的完整分辨率处理，而不是只占其中一小块。",
      },
    },
    {
      icon: "files",
      title: { en: "Unlimited batches", zh: "批量处理不限张数" },
      body: {
        en: "Competitors cap the free tier at a handful and sell monthly quotas of a thousand. There is no cap here, because the work happens on your device and costs us nothing.",
        zh: "同类产品免费版限几张，再按月卖上千张的额度。这里不限，因为处理发生在你的设备上，我们不产生任何成本。",
      },
    },
    {
      icon: "palette",
      title: { en: "Backgrounds beyond flat colour", zh: "纯色以外的背景" },
      body: {
        en: "Gradients, a studio sweep, or your own picture with adjustable blur. Not a generative model inventing a scene — that would need hundreds of megabytes and a server, and no document accepts one anyway.",
        zh: "渐变、影楼灰、或者你自己的图片加可调虚化。不是用生成模型凭空造场景——那需要几百兆和一台服务器，而且证件也不接受。",
      },
    },
    {
      icon: "image",
      title: { en: "Print sharpening", zh: "打印锐化" },
      body: {
        en: "Marketed as “HD restore”. It restores the crispness that resizing costs, using the detail your photo already has. It does not invent detail — on a document that has to match your face, that is the point.",
        zh: "别处叫“高清修复”。它把缩放损失的清晰度还原回来，用的是照片本来就有的细节。它不会凭空生成细节——证件照必须跟本人对得上，这一点很重要。",
      },
    },
    {
      icon: "shirt",
      title: { en: "Formal clothes", zh: "正装" },
      body: {
        en: "A jacket drawn to fit your face and shoulders. It is a template, and the app says so: a garment-aware generative model is the one feature on this list a browser genuinely cannot run, and pretending otherwise would be the dishonest option.",
        zh: "按你的脸型和肩线画出来的西装。它是模板，界面上也这么写：真正的生成式换装是这份清单里浏览器唯一做不到的，假装能做才是不老实。",
      },
    },
  ];

  const isZh = $derived($lang === "zh-CN");
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
              {isZh ? f.title.zh : f.title.en}
              <span class="free-tag">{$t("common.free")}</span>
            </strong>
            <p class="tiny">{isZh ? f.body.zh : f.body.en}</p>
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
