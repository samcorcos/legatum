# Any Market-family building counts as an Indian Revenue Office (2026-11-27)

`data::has_building(buildings, BuildingKind::Market)` is true for a Bank or Stock Exchange too (`building_satisfies` follows the family). India's Office rules (no city upkeep, full remittance) and the new remittance forecast all key off that check, so a city that skipped straight to a later Market-family building still counts as having an Office. Tests that want a non-capital Indian city with Gold to keep, but no Office, need a Gold source outside the family (the Eiffel Tower works).
