# Reference parity: authoritative facts the client must not invent

Client-only findings while continuing `full-screen-reference-parity.md`. Server inspected read-only at `15ada0164180b8e6b8c280029ba55ef5cac88168`. These are specific display gaps, not permission to stop unblocked client work, change the server, or check off complete screenshots.

## Military advisor recommendation

Original `ctp2_source/ctp2_code/ai/CityManagement/governor.cpp::GetUnitsAdvice` (2476 onward) uses AI build-list deficits and the player's global military rank, and `GetTacticalAdvice` (3242 onward) uses private scheduler goals and computed desired readiness. `unitmanager.cpp::UpdateAdviceText` displays the resulting localized advice with an evaluated SLIC context. The v2 projection does not expose these recommendation/context results; `foreign_statistics_history` is explicitly false. The client can render the exact installed `UnitManagerAdviceWindow`, own-unit counts and local advice visibility, but must not manufacture the reference's “strong military” statement or undisclosed cities/threats.

Needed server-side contract (not implemented here): a revision-bound, viewer-private advisor result with a verified advice ID and already authorized substitutions/targets, or explicit unavailable status. Do not publish foreign rankings/planner internals merely to reproduce text.

## Applied military production support versus gold upkeep

Original `UnitManager::UpdateAdvice`/`UpkeepButton` uses `Readiness::GetCost()` divided by the player's production, not gold maintenance. Server `engine/lifecycle.go::unitUpkeepForNation` sums `Unit.Upkeep` as monetary upkeep. Applied military production support instead uses `engine/cities.go::readinessSupportCandidates`/`readinessSupportCostForUnit`, with shield hunger, support exemptions/professionals, special forces, readiness/government coefficients, empire reductions and fractional rounding. The safe projection provides own base flags/costs but no final support budget/gross-production pair. City forecast rates alone are not that accounting contract.

The client proves zero support for a complete own-unit set with zero shield hunger or explicit exemptions, and otherwise shows unavailable in the original production-support control. It never labels gold upkeep as production or ports a divergent economy simulator. V2's known Go `omitempty` integer zero is distinct from legacy absent fields. The original zero-production/zero-support 100% presentation is misleading; the client displays 0%, with regression coverage.

Needed server-side contract: current-revision own effective military production support and its exact production denominator (plus explicit zero/status), separate from unit/building gold upkeep. A percentage/absolute presentation toggle is local; changing readiness remains an authorized command.

## Other current explicit capability limits

V2 advertises `policy_optimizer=false`, `city_culture_style=false` and `foreign_statistics_history=false`; unsupported barbarian ruin profiles remain disabled. The client must still render original controls/layouts with honest disabled/unavailable presentation. It cannot certify matching reference values/culture by using installed retail rules, hidden canonical endpoints, owner-derived culture, fabricated zeros or live trial mutations. Audit each relevant scenario separately before final acceptance.
