# Automation review: teaching and delegation boundaries

PR #376 Round 1 found that recording any newly used discovery feature suppressed all routine advisor guidance for the turn. Optional lessons now pause only other optional lessons; early build, move, research and end-turn instruction remains reachable. Opening Cities, toggling Yields and sending one's own team ping persist feature use instead of relying on transient UI state.

Economic delegation excludes World Wonders: their permanent tile placement is a strategic manual choice. All legacy focus inputs use the economic policy, so the obsolete Hard-derived military/stream chooser and its retired behavior tests were removed. Hard AI production is unchanged. Regression tests cover both boundaries.

The broader feature-discovery inventory and gameplay reviewer audit were explicitly requested by the maintainer in the conversation, alongside city automation. An issue's shorter summary must not silently narrow that authorized scope during review.
