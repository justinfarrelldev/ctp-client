# UI Schisms

Known differences between the current Rust/SDL shell and the original CTP2 shell, plus fidelity notes found while validating against LDL/source.

## Current pass findings

- Original popup close controls are not LDL `AcceptButton` rectangles. `c3_PopupWindow::AddClose` creates a `c3_PopupClose` button at `width - 70 - 17`, `height - 24 - 17`; screens now use that model where source calls `AddClose`.
- Graphics options use switch controls backed by original checkbox art; they should be stateful toggles, not static disabled buttons.
- `SPNewGameTribeScreen` updates the leader-name field from civilisation/string DB defaults when a nation radio is selected and keeps the modal open until Close/Back.
- Leader-name text fields honor `maxfieldlen 23`; current editing approximates original field behavior with select-all-on-focus replacement.
- Resolution changes should preserve the original 800x600 shell coordinate space without stretching/smoothing the art. Larger windows are letterboxed and hit testing is translated back into menu coordinates.

## Still known approximations

- Original CTP2 font metrics and text renderer are still approximated with system fallback fonts.
- Popup frame edges/title chrome are approximated by tiled pattern panels plus modeled controls rather than the full AUI popup renderer.
- Several game-rule/barbarian-risk controls are represented visually but are not yet wired into authoritative server/game setup state.
- Text-field caret/selection drawing is not yet a pixel-perfect clone of AUI `c3_EditField`.
