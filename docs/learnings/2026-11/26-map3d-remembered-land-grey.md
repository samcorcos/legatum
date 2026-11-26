# 3D map remembered land read grey, and remembered sea went dark

Remembered tiles on the 3D map first looked grey instead of the 2D map's sepia, and remembered sea looked like a dark slab. Three causes, all in how colour reaches the screen:

- **Colour space lost the warmth.** The wash maps luminance onto a ramp from umber to the 2D paper colour. With a straight umber-to-paper line, converting to sRGB at output removed the sepia hue and left grey. A warm tan midtone in the ramp, with stops in linear space, keeps it (`map3d/memoryWash.ts::memoryRamp`).
- **Haze started at the camera and was cool.** Scene fog began at camera distance with a blue-grey colour, so at strategic zoom it washed all remembered land toward blue-grey. The haze now starts at `max(6, dist * 1.1)` and turns to a warm paper colour from working zoom out (`map3d/post.ts`, `update`).
- **Water was washed like land.** Sea is dark before the wash, so the land ramp kept it dark. Water has its own lifted ramp (`memoryWashWater`).

Judge a remembered-land change at strategic zoom beside the 2D map, not at close zoom, where the cool haze hides the problem. Any new 3D layer that draws known land must apply the shared wash.
