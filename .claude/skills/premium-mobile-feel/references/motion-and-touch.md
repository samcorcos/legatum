# Motion and touch patterns

## Timing and easing

- Press state: scale 0.97 plus a slight darken on `pointerdown`, restored on `pointerup` or cancel, within 1 frame (CSS `:active` is not reliable on iOS; use pointer events or a `.pressed` class).
- UI transitions: 120 ms (micro), 200 ms (panels, sheets, cards), 320 ms (screen changes). Use spring-like easing `cubic-bezier(0.2, 0.9, 0.25, 1.05)` or a real spring (Motion library if already a dependency; otherwise CSS).
- Map camera moves: ease-out, at most 450 ms, interruptible by any touch.
- Unit movement replay: follow the real path, 90–140 ms per hex, capped at 1.2 s per turn in total (use the existing slide animation in `app/src/map`).
- Stagger lists (turn report lines, lobby seats): 30–40 ms per item, capped at 8 items.
- Numbers that change (gold, science, VP): count up or down over 300–500 ms with tabular numerals, so the eye sees the delta. The breakdown remains available on tap.
- Reduced motion: no translate or scale; crossfade at most 120 ms; the camera jumps. Haptics still fire.

## Touch

- Minimum target 44×44 pt; spacing of 8 pt between adjacent targets.
- `touch-action: manipulation` on controls (removes the double-tap zoom delay); on the map canvas, `touch-action: none` with pointer events handling pan and pinch.
- Tap versus drag threshold: 8 CSS px of movement or 180 ms; long-press at 450 ms with a visible fill ring from 150 ms.
- Two-step commit for consequential map orders: the first tap previews (path, forecast, `feel.preview`), and a second tap or the Confirm button commits (`feel.commit`). Tapping elsewhere cancels quietly.
- Bottom sheets (phone): detents at peek, half and full; drag the handle, snap with a spring, and fire `feel.sheet` on each detent. The background map stays interactive at peek.
- Disable iOS text selection and the callout on game chrome (`user-select: none; -webkit-touch-callout: none`), and keep it on real text (the guide, reports).
- Prevent overscroll bounce on the game screen (`overscroll-behavior: none`) while keeping it on scrolling panels.
- Safe areas: `env(safe-area-inset-*)` padding on bars and sheets; `viewport-fit=cover` in the HTML meta.

## Native-feeling moments

- Launch: the page background is the Atlas parchment color from the first paint, so there is no white flash. The title's key art fades in within 300 ms of first paint.
- Background/foreground: autosave on `visibilitychange` to hidden; on resume, an online client reconnects silently and shows "Reconnected" briefly.
- Loading: never a bare spinner; use the existing LoadingScreen style (compass, progress rule, a lore tip).
- Errors: human wording, the reason and the fix, with a `feel.invalid` haptic, never a raw exception.
