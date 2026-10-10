# Authoritative Projection Move Fix

## Goal

Stop server-backed movement from showing the confusing "tile missing from the authoritative projection" popup during normal play. The client must not plan or submit paths through hidden/generated fallback tiles that the Open CTP server has not exposed in the viewer projection.

## Source References

- `src/gameplay/runtime.rs` — authoritative move submission and popup.
- `src/gameplay/runtime/authority.rs` — server tile-id lookup from the current projection.
- `src/gameplay/runtime/movement.rs` — CTP2 movement preview/pathfinding.
- `src/gameplay/model/movement_orders.rs` — local/keyboard movement order validation.
- `src/gameplay/model/server_projection.rs` — server projection conversion to render state.
- `$CTPSERVER/internal/gameplay/visibility/visibility.go` — projection contains visible and remembered tiles only.

## Checklist

- [x] Add a runtime movement test proving hidden/unprojected tiles produce CTP2 bad-path feedback and never a submittable move path.
- [x] Add a model movement-order test proving selected units cannot be ordered into hidden/unprojected tiles.
- [x] Update path validation to require known visible/explored terrain for land movement.
- [x] Run focused tests.
- [x] Run `cargo make lint-fix`, `cargo make lint`, `cargo make test`, `cargo build`, coverage script if present, and `git diff --check`.
- [x] Review, update checklist, and commit.
