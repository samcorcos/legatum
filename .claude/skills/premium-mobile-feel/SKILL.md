---
name: premium-mobile-feel
description: This skill should be used when building or reviewing any Legatum mobile or touch feature (the game in a phone or tablet browser, touch controls, tablet or phone layouts) or when adding feedback polish anywhere in the app — haptics, motion, press states, sound, loading and transition moments. It defines the premium-feel standard, the game-event-to-haptic map, and how to implement and verify it.
---

# Premium mobile feel

The user's bar: Legatum should feel like a premium native application, "refined and grand, like a real 4X that seeks to be the successor to Civilization". Premium is mostly invisible: every touch answers instantly, every consequence has a proportionate physical and visual echo, and nothing ever janks, flashes or waits without explanation. This skill turns that bar into concrete rules.

## When to apply

- Any change under `app/src` that adds or alters an interaction, or any mobile or touch work (plans/mobile.md increments M1–M4).
- Reviewing an Astra or agent branch that touches UI: check it against the checklist at the end.

## The five rules

1. **Instant acknowledgement.** Visual press feedback within one frame (no 300 ms tap delay: `touch-action: manipulation`), and a haptic on the same frame as the visual. Never wait for the engine to acknowledge a tap; show the pending state and resolve it.
2. **Proportion.** Feedback intensity matches the consequence. Selecting is a whisper (selection tick); committing an order is a firm tap; losing a city is a heavy thud and a warning pattern. Never haptic on scroll or pan, never on every hover-equivalent, never twice for one action.
3. **Meaning over decoration.** Every haptic, sound and animation maps to a game event from `docs/design/haptics.md`. Add a row there before adding a new one. Decoration-only effects are cut (pillar 1).
4. **Truthful motion.** Motion explains cause and effect (a unit slides along its real path; a VP gain flies to the track). Durations 120–320 ms for UI, springs over linear easing, no motion longer than the information needs. Respect `prefers-reduced-motion` and the in-app reduced-motion setting (`app/src/ui/useReducedMotion.ts`): replace motion with a crossfade or instant state, keep haptics.
5. **Native manners.** Respect safe areas, disable rubber-band overscroll on the map, keep 60 fps while panning, keep text selectable only where it is content, launch to an interactive title screen with no white flash, and autosave on backgrounding.

## Implementation

- **Haptics:** the browser's Vibration API (`navigator.vibrate`), which Android browsers offer and iOS Safari does not. There is no native app (dropped 2026-11-22), so iPhone feedback is visual and sound only.
- **Wrap everything behind one module**, `app/src/feel/haptics.ts`, exposing semantic calls such as `feel.select()`, `feel.commit()` and `feel.turnResolved()`, not raw plugin calls. The module must:
  - no-op everywhere but Android browsers with `navigator.vibrate`, so callers never branch;
  - honor a Settings toggle for haptics (default on) plus the OS setting;
  - debounce so one user action produces at most one haptic;
  - never throw into gameplay: all calls are fire-and-forget with caught errors.
- **Emit haptics from the UI layer at the moment of user intent**, or when the turn result is applied (from the event list in the turn report), never from the deterministic core. The core stays pure; haptics are presentation.
- **Sound** (later increment): same semantic API (`feel.*` plays both), short and restrained sounds, off when the device is silent, and a separate volume.
  - **A new sound** gets its `manifest.json` entry and its row in `docs/design/haptics.md`; regenerate `app/public/audio/CREDITS.md` with `node app/scripts/audio-credits.mjs`. A new author also goes, by hand, into the file count and author list in `THIRD_PARTY_NOTICES.md` and the audio row of `app/src/assets/PROVENANCE.md` (art review, 2026-11-25). Cloud sessions cannot reach kenney.nl, Freesound or OpenGameArt; CC0 files mirrored in Unciv are reachable on raw.githubusercontent.com.
- Motion and touch patterns (press states, springs, bottom sheets, gestures, long-press) are in `references/motion-and-touch.md`.

## Verification

- Unit-test the event→feedback mapping as a pure function: given a turn's events for a viewer, produce the expected ordered feedback list. Test also debounce, the toggle, and the desktop no-op.
- Playwright with `hasTouch` on iPad and iPhone viewports: every tappable control responds with its pressed state, nothing is hover-only, and there is no horizontal page scroll.
- Dev builds log `feel.*` calls (`window.__feelLog`); assert them in tests. Real vibration needs an Android device; note that in reports rather than claiming it was felt.
- Frame-time check while panning and zooming the map on a device, with the `?perf=1` overlay.

## Review checklist

- [ ] Pillar 9: the screen states the decision it wants, with one unmistakable primary call to action. Choices look like controls (a border, chevron or button) before hover.

- [ ] Every new interaction has press feedback within one frame, plus a row in `docs/design/haptics.md` if it has a haptic.
- [ ] No haptic on continuous gestures or repeated rapid events; one per action.
- [ ] Consequence-weighted intensity (select < commit < turn events < losses and victories).
- [ ] Reduced motion honored; haptics toggle honored; desktop unaffected.
- [ ] Touch targets are at least 44 pt, with safe areas respected.
- [ ] No hover-only information (pillar 5).
- [ ] The core crates are untouched by feel code.
