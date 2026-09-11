# Review, privacy and rating answers

The same facts, in the shape each store asks for them. Everything here is
true of the shipped bytes: the app has no network endpoint of its own, and
the e2e suite (`apps/webapp/e2e/app.spec.js`, "nothing is sent anywhere")
asserts that no request leaves the origin during a full run.

## Review notes (both stores)

> No account is needed for any feature; do not sign in to test. Open the
> app, tap "Choose a photo", pick any portrait, choose a document (e.g.
> United States → Passport). The photo is processed on the device in a
> few seconds and the checklist fills in. "Save" writes the JPEG. The
> home-page footer link "Device test" runs the pipeline on a bundled
> sample and prints timings, if you want a one-tap check.
>
> The optional "Account" button opens our account page in the system
> browser; it is for credits in our other apps and unlocks nothing here.

## App Store — App Privacy

- **Data collection:** *Data Not Collected.* The app makes no network
  requests of its own. Photos are read from the library or camera,
  processed in memory, and written back; nothing is sent anywhere.
- **Tracking:** No.
- **Privacy policy URL:** https://openphotoid.com/privacy.html
- **Privacy manifest:** `PrivacyInfo.xcprivacy` declares no tracking, no
  collected data types, and required-reason APIs `NSUserDefaults`
  (CA92.1) and file timestamps (C617.1).

## App Store — Age rating

Every questionnaire item: *None* (no violence, no mature themes, no
gambling, no unrestricted web access — the WebView loads only bundled
files; the account link opens Safari). Result: **4+**.

## App Store — Export compliance

`ITSAppUsesNonExemptEncryption = false` in Info.plist: only HTTPS via the
system libraries when the account link is opened in Safari, no
proprietary cryptography.

## Google Play — Data safety

- **Does your app collect or share any of the required user data types?**
  No.
- **Is all of the user data collected by your app encrypted in transit?**
  Not applicable (no data collected).
- **Do you provide a way for users to request that their data is
  deleted?** Not applicable; nothing is stored off the device. Closing
  the app leaves nothing behind.
- **Privacy policy:** https://openphotoid.com/privacy.html

## Google Play — Content rating (IARC)

Category: *Utility, productivity, communication, or other*. Every
question: No. Result: **Everyone / 3+ / PEGI 3**.

## Google Play — App access

*All functionality is available without special access.*

## Google Play — Ads, target audience

No ads. Target audience: 18 and over is the simplest truthful answer
(passport photos are an adult errand); the app is not designed for
children and contains nothing that would need the Families policy.

## Permissions the app declares

| Platform | Permission | Why |
|---|---|---|
| iOS | Camera | "Take a photo" |
| iOS | Photo library (read) | "Choose a photo" |
| iOS | Photo library (add) | "Save" to the library |
| Android | `CAMERA` | "Take a photo" |
| Android | photo picker | "Choose a photo" (no storage permission on Android 13+) |

No location, contacts, microphone, or background activity.
