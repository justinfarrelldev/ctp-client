# Reference parity: authoritative facts the client must not invent

Client-only findings while continuing `full-screen-reference-parity.md`. Server inspected read-only at `15ada0164180b8e6b8c280029ba55ef5cac88168`. These are specific display gaps, not permission to stop unblocked client work, change the server, or check off complete screenshots.

## Military advisor recommendation

Original `ctp2_source/ctp2_code/ai/CityManagement/governor.cpp::GetUnitsAdvice` (2476 onward) uses AI build-list deficits and the player's global military rank, and `GetTacticalAdvice` (3242 onward) uses private scheduler goals and computed desired readiness. `unitmanager.cpp::UpdateAdviceText` displays the resulting localized advice with an evaluated SLIC context. The v2 projection does not expose these recommendation/context results; `foreign_statistics_history` is explicitly false. The client can render the exact installed `UnitManagerAdviceWindow`, own-unit counts and local advice visibility, but must not manufacture the reference's “strong military” statement or undisclosed cities/threats.

Needed server-side contract (not implemented here): a revision-bound, viewer-private advisor result with a verified advice ID and already authorized substitutions/targets, or explicit unavailable status. Do not publish foreign rankings/planner internals merely to reproduce text.

## Applied military production support versus gold upkeep

Original `UnitManager::UpdateAdvice`/`UpkeepButton` uses `Readiness::GetCost()` divided by the player's production, not gold maintenance. Server `engine/lifecycle.go::unitUpkeepForNation` sums `Unit.Upkeep` as monetary upkeep. Applied military production support instead uses `engine/cities.go::readinessSupportCandidates`/`readinessSupportCostForUnit`, with shield hunger, support exemptions/professionals, special forces, readiness/government coefficients, empire reductions and fractional rounding. The safe projection provides own base flags/costs but no final support budget/gross-production pair. City forecast rates alone are not that accounting contract.

The client proves zero support for a complete own-unit set with zero shield hunger or explicit exemptions, and otherwise shows unavailable in the original production-support control. It never labels gold upkeep as production or ports a divergent economy simulator. V2's known Go `omitempty` integer zero is distinct from legacy absent fields. The original zero-production/zero-support 100% presentation is misleading; the client displays 0%, with regression coverage.

Needed server-side contract: current-revision own effective military production support and its exact production denominator (plus explicit zero/status), separate from unit/building gold upkeep. A percentage/absolute presentation toggle is local; changing readiness remains an authorized command.

## Trade ranking, import bonus, native nation color and cancellation

Read-only v2 source evidence: `visibility/visibility.go::tradeRouteActions` advertises exact source/destination payloads only. `TradeRouteProjection` supplies confirmed good/gross gold/transport cost/pirating army for existing routes, not price/good/transport quotes for each prospective destination. `RulesProjection` and `rules.RuntimeRules` do not contain resource/goods Food/Production/Gold definitions. Diplomacy supplies relation IDs/treaties, not original player-color indicators. The gateway manifest has create-trade-route but no registered cancellation operation.

The source/reference three-tab Trade Manager can show authorized candidate pairs with distinct complete-payload identities, explicit local own/friendly/all filters, independent own-export/import rows and received route accounting. It cannot claim top-N profit ordering by calculating a separate economy, show route transport cost as Import's resource Food/Production/Gold bonus, fabricate a nation flag/color from an ID, or enable Break Route through an invented endpoint. The source 9x20 ranked-destination thumb remains visible but disabled with the quote reason. The local source initial count is 1; available unranked authorizations are not claimed to be a top-1 market calculation.

Current public `TradeManager::UpdateAdviceWindow` sums own cities' `GetGoldFromTransitRoutes()` and `GetGoldLostToPiracy()`; these effective nonempty accounting fields are not projected. The client can prove both zero with **no owned cities in a complete v2 snapshot**, and display the original no-opportunity `MAXIMUM_TRADE_EFFICIENCY` branch in that case. With cities, it leaves the unsupported accounting/recommendation unavailable rather than summing gross revenue into transit/net loss.

Needed authoritative contracts: revision-bound per-good market quote/ranking (including legal exact payload, current good, gross price and transport cost), import resource bonus tuple/pinned public definition, own effective transit/piracy totals and public nation color/identity metadata; a registered cancellation command with phase/ownership/revision/idempotency semantics for routes the viewer may cancel. This report is client-only and does not implement or alter the server.

Original source issue encountered: the old best-price insertion loop reads `maxPrice[j-1]`/`maxCity[j-1]` at `j == 0`. No such undefined negative indexing is ported; future projected quote sorting must use checked stable collections instead. Current client does not duplicate the private quote calculation at all.

## Other current explicit capability limits

V2 advertises `policy_optimizer=false`, `city_culture_style=false` and `foreign_statistics_history=false`; unsupported barbarian ruin profiles remain disabled. The client must still render original controls/layouts with honest disabled/unavailable presentation. It cannot certify matching reference values/culture by using installed retail rules, hidden canonical endpoints, owner-derived culture, fabricated zeros or live trial mutations. Audit each relevant scenario separately before final acceptance.
