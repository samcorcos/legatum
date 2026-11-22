# Presentation variants

The app's two looks remain Atlas (paper, Cormorant and Libre Franklin) and Court
(lapis, Bodoni Moda and Josefin Sans). `app/src/brand/tokens.css` is the color and
type source for the app and supporting tools.

## Celebration layer

Celebrations intentionally retain their gold gradients, glows, light sweeps and
particles in both looks. They are a temporary ceremony over the interface, as
requested in `celebration-slots.md`, rather than a new treatment for ordinary
panels or buttons. Keep their current tiering, dismissal controls, readable text
surfaces and reduced-motion behavior. The no-glow/no-gradient chrome rule does
not apply to this layer.

## Editorial film and report

The evolution film and its report use an explicit **editorial** variant: Court
colors with Atlas typography (Cormorant display, Libre Franklin body and data).
This variant is for narrated films and their companion reports, not a third
selectable app look. The title and end cards use the canonical SVG wordmark.
Charts keep categorical strategy/player colors; their surfaces, labels and
semantic colors use the shared palette. Film animation and atmospheric lighting
remain part of the presentation; the report uses flat, square chrome.

`tools/design/brand.mjs` reads the app tokens and wordmark and embeds the bundled
Fontsource faces. Reports remain self-contained and readable offline. The film
renderer receives the same palette and wordmark in its payload; these assets
also participate in its render-cache hash. The build dashboard uses Court,
including Court's own typography.

## Type and actions

Court uses Bodoni only at 17 px or above, including italic notes. Body prose is
at least 14 px; small controls and notes use Josefin Sans. Compact tracked labels
may use the existing label scale. Guide prose meets the body minimum in both
looks. These limits also apply to lobby subtitles and research navigation.

The title menu has one primary action: Resume for an open game, otherwise
Continue when an autosave exists, otherwise Single Player. In the multiplayer
menu, Resume takes precedence over Join. Contextual unit actions and the turn
button share the primary fill, inner rule and Court notches. Keyboard focus
removes clipping so the focus outline remains visible. Pointer checks must use
the button center or a point at least 9 px inside its notched corners.

Milestone symbols describe an achievement, not its requirements: Developed
Empire uses a civic emblem and Great Work uses the victory wreath, independent
of city count or the chosen Great Work route. The application icon is the Atlas
Seal in both looks; see `tools/design/directions.md` for its canonical source.
