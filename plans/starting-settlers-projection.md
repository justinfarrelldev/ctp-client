# Diagnose missing starting settlers and visibility

## Scope
Trace the server's local-game spawn, ownership, fog refresh and SSE/receipt wire shape into the client's model and renderer. Do not invent local units/terrain, change the main menu or modify server implementation. Tests use accurate in-memory/mock HTTP fixtures only, never a live game or file-writing capture sink.

## Checklist
- [x] Inspect server spawn/start, viewer identity/visibility, and event-envelope contracts read-only.
- [x] Inspect client bootstrap/SSE decoding, owner mapping, tile/unit conversion, and camera/render visibility.
- [x] Inspect `images/current-ingame.png`; check live `/health` read-only (healthy) and only relevant server process configuration (default synthetic rules). No live game/account was created, read or advanced for testing.
- [x] Add failing mocked regressions reproducing the actual wire contract, plus malformed/empty/negative cases.
- [x] Fix the client integration without local-world fallbacks or authority violations. No server defect requiring an implementation patch was found.
- [x] Verify authoritative starting units, visible tiles, selection, camera, Move/Settle controls and minimap with in-memory tests for synthetic and retail profiles.
- [x] Run lint-fix, strict lint, full tests, native build, coverage and diff/security/portability review.
- [x] Update findings, FIXES.txt and PLAN.md; run pre-commit lint-fix and commit normally with verification enabled.

## Source evidence and root causes

### Server does spawn starter units

`$CTPSERVER/internal/game/start_gameplay.go::initialGameplayState` creates `StartingSettlers` units per player on generated spawn tiles, using the pinned runtime starter definition. It validates the total and that every player has an owned starter. `createStartedGameplay` persists canonical state and applies the initial no-op. The visibility update/projection path in `internal/gameplay/visibility/visibility.go` reveals owned-unit sight and includes their unit records.

With the client's requested one starter, this is a server-owned unit, not a client-generated preview. No server start/spawn/visibility implementation was changed.

### Client decoded the SSE envelope as an empty projection

`internal/gameplay/events/events.go::Store.resync` emits:

```text
event: resync
data: {"current_revision":1,"snapshot":{"revision":1,"turn":1,"tiles":[...],"units":[...],"cities":null}}
```

The old client deserialized this whole object directly as `ServerProjection`. Unknown envelope fields were ignored and every projection field defaulted, producing revision zero and no tiles/units. This exactly explains the screenshot's empty unit panel, black map and empty radar. Earlier fabricated local-world fallbacks masked this protocol mismatch; restoring such a fallback would be wrong.

The new decoder extracts only `snapshot`, requires projection revision/turn, checks the envelope revision, rejects invalid/missing snapshots and handles standard SSE framing. A legitimate empty viewer snapshot remains empty; it is not replaced by local geography or armies.

### Go nil slices must be accepted

The server's `Projection.Cities` is nil before founding a city and encodes as JSON `null`, not `[]`. `#[serde(default)]` alone does not accept explicit null. Nullable typed-vector decoding now handles tiles, units, cities and optional unit effects in resyncs and direct command receipts. Wrong scalar/object list values still fail.

### Default server starter has an opaque type name

The running server has no `OPEN_CTP_RULES_MODE=retail` override. `rules.ServerOwnedSyntheticRuleset` defines `UNIT_SYNTHETIC` as its starter, with the canonical `settler` effect, movement 2 and sight 2. The previous name-only mapping classified it as a warrior, hiding settlement controls even after fixing the envelope.

Client render-family conversion now recognizes the exact viewer-exposed `settler` effect while retaining retail `UNIT_SETTLER` support. It does not guess from instance/effect IDs or substring-match unrelated effects such as `special_settler_slave_raid`. Original server identities remain intact for commands. This is capability-based client integration, not a change to server rules or authority.

## Red / green and verification

- `/tmp/ctp-starting-settler-red.log`: all four initial envelope/nil-list regressions failed before the fix.
- `/tmp/ctp-starting-bootstrap-red.log`: real-format mocked HTTP launch returned revision zero before the fix.
- `/tmp/ctp-synthetic-settler-red.log`: after envelope correction, the synthetic starter still failed render-family/settlement selection before effect-based classification.
- Accurate fixtures follow `events.Store.resync`, `visibility.Projection` and start/rules code. Thirteen unobstructed visible cells model the server's Manhattan radius 2; this is not the unrelated hand-built visual screenshot scene.
- Mocked bootstrap verifies `starting_settlers:1`, one-shot resync request header, revision, starter identity/capability, visible tiles and nil cities.
- In-memory SDL tests verify both rule profiles: authenticated owner, selection, real command identity, centered non-default spawn position, actual actor/terrain/radar pixels and semantic Move/Settle intents. Unknown cells stay hidden. No screenshot files or live game requests occur in tests.
- `cargo make lint-fix`, `cargo make lint`, `cargo make test` (308 library, 326 binary, 19 integration tests) and `cargo build` pass.
- `./scripts/check-coverage-fast.sh` was attempted but is absent. Available alternative `cargo llvm-cov --all-features --all-targets --summary-only` passes with **80.35% line coverage**.
- Changed Rust files remain below 800 lines. No restricted files, assets, server implementation or main-menu files changed. LF/CRLF decoding and generic Rust/SDL paths remain portable; Windows binary validation was not performed on this Linux host.

## User follow-up

Restart the rebuilt client and press Launch again; the already-running server does not need a restart or patch. The existing empty window does not hot-reload code. Info logging now reports resync revision/turn and tile/unit/city counts without session credentials. Gameplay is still server-authoritative.
