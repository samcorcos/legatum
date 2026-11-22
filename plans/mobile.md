# Mobile (Tauri v2 iOS/Android)

Goal: play Legatum on a phone or tablet, solo against AIs or joining a LAN game hosted on a computer, with the same engine and rules as desktop.

## What carries over unchanged

- The Rust core, compiled to WASM, runs in the mobile WebView (WKWebView on iOS, Android WebView). This covers determinism, saves and AI.
- The React UI, brand and art.
- LAN play as a guest: the phone connects to `legatum-server` on a computer, exactly like a second browser does.

## What has to change

| Area | Desktop assumption | Mobile answer |
| --- | --- | --- |
| Map orders | Right-click to move or attack | Tap a unit, then tap a destination to preview the path and forecast, then tap again to confirm. Long-press opens the context actions. |
| Map camera | Mouse wheel, arrow keys, drag | One-finger pan, pinch zoom, double-tap to zoom |
| Numbers and breakdowns (pillar 5) | Hover tooltips | Tap to open; tapping elsewhere or Close dismisses. Nothing is hover-only. |
| Research tree | Hover to trace a path | Selecting traces the path (already supported) |
| Keyboard | Enter ends the turn, Space cycles units, and other shortcuts | Large on-screen End Turn and Next Unit buttons |
| Layout | 1000×600 minimum, side panels | Tablet landscape keeps side panels. Phone portrait uses bottom sheets (city, unit, report), a compact top bar, and the VP track as a drawer. Touch targets are at least 44 pt, and safe areas are respected. |
| Files | Native Save/Open dialogs | App-data saves with a Saves list, plus share and export |
| Lifecycle | Always in the foreground | Autosave on background, and restore on resume (reconnect for LAN) |
| LAN join | The page is served by the host | The app bundles the UI, so Join asks for the host address (auto-discovery comes later) |
| Hosting | The computer runs `legatum-server` | Out of scope. Phones join; a computer hosts. |

## Increments (skateboard first)

Target devices (user, 2026-09-27): iPhone and Mac come first. iPad comes along almost for free. Android is nice to have but not a priority.

1. **Skateboard: touch-playable on iPhone.** Touch map controls, tap-for-breakdowns and on-screen turn controls; Tauri iOS scaffolding running on the iPhone simulator; a solo game playable end to end.
2. **Scooter: a real iPhone layout.** Portrait bottom sheets (city, unit, report), a compact top bar, the VP drawer, safe areas, autosave on background, the app-data Saves list, and the premium feel layer (haptics and motion; see the .claude/skills/premium-mobile-feel skill). iPad landscape is checked here too.
3. **Bicycle: on your phones.** A TestFlight build installed on both players' phones, a LAN guest join flow from the app (enter the host address; the computer hosts), reconnect on resume, app icon and launch screen.
4. **Later:** the Android build (needs the Android SDK and NDK) and auto-discovery of LAN hosts.

## Distribution

Expo EAS builds React Native and Expo projects, not Tauri apps, so it isn't a fit here. Use Tauri's own iOS pipeline:
- `tauri ios build --export-method app-store-connect` produces the .ipa;
- upload it to App Store Connect / TestFlight with `xcrun altool --upload-package` (or Transporter), authenticated with an App Store Connect API key.

An Apple Developer account is available. Needed at increment 3: the Team ID and an App Store Connect API key (issuer ID, key ID, .p8). Store them outside the repository (an untracked env file), never in it.

## Verification

- Every increment keeps desktop working: the existing Playwright suites plus new mobile-viewport and touch Playwright tests (iPhone and iPad device emulation).
- Simulator runs are checked visually by the lead (the iOS simulator panel), including a full solo game.

## Open questions for the user

- At increment 3: the Apple Team ID and an App Store Connect API key for TestFlight uploads.
