# Max-rank promotion marks

The later [US enlisted ladder](us-enlisted/README.md) replaces this max-rank-only design. The images below remain its historical before/after record.

At normal zoom, the bronze Λ shield replaces Sparta's six-chevron ribbon at level 6. A gilt laurel replaces the five-chevron ribbon for every other faction at level 5. Lower ranks keep their chevrons. At far zoom, the shield or laurel remains a screen-sized mark on the class roundel. The unit panel, tile tooltip and combat forecast use the same rank mark.

`before-*` and `after-*` are full Chromium screenshots from `app/e2e/max-rank-insignia.spec.ts`. Each set includes a Spartan level-6 Field Cannon, a Japanese level-5 Field Cannon, and a Japanese level-3 control unit at **1000×600** and **1440×900**. Each case has `normal`, `far`, and `panel` views. The test loads a real WASM view and checks each unit's promotion level before capture. The before set was captured from the merged `claude/land-6` insignia code; the after set uses the replacement implementation.

For example, compare [Sparta before at normal zoom](before-sparta-level-6-1000x600-normal.png) with [Sparta after](after-sparta-level-6-1000x600-normal.png), or [Japan before in the panel](before-japan-level-5-1440x900-panel.png) with [Japan after](after-japan-level-5-1440x900-panel.png). The level-3 shots check that the ordinary chevrons remain.
