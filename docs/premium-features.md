# The features competitors charge for

The market research (`docs/research/`, §五 推荐功能清单) split this
category's features into two lists: the four that every competitor gives
away, and five that every competitor gates behind a subscription. The
four free ones were already OpenPhotoId's MVP. This document is about the
other five.

All five now ship, and all five are free. This is what each one actually
does, and — where it differs from what a competitor sells under the same
name — why.

The rule they follow is the one the research argued for (§九 切入点1):
the complaint that shows up in five of seven competitors' review pages is
not price, it is *finding out about the price after doing the work*. A
feature that is free and clearly explained beats one that is impressive
and conditional.

---

## 1. Hair-level edge detail

**Sold elsewhere as:** "AI 高精度抠图", an upgrade tier.
**Here:** `Matte::blend_region` — a second matting pass over the head.

MODNet has a fixed 512×512 input. On a half-body photo the head occupies
maybe a fifth of that, so hair — the part that is hard — is resolved at
roughly 200 px across. The detail pass crops to the head, runs the same
model again on that crop, and merges the result back into the full-frame
matte with a feathered seam.

The cost is one extra inference at the same size. The gain is real
resolution where the alpha is difficult. `Session::detail_input` skips
the pass when the head already fills the frame, because then the crop is
the image and the second run buys nothing.

**What was considered and rejected:** BiRefNet-lite, which the desktop
already has code for. It is 224 MB — not a download to hand a phone on a
mobile connection — and `MODELS.md` still records its inference as
unverified after it OOMed on the dev VM. Shipping a quarter-gigabyte
download of a model we have not confirmed works would be worse than the
cheap trick that does.

## 2. Unlimited batch

**Sold elsewhere as:** free tiers capped at 9 photos; paid tiers at
1,000–10,000 exports a month.
**Here:** no cap.

There is nothing to meter. The work happens in the visitor's browser and
costs the operator nothing, so a limit would exist purely to create a
reason to charge — which is the pattern the research identified as this
category's main source of ill will.

Photos are processed one at a time rather than concurrently: parallel
runs multiply peak WebAssembly heap by the concurrency and will take a
mobile browser down on a large set. A per-item CSV report comes out
alongside, the same one the desktop batch writes.

## 3. Backgrounds beyond a flat colour

**Sold elsewhere as:** "AI 背景生成", billed per generation.
**Here:** `frame_retouch::backdrop` — gradients, a studio vignette, or the
visitor's own image with adjustable blur, composited with the same
foreground-colour estimation the flat-fill path uses.

For an actual document this feature is irrelevant: specs require one flat
colour, and that is what the compliance path selects on its own. What the
review evidence shows people reach for a "generated background" for is a
studio-looking backdrop on a photo they want to reuse as a profile or CV
portrait — and that is a compositing problem, not a diffusion problem.

**Not implemented:** genuinely generative backgrounds. A diffusion model
capable of it is hundreds of megabytes and tens of seconds per image in a
browser; doing it server-side would mean uploading the photo, which is
the one thing this product promises not to do. If it is ever added it
should be an explicit, clearly-labelled upload, not a silent one.

## 4. Print sharpening

**Sold elsewhere as:** "HD 画质修复" / "AI quality restore" — in one
competitor's case with the free quota cut from 6 uses a day to 1, which
the research logged as a source of sustained complaint.
**Here:** `frame_retouch::enhance` — threshold-gated unsharp masking
after the final resize.

This is deliberately not a super-resolution model, and the UI says
"sharpen for printing" rather than anything with "AI" in it. **An ID
photo is evidence.** A model that invents plausible detail — a crisper
iris, a cleaner jawline, skin that was never in the source — is inventing
detail about a person's face on a document that has to match them at a
counter. Deterministic resampling plus local-contrast recovery puts back
what the pixels actually contain and nothing else.

The threshold matters: it gates on luma difference so the pass does not
amplify sensor noise and JPEG blocking in flat areas, which in a
head-and-shoulders photo against a plain background is most of the frame.

## 5. Formal clothes

**Sold elsewhere as:** "AI 换正装", a per-use or subscription feature.
**Here:** `frame_retouch::garment` — a parametric garment positioned from
the face landmarks and the alpha silhouette.

The need is real and specific: someone has to submit a photo in a jacket
for a visa, a job application or an exam registration, and does not own
one. This is the one feature on the list a browser genuinely cannot solve
with a model, so it is solved the way the open-source prior art in this
category solves it — with a template whose geometry is derived from the
detected face.

Geometry is in units of face width, anchored at the chin, and de-rotated
by the head's roll so a slightly tilted subject gets a slightly tilted
collar rather than a level one that reads as pasted on.

Telling hair from a shirt needs a segmentation model this build does not
have, so the keep-out is geometric: solid subject alpha sitting *above*
where a shoulder could be is hair, and the garment does not draw there.
Below that band it always draws, because that is the torso and replacing
what the person is wearing is the point. Hair much longer than a face
width past the chin gets overpainted at the ends.

**The UI says it is a template**, in both languages, and suggests looking
at the result before relying on it. A generative version would need a
garment-aware diffusion model and a server; the honest version of that
feature is not one we can ship in a page that promises the photo never
leaves the device.

---

## What is deliberately absent

From the research's own "do not build" column (§五.2), unchanged:

- **Stickers, filters, social sharing.** This is a task people want to
  finish and leave. Adding somewhere else to go is a cost, not a feature.
- **AI face effects** (ageing, face swap). Directly at odds with a
  document photo's whole purpose.
- **An all-in-one AI tool hub.** The opposite of a single-purpose tool,
  and the research found the competitor built on that pitch losing
  traffic.
- **4K export.** No document needs it, and no review evidence asks for it.

And one more, added here rather than inherited: **retouching is off by
default and capped.** The research found forced beautification is itself
one of this category's top complaints ("强制开启美颜、无法完全关闭"), and
a photo that stops looking like its holder fails at the counter. The
slider exists, starts at zero, tops out well below "smooth plastic", and
is masked to the face so hair, collar and background keep real texture.
