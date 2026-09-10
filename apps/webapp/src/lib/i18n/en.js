// English: the source every other catalogue is translated from.
export default {
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
  "nav.diagnostics": "Device test",

  "diag.title": "Does it work on this device?",
  "diag.lede": "Runs the whole pipeline on a sample portrait and shows what this browser did with it. Nothing you own is involved.",
  "diag.browser": "Browser",
  "diag.viewport": "Screen",
  "diag.isolated": "Threads possible",
  "diag.run": "Run the test",
  "diag.provider": "Ran on",
  "diag.source": "Sample photo",
  "diag.total": "Total",
  "diag.noSession": "This browser's WebAssembly engine could not start the model. This is known on iOS 17; iOS 18 or later, and any current Chrome or Firefox, run it.",
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

  "about.f1.title": "Hair-level edge detail",
  "about.f1.body":
    "Sold as an AI upgrade elsewhere. Here the separation runs a second time zoomed in on your head, so hair is resolved at the model's full resolution instead of a fraction of it.",
  "about.f2.title": "Unlimited batches",
  "about.f2.body":
    "Competitors cap the free tier at a handful and sell monthly quotas of a thousand. There is no cap here, because the work happens on your device and costs us nothing.",
  "about.f3.title": "Backgrounds beyond flat colour",
  "about.f3.body":
    "Gradients, a studio sweep, or your own picture with adjustable blur. Not a generative model inventing a scene — that would need hundreds of megabytes and a server, and no document accepts one anyway.",
  "about.f4.title": "Print sharpening",
  "about.f4.body":
    "Marketed as “HD restore”. It restores the crispness that resizing costs, using the detail your photo already has. It does not invent detail — on a document that has to match your face, that is the point.",
  "about.f5.title": "Formal clothes",
  "about.f5.body":
    "A jacket drawn to fit your face and shoulders. It is a template, and the app says so: a garment-aware generative model is the one feature on this list a browser genuinely cannot run, and pretending otherwise would be the dishonest option.",

  "common.close": "Close",
  "common.retry": "Try again",
  "common.free": "Free",
  "common.loading": "Loading…",
  "common.downloading": "Downloading the one-time setup, {pct}%",
  "common.error": "Something went wrong",
};
