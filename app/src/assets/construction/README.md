# Great Work construction stills

The Great Library, Grand Bazaar and Royal Arsenal each have three approved,
transparent construction stills: footings, lower courses, then a roofless
shell. Their finished fourth frames live in
`../../../../docs/design/unlock-art/masters/` as `building-greatlibrary.png`,
`building-grandbazaar.png` and `building-royalarsenal.png`. The `-256`
and `-128` files are Lanczos downscales of each 1254 px stage.

The stages were image edits of their finished masters, with their construction
prompts saved beside each full-size image. The designer approved the Great
Library's stage concepts on 2026-10-24. Its full-resolution stages were then
registered to the finished master by matching ground details or foundation
corners. On 2026-10-25, `tools/art/landmark-atlas-edit.py` applied one restrained
map-palette and ground-edge edit to all three masters and their stages. It reads
the approved source pixels from the recorded base commit, preserving their
construction content and alignment. The derivatives were regenerated from the
edited full-resolution files. They use the same CC BY 4.0 art license as the
rest of the painted set.

The Pyramids, Hagia Sophia, the Eiffel Tower and the ITER Fusion Reactor
(`building-pyramids`, `-hagiasophia`, `-eiffeltower`, `-iterfusionreactor`) have the same
three stages, made on 2026-11-01 with `tools/art/wonder-art.py`: each master was painted
by image generation on a flat magenta field, then each stage was an image edit of that
raw master (prompts beside the files). `wonder-art.py stage` keys the field to alpha and
registers the stage to its master with the master's own fit and ground-fade geometry
(`docs/design/unlock-art/masters/wonder-fit.json`), so the footprint and patch edge never
jump between frames (the lower-half patch overlap is above 0.96 for every stage). Pyramids:
levelled footings, rubble-and-mudbrick ramps up stepped cores, then a nearly full stepped
core with the white casing laid from the bottom and no capstone. Hagia Sophia: footings
with timber hoists, piers and walls at half height, then the great piers and arches done
with the dome as an open timber centring. Eiffel Tower (all three stages repainted 2026-11-22 from the new master, stage 3 keyed with
`wonder-art.py --lattice`): four masonry piers with iron stubs,
four separate trestled legs not yet joined, then the legs joined by the first platform and
the shaft rising to a short unfinished top. ITER: the excavated pit with the isolation pedestals, half-height tokamak walls
with the bare steel assembly hall, then full-height concrete with the hall half clad.

The Yam Relay Station (`building-yamrelaystation`) also occupies a chosen tile under
the wonder-tile rule. Its three 1254 px edits show circular footings, exposed
lattice yurt frames, and partially covered roof ribs, with matching 256 and 128
px derivatives. These were added after the original stage set; the designer has
not reviewed their alignment or construction clip yet.

**Alignment rule:** each stage's camera, scale, structure footprint and ground
patch stay registered to its finished master. Review the Great Library stages at
50% opacity over its master in
`../../../../docs/design/construction/great-library-stages-sheet.png`.
Regenerate that sheet with
`python3 docs/design/construction/regenerate-stages-sheet.py` after changing a
Great Library PNG. Structural changes between stages are intentional; movement
of the site or footprint is not.
