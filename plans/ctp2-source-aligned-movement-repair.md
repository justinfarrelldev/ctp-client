# CTP2 Source-Aligned Movement Repair

## Original source references reviewed

- `ctp2_source/ctp2_code/gfx/tilesys/tiledmap.cpp`
  - `TiledMap::Click`, `TiledMap::MouseDrag`, `TiledMap::Drop`
  - mouse positions are converted to `MapPoint` and routed into `SelectedItem::RegisterClick`.
- `ctp2_source/ctp2_code/ui/aui_ctp2/background.cpp`
  - left-grab, left-drag, and left-drop events are queued and delivered to `TiledMap`.
  - `ProcessLastMouseMoveThisFrame` updates the drawable path destination for the selected local army in CTP2 mode.
- `ctp2_source/ctp2_code/ui/aui_ctp2/SelItem.h`
  - movement state: `m_is_pathing`, `m_cur_mouse_tile`, `m_good_path`, `m_bad_path`, `m_waypoints`, `m_isDragging`, `m_startDragPos`, `m_moveOrder`.
  - click modes/buttons: `SELECT_MODE_CTP2`, `SELECT_BUTTON_LEFT_PATHING`, `SELECT_BUTTON_LEFT_DRAG`, `SELECT_BUTTON_LEFT_DROP`.
- `ctp2_source/ctp2_code/ui/aui_ctp2/SelItem.cpp`
  - `SelectedItem::RegisterClick` / `NewRegisterClick` choose click handlers and convert CTP2 pathing clicks.
  - `SelectedItem::EnterArmyMove` queues `GEV_MoveOrder` with the constructed path.
- `ctp2_source/ctp2_code/ui/aui_ctp2/SelItemClick.cpp`
  - `StartMoveClick` records a potential drag start.
  - `MoveDrag` begins order delivery after the cursor leaves the start tile.
  - `MoveDrop` dispatches movement only when a real pathing drag/drop exists.
- `ctp2_source/ctp2_code/gfx/tilesys/TileHighlight.cpp`
  - `DrawLegalMove` renders path lines; it does not paint every reachable tile as a movement marker.
  - `DrawUnfinishedMove` renders queued path continuation lines.
- `ctp2_source/ctp2_code/gfx/spritesys/UnitActor.cpp`
  - selected units render bracket overlays, green when the army can move.

## Behaviour to match

- [x] Selecting a human unit and starting a movement drag are part of the same initial left mouse grab, like `SelectArmyStartMoveClick`.
- [x] Releasing on the origin tile with no planned path only selects/activates; it must not move or submit a server command.
- [x] Dragging away from the selected unit computes a path from the unit origin to the current cursor destination, rather than accumulating every cursor tile as a freehand route.
- [x] Releasing a valid drag submits the exact computed path order to the authoritative server.
- [x] Plain left-clicking terrain remains a no-op.
- [x] Popups, dropdowns, and lower/top UI chrome continue to consume/cancel movement input.
- [x] Remove reachable/moveable tile markers from gameplay rendering.
- [x] Render CTP2-style selected-unit feedback as brackets instead of map-wide moveable overlays.
- [x] Render movement planning as line segments between path tile centers, following the original `DrawLegalMove` approach.
- [x] Keep Save/Load menu rows visible but disabled/no-op.
- [x] Do not add automatic resyncs for 409 responses in this movement-only repair.

## TDD checklist

- [x] Add/adjust tests proving first click-drag from an owned unit creates a path and moves on release.
- [x] Add/adjust tests proving drag planning recomputes from the unit origin to the current destination instead of preserving cursor detours.
- [x] Add/adjust rendering geometry tests for line-based movement feedback and selection brackets.
- [x] Run the focused test set and confirm the new expectations fail before implementation.
- [x] Implement input-state changes.
- [x] Implement rendering changes and remove reachable/moveable markers.
- [x] Run `cargo make lint-fix`.
- [x] Run `cargo make lint`.
- [x] Run `cargo make test`.
- [x] Run a dedicated build step.
- [x] Update this checklist and commit the repair.

## Verification notes

- `cargo make lint-fix` passed.
- `cargo make lint` passed.
- `cargo make test` passed.
- `cargo build` passed.
- `git diff --check` passed.
- `./scripts/check-coverage-fast.sh` is still absent in this checkout, so coverage could not be measured.
