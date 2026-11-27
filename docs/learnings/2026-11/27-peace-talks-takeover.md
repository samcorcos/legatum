## 2026-11-27: Peace talks and declarations

- Core refuses `ProposeTreaty` while an offer is already open between the pair (`diplomacy::check_order`). Its one-diplomatic-order-per-other-team rule also prevents a same-turn counter-offer alongside a reply. The peace-talks plate therefore has no counter-offer button.
- An incoming Peace offer is already a pending decision in `turnFlow`. The plate uses that same list, so an answered or overtaken offer no longer opens; Decide later leaves the Inbox card available.
- The public view supplies the war's effective turn, known city founders and owners, and each team's army points. These suffice for the plate's facts without consulting private state or calculating a new rule.
