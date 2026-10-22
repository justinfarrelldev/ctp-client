# Local game repeated resync/current reads — user log follow-up

Parent: immediate-live-play-recovery.md and oct-22-2026-ui-followups.md; added explicitly at user request. Remains tracked alongside P0 fixes, **without displacing the latest UI-screen/original-input/all33 reference priority**. Do not equate old regression fixes with live resolution.

## User evidence (game55, local, early turns)
- 2026-10-22T19:07:02Z: revision10, turn2,22 tiles,2 units,0 cities.
- 2026-10-22T19:07:52Z: revision14, turn2,22 tiles,2 units,0 cities.
- 2026-10-22T19:07:55Z: revision18, turn3,28 tiles,2 units,0 cities.
- 2026-10-22T19:07:58Z: revision21, turn3,28 tiles,2 units,0 cities.
- 2026-10-22T19:08:01Z: revision23, turn4,32 tiles,2 units,0 cities.
- 2026-10-22T19:08:04Z: revision28, turn5,36 tiles,2 units,0 cities.

All lines are INFO from ctp_client::gameplay::server: Open CTP game #55 resync. First gap50s then approximately3s intervals. These are observations, not latency measurements or proof of one particular loop. User expects healthy local play not to continually enter recovery, especially turn2.

Source fact: server.rs::fetch_resync_projection logs that string for **every finite X-Gameplay-Resync:current full-snapshot read**, including normal SSE-triggered refresh. It does not by itself prove state.network became Resynchronizing, rollback occurred or a command failed. Worker watcher also makes a mandatory notify(None) on disconnect; shared event/read budget is3100ms. Historical-notice-driven recovery was separately removed in3df0793. Verify running build/restart before attributing logs to that fix or claiming it deployed.

- [ ] Establish exact running client build/server rules/local mode and command timeline (read-only/injected; no server implementation changes). Preserve user logs and distinguish local session replication from actual error recovery; don't merely rename/suppress warnings or disable required authoritative recovery.
- [ ] Instrument bounded cause-tagged timings/counters: command queued/start/receipt, SSE event revision/durable sequence, normal close/timeout/malformed/status, optional versus mandatory snapshot request/coalescing, budget wait and UI Ready/Pending/Resynchronizing transitions. Avoid tokens, private map state or unbounded logs.
- [ ] Audit actual server SSE wire envelope/revision hint versus stream parser, reconnect/idle behavior, response timeouts and watcher notify(None). Reproduce with accurate fragmented/idle/normal-close/rate-limited streams; don't rely only on idealized revision hints.
- [ ] Check redundant reads for already-covered own receipts/AI events, stale hints, disconnect storms, retry ambiguity and mandatory-bit retention. A full current read may be normal replication, not a rollback; fresh historical notices must not reopen recovery or acknowledge Pending.
- [ ] Diagnose3s cadence/head-of-line blocking: optional read waits and commands share a worker; test whether a user order waits behind a3100ms event-read admission even when its own HTTP is ungated. Separate that delay from true server phase/AI resolution before assigning blame.
- [ ] Fix unnecessary refresh/recovery and classify normal snapshot versus real recovery honestly, without reducing correctness/reconnect safety or locally mutating game state. Local mode must remain server-authoritative; do not bypass revisions or substitute a client engine.
- [ ] Inert-channel/fake-clock/isolated HTTP regressions for healthy local early turns, multiple successive passes, idle SSE, AI revisions, queued receipt, stale/ambiguous command, rate limit and genuine disconnect/full recovery. No real local-server calls/file writes in tests.
- [ ] Validate live acceptance separately: user sees healthy early local turns without unexplained recovery churn and with measured improved responsiveness; actual failures still recover. Necessary snapshot replication must be visibly/logically distinct from fault resync, not hidden. Original nation-progress strip is a linked diagnostic, never guessed AI sequencing.
- [ ] Update $CTPSERVER/server-client-schisms.md report with proven client/server ownership, exact source evidence/required protocol changes only if needed; server devs own implementation. Strict lint/full tests>=80% LLVM/native build/<800/lint-fix/normal hooks for code slices.

Then continue all remaining UI-first work/main/parent tasks; no resync investigation or report checkpoint certifies any of33 reference screens.
