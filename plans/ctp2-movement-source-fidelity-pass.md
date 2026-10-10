# CTP2 Movement Source Fidelity Pass

## Goal

Bring in-game unit activation, path preview, drag-to-move submission, and movement feedback closer to the original CTP2 source behavior: selected/activated local armies show source-style brackets, hovering an activated unit previews the computed path, dragging away from the selected army commits only the computed path, and invalid legs render as stop feedback instead of silently disappearing.

## Source References

- `ctp2_source/ctp2_code/ui/aui_ctp2/SelItemClick.cpp` — `SelectArmyStartMoveClick`, `StartMoveClick`, `MoveDrag`, `MoveDrop`, `CancelPathingClick`.
- `ctp2_source/ctp2_code/ui/aui_ctp2/SelItem.cpp` — `SetDrawablePathDest`, `ConstructPath`, `EnterArmyMove`, `GetIsPathing`.
- `ctp2_source/ctp2_code/ui/aui_ctp2/background.cpp` — `ProcessLastMouseMoveThisFrame` CTP2 hover path updates.
- `ctp2_source/ctp2_code/gfx/tilesys/TileHighlight.cpp` — `DrawLegalMove` line colors and turn boxes.
- `ctp2_source/ctp2_code/gfx/spritesys/UnitActor.cpp` — selected-unit bracket color rules.

## Checklist

- [x] Add tests for CTP2 hover path preview after unit activation without mouse button state.
- [x] Add tests proving local fallback movement follows the computed path around impassable terrain instead of recomputing a straight blocked path.
- [x] Add movement-state tests for deterministic source-style path recomputation, invalid-target stop feedback, and hover preview not submitting movement.
- [x] Add rendering-geometry tests for CTP2 path turn boxes / continued line feedback.
- [x] Replace straight-line-only drag planning with deterministic passability-aware pathfinding that recomputes from the army origin to the current cursor destination.
- [x] Preserve the source distinction between hover pathing, a potential click-drag, and an actual drag that left the origin tile.
- [x] Submit/apply only finalized drag paths; never submit hover-only previews or invalid stop paths.
- [x] Render movement feedback before UI chrome, with green/yellow selected-unit brackets, green legal path legs, red invalid legs, and CTP2-style turn boxes.
- [x] Keep popup/dropdown/control-panel interactions consuming and cancelling movement input.
- [x] Run focused tests, `cargo make lint-fix`, `cargo make lint`, `cargo make test`, `cargo build`, coverage script if present, `git diff --check`.
- [x] Review restricted files/file sizes/source-reference notes, update checklist, and commit.

## Verification Notes

- Focused movement tests passed: `cargo test gameplay::runtime::movement --lib`.
- Focused render geometry test passed: `cargo test gameplay::runtime::render::tests::movement_feedback_uses_ctp2_line_bracket_and_turn_box_geometry --lib`.
- `cargo make lint-fix` passed.
- `cargo make lint` passed.
- `cargo make test` passed.
- `cargo build` passed.
- `./scripts/check-coverage-fast.sh` is absent in this checkout; coverage could not be measured with that script.
- `git diff --check` passed.
- File-size review passed; touched source files remain below 800 LOC and no restricted asset/source folders were modified.
