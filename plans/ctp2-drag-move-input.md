# CTP2 Drag Move Input Plan

## Goal

Stop accidental authoritative `move` submissions from ordinary left clicks and align in-game unit movement with the original CTP2 interaction model: left-click a unit to activate it and show the green reticle, then click-drag from that active unit to draw a path and release to submit that path. Simple left-clicking a terrain tile must do nothing.

## Source References

- `ctp2_source/ctp2_code/gfx/tilesys/tiledmap.cpp` — original tile/unit interaction reference.
- `ctp2_source/ctp2_code/ui/interface/controlpanelwindow.cpp` — control panel/menu click containment.
- `ctp2_source/ctp2_code/gfx/spritesys/*` — selected-unit/activation rendering context.
- `src/gameplay/runtime.rs` — SDL input routing and authoritative command submission.
- `src/gameplay/runtime/render.rs` — selected-unit reticle/path rendering.
- `src/gameplay/model/actions.rs` — local fallback movement/path semantics.

## Checklist

### A. Protect Against Accidental Server Moves

- [x] Add tests proving a plain left-click on a map tile does not move locally and does not submit an authoritative move.
- [x] Add tests proving clicks on UI/popup/menu areas continue to consume input without map movement.
- [x] Add tests proving keyboard shortcuts are blocked by visible modal popups except popup dismissal.

### B. Add CTP2 Unit Activation State

- [x] Add a testable runtime/input state that tracks an activated unit independently from model selection.
- [x] Add tests proving clicking a visible owned unit activates/selects it and does not submit movement.
- [x] Ensure activation survives render/update only while the same unit remains selectable.
- [x] Render the activation reticle in green rather than treating every model-selected unit as an active order target.

### C. Add Drag Path Planning

- [x] Add tests proving mouse-down on the active unit starts drag planning.
- [x] Add tests proving dragging over map tiles accumulates a deterministic path with no duplicate consecutive tiles.
- [x] Add tests proving mouse-up without a dragged path submits nothing.
- [x] Add tests proving mouse-up with a dragged path submits exactly that path to the server command payload.
- [x] Draw the planned path while dragging with visible CTP2-style green path markers.

### D. Preserve Server Authority and Local Fallback

- [x] For authoritative sessions, submit only the finalized drag path and rebuild local state only from the server projection.
- [x] For non-authoritative tests/local fallback, apply the finalized path locally.
- [x] Keep keyboard movement commands server-authoritative but do not let modal popups leak keys into commands.
- [x] Do not add automatic resyncs for this fix.

### E. Verification and Commit

- [x] Run targeted tests while implementing.
- [x] Run `cargo make lint-fix` before commit.
- [x] Run `cargo make lint`.
- [x] Run `cargo make test`.
- [x] Run `cargo build`.
- [x] Attempt `./scripts/check-coverage-fast.sh`; record if absent: script is missing in this checkout.
- [x] Run `git diff --check`.
- [x] Review file sizes/restricted files/no asset copying/no file-mutating tests.
- [x] Commit with a clear message.
