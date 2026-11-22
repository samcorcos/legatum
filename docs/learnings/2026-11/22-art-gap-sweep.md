# Art gap sweep: most gaps were wiring, not missing paintings (2026-11-22)

A sweep for plain icons where comparable items have painted art found four gaps, and three of them needed no new painting:
the built-wonder tile card, and the Yam Relay Station, Imperial Diwan and Agoge goal moments, all had a painting on disk
that the screen did not use. Only the Random faction sheet lacked a picture. Before commissioning art, look for an existing
painting first (`unlockArt.ts` `slugUrl`, `wonders.ts`, `assets/art/`).

`<Icon name="resource-…">` already renders the painted resource art (`bits.tsx` checks `resourceArtUrl` first), so a
resource shown through `Icon` is not an engraved-icon gap. Moments stay free of Vite imports: give a `Moment` a kind
(`unit`, `building`, `resource`) and let `MomentLane` resolve the picture.
