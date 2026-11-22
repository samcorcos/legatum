# Mobile (in the browser)

Goal: play Legatum on a phone or tablet in its browser, solo against AIs or in an online game, with the same engine
and rules as everywhere else. There is no native app: the desktop and iOS apps were dropped on 2026-11-22 and the
web build is the only way to play.

Not a target yet: the README says the game is made for a laptop or desktop with a keyboard and mouse. This plan is
what it takes to change that.

## What carries over unchanged

- The Rust core, compiled to WASM, runs in the phone's browser (Safari's WebKit on iOS, Chrome on Android). This
  covers determinism, saves and AI.
- The React UI, brand and art, and online play through the lobby.

## What has to change

| Area | Desktop assumption | Mobile answer |
| --- | --- | --- |
| Map orders | Right-click to move or attack | Tap a unit, then tap a destination to preview the path and forecast, then tap again to confirm. Long-press opens the context actions. |
| Map camera | Mouse wheel, arrow keys, drag | One-finger pan, pinch zoom, double-tap to zoom |
| Numbers and breakdowns (pillar 5) | Hover tooltips | Tap to open; tapping elsewhere or Close dismisses. Nothing is hover-only. |
| Research tree | Hover to trace a path | Selecting traces the path (already supported) |
| Keyboard | Enter ends the turn, Space cycles units, and other shortcuts | Large on-screen End Turn and Next Unit buttons |
| Layout | 1000×600 minimum, side panels | Tablet landscape keeps side panels. Phone portrait uses bottom sheets (city, unit, report), a compact top bar, and the VP track as a drawer. Touch targets are at least 44 pt, and safe areas are respected. |
| Lifecycle | Always in the foreground | Autosave when the tab is hidden, and reconnect an online game on return |

## Increments (skateboard first)

Target devices (user, 2026-09-27): iPhone first, iPad comes along almost for free, Android is nice to have.

1. **Skateboard: touch-playable on iPhone.** Touch map controls, tap-for-breakdowns and on-screen turn controls; a
   solo game playable end to end in iPhone Safari.
2. **Scooter: a real iPhone layout.** Portrait bottom sheets (city, unit, report), a compact top bar, the VP drawer,
   safe areas, autosave when hidden, and the premium feel layer (motion and sound; haptics only where the browser
   offers vibration, which iOS Safari does not; see the `premium-mobile-feel` skill). iPad landscape is checked here too.
3. **Bicycle: on your phones.** An online game between two phones, reconnect on return, the touch icon and a web app
   manifest so the game can be added to the home screen.

## Verification

- Every increment keeps desktop working: the existing Playwright suites plus mobile-viewport and touch Playwright
  tests (iPhone and iPad device emulation, Chromium and WebKit).
- A full solo game on a real iPhone before increment 1 counts as done.
