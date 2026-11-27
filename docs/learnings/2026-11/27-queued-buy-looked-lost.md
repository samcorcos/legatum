# A queued purchase looked lost because the panel never named it

A player bought what they took to be the Granary in a city that was building it. The replay showed the Gold had bought a Workshop, and the same turn's queue edit had put Walls ahead of the Granary. The engine was right: Gold was spent on the Workshop and it was built. But the only sign of a queued `BuyItem` was a tick on one Buy button far down the production list, so nothing on the panel showed that the wrong item was being bought. The Gold total also drops as soon as the buy is queued, while the item only arrives when the turn resolves, which made it look as if the Gold had been taken for nothing.

Fix: the city panel now names a queued purchase under "Now producing" ("Buying Workshop for 120 Gold. It arrives when the turn ends.") with a Cancel button.

How to check a report like this: pull the game from the games read API, replay its tape with `game_dump` built at the game's `engine_commit`, and read the city's `producing`, banks, buildings and the `ItemBought` events turn by turn.
