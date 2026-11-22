# Painted map improvements

These are transparent, hand-painted land scenes made for the normal map zoom. They use the subdued earth pigments, upper-left daylight and small isometric viewpoint of the shipped city miniatures. The worked ground tapers into the terrain; none has a plinth. The image sources are in `sources/`; the 128 px and 64 px map assets are in `app/src/assets/improvements/`.

The review images below are tight crops of Chromium captures of an actual WASM game map on the cloud box, using one improved resource tile on its usual terrain beside a city. The wider original captures are the same filenames without `-detail`. Each pair uses the same tile at normal (1.25) and far (0.46) zoom. At far zoom the existing solid ring on the resource marker is the improvement badge. The four upgraded forms use the game's `CropRotation` or `IronWorking` upgrade state.

| Improvement and terrain | Normal zoom | Far zoom |
| --- | --- | --- |
| Farm · Plains | ![Farm beside city, normal zoom](farm-normal-detail.png) | Farm beside city, far zoom (archived outside the repository) |
| Irrigated Farm · Plains | ![Irrigated Farm beside city, normal zoom](irrigated-farm-normal-detail.png) | Irrigated Farm beside city, far zoom (archived outside the repository) |
| Pasture · Grassland | ![Pasture beside city, normal zoom](pasture-normal-detail.png) | Pasture beside city, far zoom (archived outside the repository) |
| Ranch · Grassland | ![Ranch beside city, normal zoom](ranch-normal-detail.png) | Ranch beside city, far zoom (archived outside the repository) |
| Quarry · Hills | ![Quarry beside city, normal zoom](quarry-normal-detail.png) | Quarry beside city, far zoom (archived outside the repository) |
| Deep Quarry · Hills | ![Deep Quarry beside city, normal zoom](deep-quarry-normal-detail.png) | Deep Quarry beside city, far zoom (archived outside the repository) |
| Mine · Hills | ![Mine beside city, normal zoom](mine-normal-detail.png) | Mine beside city, far zoom (archived outside the repository) |
| Deep Mine · Hills | ![Deep Mine beside city, normal zoom](deep-mine-normal-detail.png) | Deep Mine beside city, far zoom (archived outside the repository) |
| Saltpetre Works · Plains | ![Saltpetre Works beside city, normal zoom](saltpetre-works-normal-detail.png) | Saltpetre Works beside city, far zoom (archived outside the repository) |
| Oil Well · Plains | ![Oil Well beside city, normal zoom](oil-well-normal-detail.png) | Oil Well beside city, far zoom (archived outside the repository) |
| Quartz Mine · Hills | ![Quartz Mine beside city, normal zoom](quartz-mine-normal-detail.png) | Quartz Mine beside city, far zoom (archived outside the repository) |

The code still calls the Quartz Mine `LithiumMine`, matching the current rules identifier. This art change does not alter resource or upgrade rules.
