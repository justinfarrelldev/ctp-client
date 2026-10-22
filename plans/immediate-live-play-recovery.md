# Immediate live-play recovery: production catalog and synchronization

Parent: PLAN.md and [full-screen-reference-parity.md](full-screen-reference-parity.md). User confirmed the Strings.txt fix works, then reported cities only offering "Synthetic" and a continuously Resynchronizing client. These are immediate P0 concerns ahead of further visual work. Original source authority: local ctp2_source only. Server is read-only.

## Production
- [ ] Audit current server revision, local startup/configuration/retail options, actual bootstrap payloads and received rules/legal-action schema.
- [ ] Reproduce the synthetic-only catalog with accurate isolated HTTP fixtures; distinguish a client filter/label bug from an actually synthetic server profile.
- [ ] If the client omits a supported profile/setup option, forward the exact supported selection and verify retail records reach original Units/Buildings/Wonders categories. No locally invented production authorization, item cost or synthetic-record renaming.
- [ ] Preserve city ownership/revision/queue identity, explicit selection and queued-versus-executed semantics. Report exact server-only bug/configuration/start command if required.

## Synchronization
- [ ] Trace Ready/Pending/Resynchronizing transitions, worker busy/disconnect, SSE parsing/resume/current snapshot requests and server response protocol.
- [ ] RED deterministic fragmented/idle/normal-close/error/current-recovery tests, including repeated ticks and no real server contact or file writes.
- [ ] Fix root cause while preserving immutable command retries, exact current receipt acknowledgement, monotonic full-snapshot recovery and bounded nonblocking workers.
- [ ] Native/semantic transitions verify recovered state remains Ready on idle and new commands are not constantly blocked by needless recovery.

## Delivery
- [ ] Strict lint/full tests >=80% LLVM line gate, dedicated native build, honest Windows/fast-script outcomes, source/privacy/performance review.
- [ ] Update FIXES/main/parent evidence, lint-fix before normal commit, unchanged hooks; server/original trees untouched.
- [ ] Continue outstanding font discovery and every remaining unchecked main-plan/33-frame acceptance item; do not mistake these P0 fixes for complete game parity.
