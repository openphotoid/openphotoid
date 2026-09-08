//! Optional portrait finishing, all of it local and deterministic.
//!
//! These are the features the competitive research (`docs/research/`,
//! §五 推荐功能清单) found every commercial competitor charges for:
//! retouching, "HD" enhancement, non-flat backgrounds, and formal-wear
//! replacement. Nothing here is behind a paywall, and nothing here calls
//! a server — which also fixes the thing that research found users
//! complain about most (§五.3: opaque paywalls, free quotas quietly
//! tightened).
//!
//! What each module deliberately is *not*:
//!
//! - [`skin`] is not a beauty filter. The same research found forced,
//!   un-disableable beautification is itself a top complaint in this
//!   category, and an ID photo has to stay a likeness of the person or
//!   it fails at the counter. It is edge-preserving smoothing with a
//!   hard ceiling, off unless asked for.
//! - [`enhance`] is not a super-resolution model. It is a proper
//!   resampling + local-contrast pass — honest about being deterministic
//!   signal processing rather than invented detail, which matters when
//!   the output is a legal document photo.
//! - [`garment`] is not generative. It draws a parametric garment
//!   positioned from the face landmarks and the alpha silhouette, which
//!   is the same approach the open-source prior art in this space uses.
//!
//! See `docs/premium-features.md` for what a generative implementation
//! of the last one would take, and why it isn't here.

pub mod backdrop;
pub mod enhance;
pub mod garment;
pub mod skin;
