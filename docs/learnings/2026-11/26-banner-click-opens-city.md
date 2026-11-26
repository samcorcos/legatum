# A city banner click opens the city in one click, on both maps

- A click on a city with units in it used to cycle: own units first, then the city. A garrisoned city took two
  clicks to open (issue #280). Civ 6 opens the city from its banner, so a banner click now selects the city
  directly, and a click on the hex itself still cycles units and city.
- The 3D map drew banners on its flat marks canvas but kept no hit areas, so a click on a 3D banner fell through
  to the ground pick and hit whatever hex lay under the banner (usually the tile north of the city).
  `drawMarks2d` now returns each banner's rects (plate plus meters) and `Map3DView` checks them before picking.
- Tile pickers (buy a tile, place a wonder, aircraft targets) skip banner hits on both maps, so a lit tile under
  a banner stays clickable.
