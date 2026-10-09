# Custom Map and New Game UI Fidelity

## Research Checklist

- [x] Read original `custommap.ldl` and `custommapscreen.cpp` for slider geometry, defaults, labels, and button behavior.
  - The six sliders are independent `C3Slider`s at x-center, y 40/80/120/160/200/240, width 250, range 0..10, quantized.
  - Base `profile.txt` defaults are WetDry=5, WarmCold=5, OceanLand=3, IslandContinent=5, HomoDiverse=8, GoodCount=5.
  - Custom map uses `c3_PopupWindow::AddOk`, i.e. a reusable `c3_PopupOk` 70x24 `CTP2_BUTTON_SMALL` moved to width-70-17, height-24-17; the LDL `BackButton` geometry is not the live button.
- [x] Read original `spnewgame.ldl`, `spnewgamepopups.ldl`, and related source for setup defaults and popup radio alignment.
  - Base `profile.txt` defaults are Difficulty=2 (Medium), RiskLevel=1 (Bandits), MapSize=1 (Regular), NumPlayers=6 displayed as 5 empires, XWrap=Yes/YWrap=No (Earth World), Pollution=Yes.
  - Popup radio rows must reflect current setup state rather than always rendering unselected.
  - World Shape uses explicit LDL row y positions 45, 80, 115, with description labels at 51, 84, 116.
- [x] Audit current reusable shell control builders/rendering/hit-testing to identify button/text layout schisms.
  - Current custom-map Back uses a one-off 40x20 blue large button, narrower than the native 70px art footprint and visually off-center.
  - Current new-game labels are drawn before later button controls, while the original source instantiates key labels after the setup buttons; native button art then covers label pixels in our renderer.
  - Current custom-map sliders all share `ShellOptionField::UnitSpeed`, so each slider edits a persisted graphics option instead of its own game setup value.
- [x] Add tests that reproduce every identified defect before implementation.

## Defect Checklist

- [x] Fix the Wet / Dry slider to move visually and update its own non-persisted custom-map value.
- [x] Fix the Warm / Cold slider to move visually and update its own non-persisted custom-map value.
- [x] Fix the Ocean / Land slider to move visually and update its own non-persisted custom-map value.
- [x] Fix the Island / Continent slider to move visually and update its own non-persisted custom-map value.
- [x] Fix the Uniform / Diverse slider to move visually and update its own non-persisted custom-map value.
- [x] Fix the Few Goods / Many Goods slider to move visually and update its own non-persisted custom-map value.
- [x] Fix Custom Map slider defaults to match the original base-game profile defaults.
- [x] Fix Custom Map Back button geometry/text centering so click and label feel consistent.
- [x] Reuse the shared popup/shell button builder/rendering path for Custom Map buttons instead of one-off geometry.
- [x] Fix World Shape popup radio/default selection so Earth World is selected by default.
- [x] Fix World Shape popup text alignment next to the actual selectors.
- [x] Fix Difficulty popup so Medium is selected by default.
- [x] Fix Difficulty companion barbarian-risk/default selection to match the base-game default and keep it interactive.
- [x] Fix Map Size popup so Regular is selected by default.
- [x] Fix Players popup so the base-game default 5 displayed empires is selected by default.
- [x] Fix Rules popup defaults so visible toggles match base-game defaults.
- [x] Fix New Game header/button vertical overlap so labels such as Difficulty are not cut off by rendered button art.
- [x] Validate all changed shell controls use reusable builders and shared render/hit-test behavior where possible.
- [x] Run `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage check if available, build, and `git diff --check`.
  - `./scripts/check-coverage-fast.sh` is not present/executable, so coverage could not be run.
- [x] Final review and commit.
