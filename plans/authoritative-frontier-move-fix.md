# Authoritative Frontier Move Fix

## Goal

Correct the over-conservative hidden-tile movement fix: if a tile is in the client-visible map area but absent from the current viewer projection, the player must still be able to order movement into it to explore. The client should submit the server's canonical local tile id (`tile-x-y`) instead of blocking the path as "missing from projection".

## Source References

- `src/gameplay/runtime/authority.rs` — maps local `MapPoint`s to server tile ids.
- `src/gameplay/runtime/movement.rs` — CTP2 movement planning.
- `src/gameplay/model/movement_orders.rs` — local fallback movement validation.
- `$CTPSERVER/internal/game/start_gameplay.go` — local server tile ids are `tile-%d-%d`.
- `$CTPSERVER/internal/gameplay/engine/units.go` — move commands validate canonical tile ids against server state.

## Checklist

- [x] Add tests proving hidden/frontier map tiles remain submittable movement destinations.
- [x] Add tests proving missing projection coordinates synthesize canonical server tile ids while preserving projected ids when present.
- [x] Revert the hidden-fog movement block and keep terrain/in-bounds validation.
- [x] Add canonical tile-id fallback for unprojected frontier coordinates.
- [x] Run focused tests.
- [x] Run `cargo make lint-fix`, `cargo make lint`, `cargo make test`, `cargo build`, coverage script if present, and `git diff --check`.
- [x] Review, update checklist, and commit.
