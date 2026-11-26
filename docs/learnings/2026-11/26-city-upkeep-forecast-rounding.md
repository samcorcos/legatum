# 2026-11-26: Forecast city upkeep from the two totals

- **Cause:** The city-upkeep experiment scales the empire's total bill and rounds it down. Scaling the unmodified cost of the next city separately can quote the wrong amount.
- **Evidence:** With seven cities and `city_upkeep_percent = 50`, the total rises from 3 Gold (seven cities) to 8 Gold (eight cities), so the added cost is 5 Gold, not half of the default 9 Gold rounded down to 4.
- **Change:** `economy::added_city_upkeep` subtracts `RuleExperiments::city_upkeep(n)` from `city_upkeep(n + 1)`. The view and all three UI forecasts share that result.
