# Historical faction rename map

Decision: 2026-10-10. The eight playable factions represent real peoples and historical institutions. Internal `Faction`, `UnitKind`, `BuildingKind`, goal action and save IDs remain stable for compatibility. Names below are display text. Figures and rules are unchanged; the general research tree, units, eras and buildings await the separate era overhaul.

| Internal ID | Display name | Adjective | Motto | Identity | Faction-specific pieces | Goal title and flavour |
| --- | --- | --- | --- | --- | --- | --- |
| Switzerland | Switzerland | Swiss | Guard the passes. Keep the accounts. | Alpine stewards of coin and crossings | Customs Bastion → Alpine Redoubt (Walls) | The Reserve: hold the required Gold in the treasury; it scores only while the money remains. |
| Mongols | Mongols | Mongol | Across the steppe, news travels. | Riders of the steppe and the yam | Horseman → Keshig (the imperial guard of mounted archers; Civ: Keshik), replacing the Mongol Outrider Scout on 2026-10-17; Courier Hall → Yam Relay Station | The Yam Network: build and hold the relay station; its road network helps everyone and opens the holder's borders at Peace. |
| Sparta | Sparta | Spartan | Train in the agoge. Hold the line. | Citizens trained for war | Oathbearer → Spartan Hoplite (Soldier); War Shrine → Agoge; Iron Guard promotion → Spartan Veteran | Honour the Veterans: retire three promoted soldiers at the Agoge. This is game fiction, not an attested Spartan rite. |
| Germany | Germany | German | Coal, steel, and skilled hands. | Workshops and industrial cities | Foundry → Ruhr Foundry (Factory); Engine → Machine Works | Every City a Workshop: build a Machine Works in every city, with at least four cities. |
| Japan | Japan | Japanese | Rice sustains the castle town. | A network of towns and rice stores | Brood Hall → Rice Storehouse; Hive Spire → Castle Town (where the name appears in art/UI) | Great Migration: send 30 population from the largest cities, leaving at least one in each, for 2 VP. **Lead review:** the mechanic removes population without placing settlers elsewhere; this is a metaphor, not a literal migration. |
| Byzantium | Byzantium | Byzantine | Hold Constantinople. Preserve the record. | Roman continuity at Constantinople | Salvage Works → Imperial Scriptorium (Laboratory); Legionary → Tagmatic Soldier (Trooper); Heir → Porphyrogennetos (civilian) | The Purple-Born: train and guard the Porphyrogennetos; whoever captures the heir holds the points. |
| Celts | Celts | Celtic | From hillfort to forest, know the land. | Communities rooted in their terrain | Thorn Hunter → Celtic Ambusher (Ranger); Call the Wild action → Raise the Warbands | Raise the Warbands: each city hosts a hostile camp; rivals can clear the camps and remove the live points. |
| Athens | Athens | Athenian | Debate in the agora. Study at the Lyceum. | Debate, learning and civic exchange | Listening House → Lyceum (Laboratory) | Share the Learning: teach every rival three technologies of its own choice. |

## Lore

**Switzerland.** Alpine passes tied Swiss communities to trade across Europe, while local defense made the mountains a formidable refuge. Swiss banking and public finance grew over centuries; the game's treasury interest and city-capture bounty condense that history into a competitive rule. The Alpine Redoubt invokes the country's mountain defenses without claiming every Swiss city had such a fort.

**Mongols.** Mongol armies used mobility and intelligence to connect widely separated territories. The imperial yam used relay stations and remounts to carry messages and travelers across those distances. In play the Yam Relay Station links every city, so even rivals can use the roads.

**Sparta.** Sparta trained its citizen soldiers through the agoge and relied on disciplined hoplite warfare. Military life rested on the labor of helots, a coercive system that should not be romanticized. The game's desertion during peace and healing through combat are abstractions of its war-focused identity, not historical claims.

**Germany.** German-speaking lands included many states before unification, and later became major centers of engineering, coal, steel and manufacturing. The Ruhr Foundry and Machine Works draw especially on nineteenth-century industry. The faction's slow units are a gameplay cost attributed to heavy equipment; real German armies were not inherently slow.

**Japan.** Japanese polities linked provinces through roads, rice taxation and castle towns; agricultural stores and dense settlements supported political centers. The Rice Storehouse and Castle Town use that human geography in place of a hive. The population sacrifice is a game rule, not a claim that a specific Japanese state sent thirty people abroad.

**Byzantium.** The eastern Roman Empire preserved Roman state traditions around Constantinople for more than a thousand years, adapting Greek learning, law and military organization. Imperial scribes copied and studied older texts; the tagmata were professional military formations. *Porphyrogennetos* means born in the purple, a title associated with children born to a reigning emperor.

**Celts.** Celtic-speaking peoples lived across Europe in many distinct communities and eras; this faction does not imply they were one empire. Its hunters and defended territory draw on local knowledge of landscape rather than magical control of nature. The hostile camps summoned by its goal remain a deliberate game abstraction.

**Athens.** Classical Athens was a center of civic debate, philosophy and learning, alongside a maritime empire and a democracy that excluded women, enslaved people and noncitizens. The Lyceum recalls a later Athenian school. Sharing discoveries with every rival gives this faction's knowledge a difficult strategic price.

## Setting and language

- Title screen: the Legatum wordmark alone (the tagline and “A contest through history” line were removed, user 2026-11-22).
- Guide opening: eight historical peoples meet on a shared game map of ruins, cities and contested roads. The map is a historical thought experiment, not one place or date where all eight met.
- The Relay: remove the fallen interstellar communications network from setting prose. The Mongol **yam** is a faction-specific relay system. Generic `Relay Uplink`, its technology and generic building remain for the era overhaul, with temporary incongruity.
- “The Race for the Legacy” → **The Contest for History**. Victory points continue to work the same way.
- No claim that the names span a single date: Sparta and Athens are classical, Byzantium medieval, and the German industrial references are modern.
- Procedural city-name lists now use historical places associated with each faction. City names are stored in game state, so `tools/sim/repin.sh` changed recorded hashes and runner pins even though no mechanics changed; the generated tapes and fixed-order fixture hashes were updated with that script. A comparison with the prior seven affected fixtures found identical setups, turn counts and recorded orders.

## Mechanics that need lead review

- **Japan:** “Great Migration” currently destroys population and releases territory; no settlers appear. Consider a title such as “The Great Departure” if the metaphor overpromises. The loss of population is unchanged.
- **Sparta:** peace desertion, battle-only healing and retirement at an Agoge are gameplay inventions. The agoge historically trained youths, rather than serving as a veterans' retirement hall.
- **Mongols:** the no-Walls and weak-siege rules landed on master during this work. They express an open, mobile game faction, though Mongol forces historically conducted major sieges. The Yam Network giving every rival passage across the holder's territory is a cost mechanic, not historical policy.
- **Celts:** automatic razing and hostile camps are abstractions and risk flattening several peoples into a stereotype; wording should describe a game faction without treating all Celts as one polity.
- **Byzantium:** a captured porphyrogennetos transferring victory points is a game abstraction of dynastic legitimacy. The no-city-Science rule and Science from occupying rivals' land fit the existing mechanic but understate Byzantine scholarship in its own cities. A tagma was a professional military formation; “Tagmatic Soldier” uses the formation as an individual unit label.
- **Germany:** the Ruhr industrial names sit centuries later than the classical factions; the era overhaul should decide whether factions share one chronological setting.
- **Athens:** the Lyceum was founded by Aristotle in Athens; the game's Laboratory replacement and instant ruins Science are abstractions.

## Art handoff

The historical portrait branch landed on master during this work. All eight `people-<faction>-360.jpg` images are now served by `leaderPortraitUrl` throughout the app, replacing the old speculative character portraits. Text labels and alt text use the historical display names. **TODO:** review the heraldry raster art, title key art (still a fallen relay spire), and Japanese map ornament (still a crystal spire) in a historical art pass.

The five shipped scenarios have historical faction names in their briefings and advice, but their encoded start states retain place names from recorded older games. Older user saves likewise preserve names already assigned to cities. Regenerating scenario states would change their tactical positions and scores; update their city names with the scenario fixture workflow during a dedicated scenario pass.

## Historical reference points

- [The Met: Mongol relay stations](https://www.metmuseum.org/fr/essays/the-legacy-of-genghis-khan)
- [Cambridge: The Mongol Yām and its legacy](https://www.cambridge.org/core/books/abs/postal-systems-in-the-premodern-islamic-world/mongol-yam-and-its-legacy/090AB53E36D25FBB7089C4ACA28F43A1)
- [The Met: Athenian democracy and its male citizen body](https://resources.metmuseum.org/resources/metpublications/pdf/Greek_Art_From_Prehistoric_to_Classical.pdf)
- [The Met: Byzantium, ca. 330–1453](https://www.metmuseum.org/de/essays/byzantium-ca-330-1453)
- [The Met: Constantine VII Porphyrogennetos in the ruler list](https://www.metmuseum.org/de/toah/hd/byru/hd_byru.htm)
- [Edo-Tokyo Museum: the city and its road stations](https://www.edo-tokyo-museum.or.jp/en/p-exhibition/)
