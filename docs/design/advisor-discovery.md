# Advisor feature discovery

The advisor teaches useful optional controls as well as mandatory decisions. A feature is discoverable only when ordinary play can reach a relevant trigger and the instruction points at a control the player can use. Text in How to Play alone is reference material, not a trigger.

`app/src/ui/advisorModel.ts::advisorSuggestion` handles mandatory decisions and plan advice. `advisorTips.ts::strategyTip` handles visible strategic situations with cooldowns. `advisorDiscovery.ts::discoverySuggestion` handles one-time optional lessons. The advisor remains opt-in after being disabled, never issues an order, and hides while modal decisions or celebrations have control.

## Natural triggers

| Feature | Trigger and target | Coverage |
| --- | --- | --- |
| Move, Found City, production, research, ranged attacks | Pending decision and Hard's fogged plan; selected unit/city, destination, Build or Research control | `advisor.test.ts`, `AdvisorCard.test.tsx` |
| Fortify/Wait, Skip, Auto-explore, End turn | Relevant plan action; routine steps stop after practice on three turns | `advisorModel.ts::practiceKind`, `advisor.test.ts` |
| Auto-explore discovery | From turn 5, an unassigned healthy Scout with unexplored land and no visible nearby threat; Scout then Auto-explore | `advisorDiscovery.test.ts`; independent of Hard choosing the order |
| City automation | At least three cities from turn 80, or era 4 onward; a population-4+ safe city without automation, resistance or a manual build order; city then Automate | `advisorDiscovery.test.ts`; opening remains manual |
| Resource tile improvements | Open a peaceful idle city that can build an improvement; ring its Build control | `advisorDiscovery.ts` |
| Production queue | Open a peaceful city already producing, with no follow-up queue and another available build; ring + Queue | `advisorDiscovery.ts`, `Advisor.tsx::targetElement` |
| Heal | Select a damaged unit that can heal, outside visible nearby danger; ring Heal | `advisorDiscovery.ts` |
| Auto-fire | Select a ranged unit in your city which is not already on Auto-fire; ring Auto-fire from city | `advisorDiscovery.ts` |
| Siege equipment | Select a unit with an available, affordable Siege offer and spare required supply; ring Siege in Improve | `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx` |
| Project conversion | Open an idle city with no available economic build and a Gold project; ring Run Gold project, explain it continues until replaced | `advisorDiscovery.test.ts`; actual per-turn yield remains on the project control |
| Map Yields | Open a safe city from turn 12; ring Lens then the revealed Yields toggle | `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx` |
| Batch city management | Own at least three cities from turn 20 with no selection; ring Cities, explain comparison and bulk automation | `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx` |
| Unit modernization | Select a unit with an affordable, supplied upgrade in owned territory; ring Upgrade and show its actual reserved-budget-aware price | `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx`; reuses `UnitImprovements.tsx::upgradeCheck` |
| Aircraft operations | Select a new own aircraft with a legal Recon order; ring Recon and explain the one-order choice among scouting, striking and rebasing | `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx`; `AirWing.tsx` lists targets/forecasts and hangar destinations |
| Team map pings | From turn 3, a live host supporting pings and another human teammate; ring Ping | `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx`; uses `store.ts::canPing`, not an assumed multiplayer capability |
| Map lenses and settlement guidance | Selecting a founder automatically opens the Settle lens, with darker-green site guidance and a ranked list; the Lens control retains Default and the Yields overlay | `map/useMapLens.ts`, `map/LensSwitcher.tsx`, `map/LensSwitcher.test.ts` |
| Diplomacy and Alliances | First contact opens a plate saying the current relationship and offering Diplomacy; after meeting a living rival with nothing selected and Diplomacy not yet opened, the advisor teaches opening Diplomacy by clicking a rival in the right sidebar (`diplomacy`, ring on `rivalRow`); incoming offers are mandatory turn decisions | `advisorDiscovery.ts`, `advisorDiscovery.test.ts`, `advisorDiscoveryRing.test.tsx`, `celebrate/MeetingTakeover.tsx`, `turnFlow.ts`; garrison and strategic tips point back to Diplomacy when relevant |
| Faction goals and one-time actions | People's introduction explains the goal; ready goal actions join pending decisions before routine city/unit work; goal button/dialog names its effect and cost | `celebrate/PeopleIntro.tsx`, `ui/GoalActions.tsx`, `turnFlow.ts::pendingDecisions` |
| Research queue | Open Research with one technology queued and another available, or finish the queue; ring the next + Queue | `advisor.test.ts`, `AdvisorCard.test.tsx` |
| Turn report, victory race and scoring | After the first turn, then turn 3; explain the four VP sources and hold | `advisorModel.ts::howToWin`, `advisor.test.ts` |
| Occupation locks and freeing garrisons | First locked garrison; later two locked units or one observed for ten turns; Diplomacy | `advisorTips.test.ts`, `advisor.test.ts` |
| Army strength, resources, spending Gold | Nearby stronger rival, a missing revealed strategic resource, or spare Gold; applicable production/tile/Diplomacy action | `advisorTips.test.ts` |
| Offers, free technologies, siege alarms and other required responses | Pending decision takes priority over discovery; turn action names the decision | `advisor.test.ts` |

One-time lessons use persisted `seen` IDs. Learning or acknowledging one pauses other optional discovery lessons for the rest of that turn. Core build, move, research and end-turn guidance remains available, even when a player discovers a feature without seeing its lesson. Automated cities do not get manual build, queue or map-yield discovery prompts. Opening Cities, opening Diplomacy, enabling Yields and sending a team ping persist their seen IDs, including when the advisor is disabled. Using the feature records it as known, so turning automation off does not cause another lesson. Dismissal and already queued manual orders are respected. Discovery does not interrupt Research, placement modes, or mandatory decisions. A safety check uses only visible hostile military units, not hidden information.

## Audit boundary and follow-up checks

The table records verified discovery routes, including introductions outside the advisor where the game already presents the mechanic at the relevant moment. Individual faction abilities, diplomacy terms and air-operation variants still need their own changed-feature check: the existence of a general panel is not evidence that a new mechanic is taught. The common optional controls listed here now have independent discovery triggers. A future feature still needs an explicit row and reachable trigger; this table is not a blanket exemption for features inside an existing panel.

For every gameplay PR, the design and advisor reviewers must inventory affected mechanics, identify a reachable teaching situation and control, and name the evidence. Missing optional-feature coverage is a review finding just like a new mandatory decision with no instruction. Add a direct trigger test, suppression tests for unsafe/inapplicable states, and a mounted target test when a new ring selector is introduced. Do not require a player to happen to follow a rare AI plan before learning a broadly useful feature.
