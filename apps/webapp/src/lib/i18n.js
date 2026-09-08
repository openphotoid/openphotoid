/**
 * Two languages, because the go-to-market plan starts in Chinese and the
 * document specs are worldwide.
 *
 * Strings are deliberately plain. The audience for this product is
 * someone who needs a visa photo tonight, not someone who wants to learn
 * what a "spec engine" is — so the UI says "size and rules for the
 * country you picked", and the word "matting" appears nowhere.
 */

import { writable, derived, get } from "svelte/store";

const en = {
  "app.name": "OpenPhotoId",
  "app.tagline": "Passport and ID photos that pass, made on your own phone.",
  "app.sub":
    "Pick the country, take or choose a photo, and get a file that meets the official size, background and head-position rules. Free, with nothing to sign up for.",

  "nav.make": "Make a photo",
  "nav.batch": "Several at once",
  "nav.coverage": "Documents covered",
  "nav.about": "What this costs",
  "nav.back": "Back",
  "nav.home": "Home",
  "nav.account": "Account",

  "account.title": "Account",
  "account.lede":
    "Optional, and it unlocks nothing here. An account carries credits to our other apps; everything on this site stays free either way.",
  "account.signin.title": "Sign in to OpenPhotoId",
  "account.signin.body":
    "One account across our apps. You do not need it here — nothing on this site is behind it.",
  "account.free.title": "Nothing here is behind it",
  "account.free.body":
    "Every feature runs on your own device, so it costs us nothing per photo and there is nothing to charge for. Signed in or not, you get all of it, with no limit.",
  "account.private.title": "Your photo is still not involved",
  "account.private.body":
    "Signing in sends an email address or a wallet signature to our account server, and nothing else. No photo is uploaded, before or after — there is still no server that accepts one.",
  "account.loading": "Loading the account tools…",
  "account.offline.title": "Could not reach the account server",
  "account.offline.body":
    "Everything else on this site works without it — nothing here depends on an account. Try again later.",

  "promise.free.title": "Free, all of it",
  "promise.free.body":
    "Every feature on this page. No account, no trial, no watermark, and no step where a price appears after you have done the work.",
  "promise.private.title": "Your photo stays here",
  "promise.private.body":
    "It is processed inside this browser tab. It is never uploaded, because there is no server to upload it to.",
  "promise.offline.title": "Works with no signal",
  "promise.offline.body":
    "After the first visit everything is stored on your device. Add it to your home screen and it opens on a plane.",
  "promise.offline.ready": "Ready to work offline",

  "home.start": "Choose a photo",
  "home.camera": "Take a photo",
  "home.count": "{n} documents from {c} countries and regions",
  "home.tips.title": "For the best result",
  "home.tips.1": "Face the camera straight on, with a neutral expression.",
  "home.tips.2": "Even light on your face — a window works, direct sun does not.",
  "home.tips.3": "Any background at all. It gets replaced with the one your document requires.",
  "home.tips.4": "Head and shoulders in frame, with a little room above your hair.",

  "picker.title": "Which document?",
  "picker.search": "Search country or document",
  "picker.none": "Nothing matches “{q}”.",
  "picker.recent": "Recently used",
  "picker.size": "{w} × {h} px",
  "picker.print": "{w} × {h} mm",
  "picker.verified": "Rules checked {date}",
  "picker.background": "{name} background",

  "studio.working": "Working…",
  "studio.stage.decoding": "Reading the photo",
  "studio.stage.detecting": "Finding the face",
  "studio.stage.matting": "Separating you from the background",
  "studio.stage.detail": "Sharpening the edges around your hair",
  "studio.stage.rendering": "Applying the document rules",
  "studio.noface":
    "No face found in this photo. Try one where the face is larger and clearly lit.",
  "studio.padded":
    "Your photo was cropped tighter than this document needs, so the edges were extended with the background colour. A photo with more room around the head will look better.",
  "studio.before": "Original",
  "studio.after": "Result",
  "studio.checks": "Official checks",
  "studio.checks.pass": "Passes every check",
  "studio.checks.warn": "Passes, with cautions",
  "studio.checks.fail": "Does not pass yet",
  "studio.retry": "Use a different photo",
  "studio.changeDoc": "Change document",

  "panel.adjust": "Position",
  "panel.adjust.hint":
    "The automatic crop aims for the middle of every allowed range. Move these only if you want to.",
  "panel.adjust.head": "Head size",
  "panel.adjust.eye": "Eye height",
  "panel.adjust.center": "Left / right",
  "panel.adjust.reset": "Back to automatic",

  "panel.background": "Background",
  "panel.background.required": "{name} — required by this document",
  "panel.background.hint":
    "Documents require one flat colour, and that is what is selected. The others are for reusing this photo somewhere less formal.",
  "panel.background.solid": "Flat colour",
  "panel.background.color": "Colour",
  "panel.background.gradient": "Gradient",
  "panel.background.vignette": "Studio",
  "panel.background.image": "Your own picture",
  "panel.background.keep": "Leave as it is",
  "panel.background.choose": "Choose a picture",
  "panel.background.blur": "Blur",

  "panel.finish": "Finishing",
  "panel.finish.retouch": "Soften skin",
  "panel.finish.retouch.hint":
    "Off by default and on purpose. A little evens out lighting; too much stops the photo looking like you, and that is what gets one rejected.",
  "panel.finish.retouch.off": "Off",
  "panel.finish.sharpen": "Sharpen for printing",
  "panel.finish.sharpen.hint":
    "Restores the crispness that resizing costs. It recovers detail that is there — it does not invent any.",
  "panel.finish.quality": "Extra edge detail",
  "panel.finish.quality.hint":
    "Runs the separation a second time, zoomed in on your head. Slower, and noticeably better around hair.",

  "panel.garment": "Formal clothes",
  "panel.garment.hint":
    "A drawn jacket, sized from your face. It is a template, not a photograph of clothing — good enough for most forms, and worth a look before you rely on it.",
  "panel.garment.none": "Keep my clothes",
  "panel.garment.suitTie": "Jacket and tie",
  "panel.garment.suitOpen": "Jacket, open collar",
  "panel.garment.blouse": "Blouse",
  "panel.garment.shirt": "Shirt",
  "panel.garment.jacket": "Jacket",
  "panel.garment.shirtColor": "Shirt",
  "panel.garment.tie": "Tie",

  "export.title": "Save",
  "export.digital": "For uploading",
  "export.digital.hint": "{w} × {h} px, JPEG{size}",
  "export.digital.window": ", {min}–{max} KB",
  "export.digital.max": ", under {max} KB",
  "export.png": "PNG (largest, no compression loss)",
  "export.sheet": "For printing at a shop",
  "export.sheet.hint": "{n} copies on one {sheet} sheet, with cutting guides",
  "export.sheet.none": "This document has no print size, so there is nothing to tile.",
  "export.saved": "Saved",

  "batch.title": "Several photos at once",
  "batch.hint":
    "Every photo gets the same document and the same settings. There is no limit on how many, and no charge — it all runs on this device.",
  "batch.choose": "Choose photos",
  "batch.run": "Process {n}",
  "batch.running": "{done} of {total}",
  "batch.download": "Download all as a folder",
  "batch.downloadOne": "Save",
  "batch.report": "Download the check report (CSV)",
  "batch.clear": "Start over",
  "batch.failed": "Could not process",

  "coverage.title": "Documents covered",
  "coverage.intro":
    "Every document this app knows, the source its rules came from, and the date they were last checked against it. If yours is not here, it is not here — you should not have to find that out after you have finished.",
  "coverage.document": "Document",
  "coverage.source": "Official source",
  "coverage.verified": "Checked",
  "coverage.missing": "Missing one? Tell us which, and it gets added.",

  "about.title": "What this costs",
  "about.free": "Nothing. There is no paid tier.",
  "about.body":
    "Tools like this usually let you do the work and then ask for money to save the result. This one does not have that step, because it has no server bill to cover: the whole thing runs on your device.",
  "about.premium.title": "The features other apps charge for",
  "about.premium.body":
    "These are the ones normally behind a subscription. They are all here, all free, and each is honest about how it works.",
  "about.privacy.title": "Privacy",
  "about.privacy.body":
    "Your photo is read into this tab, processed, and shown back to you. It is not uploaded, not stored on a server, and not seen by us — there is no us to see it. Closing the tab is all the deletion there is to do.",
  "about.source.title": "Source",
  "about.source.body": "Open source, MIT or Apache-2.0.",

  "common.close": "Close",
  "common.retry": "Try again",
  "common.free": "Free",
  "common.loading": "Loading…",
  "common.downloading": "Downloading the one-time setup, {pct}%",
  "common.error": "Something went wrong",
};

const zh = {
  "app.name": "OpenPhotoId",
  "app.tagline": "在自己手机上做证件照，一次就过。",
  "app.sub":
    "选国家、拍张照，直接得到尺寸、背景、头部位置都符合官方要求的文件。免费，不用注册。",

  "nav.make": "做证件照",
  "nav.batch": "批量处理",
  "nav.coverage": "支持的证件",
  "nav.about": "要多少钱",
  "nav.back": "返回",
  "nav.home": "首页",
  "nav.account": "账户",

  "account.title": "账户",
  "account.lede":
    "可选，而且不会解锁这里的任何功能。账户用于把积分带到我们的其他应用；本站的一切始终免费。",
  "account.signin.title": "登录 OpenPhotoId",
  "account.signin.body": "一个账户通用于我们的各个应用。这里并不需要它——本站没有任何功能需要登录。",
  "account.free.title": "这里没有任何功能需要账户",
  "account.free.body":
    "所有功能都在你自己的设备上运行，我们每张照片的成本是零，也就没有可收费的东西。无论是否登录，功能全都可用，且没有次数限制。",
  "account.private.title": "你的照片依然与此无关",
  "account.private.body":
    "登录只会把邮箱地址或钱包签名发送到我们的账户服务器，除此之外没有别的。前后都不会上传照片——依然没有任何服务器会接收照片。",
  "account.loading": "正在加载账户工具……",
  "account.offline.title": "无法连接账户服务器",
  "account.offline.body": "本站其余功能都不依赖账户，可以照常使用。请稍后再试。",

  "promise.free.title": "全部免费",
  "promise.free.body":
    "这个页面上的每一个功能。不注册、不试用、不加水印，也没有“做完了才弹出价格”这一步。",
  "promise.private.title": "照片留在你手机里",
  "promise.private.body": "全部在这个浏览器标签页里处理。不会上传——因为根本没有可上传的服务器。",
  "promise.offline.title": "没网也能用",
  "promise.offline.body": "第一次打开之后就都存在你的设备上。加到桌面，在飞机上也能开。",
  "promise.offline.ready": "已可离线使用",

  "home.start": "选一张照片",
  "home.camera": "拍一张",
  "home.count": "覆盖 {c} 个国家和地区的 {n} 种证件",
  "home.tips.title": "怎么拍效果最好",
  "home.tips.1": "正对镜头，表情自然。",
  "home.tips.2": "脸上光线均匀——窗边就行，别用直射的太阳。",
  "home.tips.3": "背景随便，反正会换成证件要求的颜色。",
  "home.tips.4": "拍到头和肩膀，头顶上方留一点空隙。",

  "picker.title": "办什么证件？",
  "picker.search": "搜国家或证件",
  "picker.none": "没有找到“{q}”。",
  "picker.recent": "最近用过",
  "picker.size": "{w} × {h} 像素",
  "picker.print": "{w} × {h} 毫米",
  "picker.verified": "规格核对于 {date}",
  "picker.background": "{name}底",

  "studio.working": "处理中…",
  "studio.stage.decoding": "正在读取照片",
  "studio.stage.detecting": "正在找人脸",
  "studio.stage.matting": "正在把人和背景分开",
  "studio.stage.detail": "正在处理发丝边缘",
  "studio.stage.rendering": "正在按证件规格裁切",
  "studio.noface": "这张照片里没找到人脸。换一张脸更大、光线更清楚的试试。",
  "studio.padded":
    "你的照片比这个证件需要的范围更紧，边缘用背景色补上了。换一张头部周围留白更多的会更好看。",
  "studio.before": "原图",
  "studio.after": "结果",
  "studio.checks": "官方规格检查",
  "studio.checks.pass": "全部通过",
  "studio.checks.warn": "通过，但有提醒",
  "studio.checks.fail": "还不符合",
  "studio.retry": "换一张照片",
  "studio.changeDoc": "换证件",

  "panel.adjust": "位置",
  "panel.adjust.hint": "自动裁切已经取了每项允许范围的中间值。想调再动。",
  "panel.adjust.head": "头部大小",
  "panel.adjust.eye": "眼睛高度",
  "panel.adjust.center": "左右位置",
  "panel.adjust.reset": "恢复自动",

  "panel.background": "背景",
  "panel.background.required": "{name}——这个证件要求的颜色",
  "panel.background.hint":
    "证件只认一种纯色，已经选好了。下面几种是给你把这张照片用在别处时用的。",
  "panel.background.solid": "纯色",
  "panel.background.color": "颜色",
  "panel.background.gradient": "渐变",
  "panel.background.vignette": "影楼灰",
  "panel.background.image": "自己的图片",
  "panel.background.keep": "保持原样",
  "panel.background.choose": "选一张图片",
  "panel.background.blur": "虚化",

  "panel.finish": "修饰",
  "panel.finish.retouch": "磨皮",
  "panel.finish.retouch.hint":
    "默认关闭，这是故意的。轻微一点能让肤色均匀；过头就不像本人了，而不像本人正是被退回的原因。",
  "panel.finish.retouch.off": "关闭",
  "panel.finish.sharpen": "打印锐化",
  "panel.finish.sharpen.hint": "把缩放损失的清晰度补回来。只还原本来就有的细节，不会凭空生成。",
  "panel.finish.quality": "发丝细节增强",
  "panel.finish.quality.hint": "对着头部再抠一次图。慢一点，头发边缘明显更干净。",

  "panel.garment": "正装",
  "panel.garment.hint":
    "按你的脸型画出来的西装，是模板不是真实衣服照片。大多数表格够用，用之前自己看一眼。",
  "panel.garment.none": "穿我自己的",
  "panel.garment.suitTie": "西装打领带",
  "panel.garment.suitOpen": "西装不打领带",
  "panel.garment.blouse": "女士上衣",
  "panel.garment.shirt": "只穿衬衫",
  "panel.garment.jacket": "西装",
  "panel.garment.shirtColor": "衬衫",
  "panel.garment.tie": "领带",

  "export.title": "保存",
  "export.digital": "网上提交用",
  "export.digital.hint": "{w} × {h} 像素，JPEG{size}",
  "export.digital.window": "，{min}–{max} KB",
  "export.digital.max": "，小于 {max} KB",
  "export.png": "PNG（最大，无压缩损失）",
  "export.sheet": "拿去照相馆打印",
  "export.sheet.hint": "一张 {sheet} 相纸排 {n} 张，带裁切线",
  "export.sheet.none": "这个证件没有打印尺寸，排不了版。",
  "export.saved": "已保存",

  "batch.title": "一次处理多张",
  "batch.hint": "所有照片用同一个证件规格和同一套设置。张数不限、不收费——全在这台设备上跑。",
  "batch.choose": "选照片",
  "batch.run": "处理 {n} 张",
  "batch.running": "{done} / {total}",
  "batch.download": "全部下载",
  "batch.downloadOne": "保存",
  "batch.report": "下载检查报告（CSV）",
  "batch.clear": "重新开始",
  "batch.failed": "处理失败",

  "coverage.title": "支持的证件",
  "coverage.intro":
    "这里是全部支持的证件、规格出处，以及最后一次核对的日期。没列出来的就是不支持——这种事不该等你做完了才发现。",
  "coverage.document": "证件",
  "coverage.source": "官方来源",
  "coverage.verified": "核对于",
  "coverage.missing": "缺你要的那个？告诉我们是哪个，会加进来。",

  "about.title": "要多少钱",
  "about.free": "不要钱，没有付费版。",
  "about.body":
    "同类工具通常让你先做完，再收钱才让你保存。这里没有那一步，因为没有服务器账单要摊——全部跑在你自己的设备上。",
  "about.premium.title": "别家收费的那些功能",
  "about.premium.body": "下面这些通常要订阅。这里全都有、全都免费，而且每一项都会说清楚它到底是怎么做的。",
  "about.privacy.title": "隐私",
  "about.privacy.body":
    "照片读进这个标签页，处理完显示给你。不上传、不存服务器、我们也看不到——根本没有“我们”能看到。关掉标签页就等于删干净了。",
  "about.source.title": "源代码",
  "about.source.body": "开源，MIT 或 Apache-2.0 协议。",

  "common.close": "关闭",
  "common.retry": "重试",
  "common.free": "免费",
  "common.loading": "加载中…",
  "common.downloading": "首次准备中，{pct}%",
  "common.error": "出错了",
};

const DICTS = { en, "zh-CN": zh };

function detect() {
  const saved = localStorage.getItem("openphotoid.lang") ?? localStorage.getItem("openpassport.lang");
  if (saved && DICTS[saved]) return saved;
  const nav = navigator.languages ?? [navigator.language ?? "en"];
  return nav.some((l) => l.toLowerCase().startsWith("zh")) ? "zh-CN" : "en";
}

export const lang = writable(detect());

lang.subscribe((v) => {
  try {
    localStorage.setItem("openphotoid.lang", v);
    document.documentElement.lang = v;
  } catch {
    /* private mode: the language just does not persist */
  }
});

/** `t("key", { n: 3 })` — missing keys render as the key, loudly. */
export const t = derived(lang, ($lang) => (key, vars) => {
  const dict = DICTS[$lang] ?? en;
  let s = dict[key] ?? en[key] ?? key;
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
});

export const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "zh-CN", label: "简体中文" },
];

export const currentLang = () => get(lang);
