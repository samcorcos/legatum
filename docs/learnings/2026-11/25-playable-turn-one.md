# Turn 1 can render before it is playable

The full-screen people's introduction (`PeopleIntro`) appears over `GameScreen` on a fresh game.
The turn number and the first map draw can both be present while it still intercepts input. Measure
playability after Begin and check that the turn control actually receives input. The leaving loading
plate is different: it fades for 450 ms with pointer events disabled, so waiting for its removal can
count a delay the player does not have to wait through (`LoadingScreen`).
