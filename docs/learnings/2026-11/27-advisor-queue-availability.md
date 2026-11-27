# Advisor queue targets

`ResearchView.available` includes current-era technologies already in the research queue. An advisor lesson that points to a new `+ Queue` action must choose from `available` after excluding the effective queue, including pending orders. Otherwise it can ring a technology the player cannot add again.

The queue lesson also needs that filtered choice before it appears. A drained queue after the final technology has no useful target.
