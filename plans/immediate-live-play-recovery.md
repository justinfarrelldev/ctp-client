# Immediate live-play recovery: production catalog and synchronization

Parent: PLAN.md and [full-screen-reference-parity.md](full-screen-reference-parity.md). User confirmed the Strings.txt fix works, then reported cities only offering "Synthetic" and a continuously Resynchronizing client. These are immediate P0 concerns ahead of further visual work. Original source authority: local ctp2_source only. Server is read-only.

## Production
- [x] Audit current server revision, local startup/configuration/retail options, actual bootstrap payloads and received rules/legal-action schema.
- [x] Reproduce the synthetic-only catalog with accurate isolated HTTP fixtures; distinguish a client filter/label bug from an actually synthetic server profile.
- [x] Verify no per-game client profile selector exists: serverRuntimePinForSetupRules uses OPEN_CTP_RULES_MODE/OPEN_CTP_RETAIL_GAMEDATA at server startup. Source/command IDs are unchanged; exact three fixture labels now explicitly say test unit/building/wonder rather than ambiguous Synthetic. Injected actual-shape retail catalogs prove all three original production categories populate and send exact item_id-only edits without inventing costs/effects.
- [x] Preserve city ownership/revision/queue identity, explicit selection and queued-versus-executed semantics. Sent user exact retail restart command (task run with environment); no server/config changes.
- [ ] User restarts retail server and creates a NEW game; existing games remain pinned synthetic. Do not mark live retail production resolved without confirmation.

## Synchronization
- [x] Trace Ready/Pending/Resynchronizing transitions, worker busy/disconnect, SSE parsing/resume/current snapshot requests and server response protocol.
- [x] RED deterministic fragmented/idle/normal-close/error/current-recovery tests, including repeated ticks and no real server contact or file writes.
- [x] Fix root cause while preserving immutable command retries, exact current receipt acknowledgement, monotonic full-snapshot recovery and bounded nonblocking workers.
- [x] UI-thread recovery/receipt tests plus real isolated-worker idle/self-event test prove full receipts remain acknowledgements and already-covered events do not issue redundant current reads or induce HTTP429/Resynchronizing. Stale current snapshots retry rather than leaving UI permanently blocked.

- [x] Separate P0 follow-up: historical order.rejected notices in an already verified snapshot no longer force Resynchronizing, overwrite a newer Pending or roll back unrelated presentation. Display ingestion is immutable/no transport access; UI current-read callback removed. Actual network disagreement/recovery and immutable retry ownership remain intact. Failure-message wording no longer permanently claims recovery. Three REDs, inert-channel status/receipt/current/real-recovery and text regressions pass;1282 executions/85.59% LLVM/lint/full suite/native build. See [main completed sync-ownership ledger](../PLAN.md#completed-implementation-ledger),3df0793.
- [ ] User confirms sustained healthy Ready behavior in live play; isolated fixtures/checks are not live acceptance. New game55/turn2 logs show repeated INFO resync/current reads at roughly3s cadence after an initial50s gap; full cause/ownership/timing/SSE/early-local-turn regression checklist is [local-game-resync-investigation.md](local-game-resync-investigation.md). Normal snapshot replication is not automatically fault recovery, and neither prior limiter nor historical-message fixes prove this remaining symptom resolved.

## Delivery
- [x] Strict lint/full tests >=80% LLVM line gate, dedicated native build, honest Windows/fast-script outcomes, source/privacy/performance review.
- [x] Update FIXES/main/parent evidence, lint-fix before normal commit, unchanged hooks; server/original trees untouched.
Evidence: server15ada01 uses one20/min limiter for both GET events subscriptions and finite current reads. Client previously refreshed on every cursor (even own receipt-covered events), reconnected400ms and retried failed reads100ms. Shared atomic monotonic3100ms admission now covers BOTH threads, including initial bootstrap cooldown; only workers wait and shutdown checks50ms. Coalesced optional revision hints are separate from mandatory recovery and durable sequence. A resync ID is a snapshot revision, NOT Last-Event-ID sequence. Large snapshots stay skipped/bounded; malformed/missing hints require safe reads; matching own revisions do not. Unknown/extra event data is never canonical or an acknowledgement. Stale current reads become retryable failure while older exact receipts can still acknowledge. Header errors retain actual HTTP status.

Verification: strict lint/full tests/dedicated build pass;590 library +608 binary +19 integration +1 example =1218 executions;85.34% LLVM lines, >=80% enforced. Fast script absent127; Windows target/sysroot gap unchanged; every module<800 and server clean. Isolated real worker holds a stream and sends sequence900/901 at revision6 after own receipt6; exactly one GET, zero redundant reads/failures. No live server or file-writing tests.

- [ ] Continue outstanding font discovery and every remaining unchecked main-plan/33-frame acceptance item; do not mistake these P0 fixes for complete game parity.
