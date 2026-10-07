# Shell UI Fidelity Review and Correction Plan

## Scope

Review the implemented CTP2 landing/single-player shell UI against original LDL/source behavior and correct mismatches in theme, layout, control art, hover/click semantics, and picker aesthetics without mutating/copying original assets.

## Review Findings to Validate

- Landing page: background/button art is close to original after prior work; continue validating no regressions.
- Single-player hub: uses original `upsg002` background and button families; some disabled/offscreen controls are modeled as visible/disabled and need original visibility/position semantics checked.
- New-game screen: main shell background/control positions are approximate but mostly anchored to `spnewgame.ldl`; popup launches need exact original popup layouts.
- Nation picker: current implementation is a generic small list popup; original `SPNewGameTribeScreen` is a 650x465 modal with 4 columns x 11 rows of radio controls, `upba2317/2318` checkbox art, leader name field, male/female radios, title, and back control.
- Difficulty picker: current generic list popup misses original 400x270 two-column skill/barbarian layout with checkbox art and exact labels.
- Map/map-size/map-shape/player/rules pickers: current generic list popup misses per-screen dimensions, radio/toggle art, explanatory labels, and back locations from `spnewgamepopups.ldl`.
- Resolution picker: current generic picker uses original dimensions but lacks original listbox pattern/header styling and warning location from `graphics.ldl`.
- Music track picker: current generic picker should be reviewed against `musicscreen.ldl` and original source; use original popup pattern/list styling where available.
- Switches/sliders/list items: current renderer uses flat fallback rectangles for checkbox/list-like controls; original uses immutable art such as `upba2317.tga`, `upba2318.tga`, `upbt11u0/u1.tga`, `upba2513.tga`, and patterned listboxes.

## Implemented Corrections and Remaining Distinctions

- Corrected: setup picker popups now use LDL-specific dimensions and control placement instead of the previous generic 235x234 list popup.
- Corrected: nation picker now uses the original 650x465 four-column tribe-radio layout, leader-name field, gender radios, and `upba2317/upba2318` checkbox/radio art.
- Corrected: difficulty, map, map-size, map-shape, players, and rules popups now use the original popup pattern, radio/toggle art, and first-control geometry from `spnewgamepopups.ldl`.
- Corrected: graphics and music toggles carry original image families (`upbt11u0/u1` and `upba2317/2318`) and render from immutable RIM entries with chroma-key transparency.
- Remaining distinction: text rendering still uses the installed fallback serif font and approximates CTP2 shadow/standard font behavior until original font metrics/text renderer are implemented.
- Remaining distinction: some game-rule and barbarian-risk popup controls are modeled for visual fidelity but are not yet wired to authoritative gameplay/server settings.
- Remaining distinction: original AUI popup frame/close behavior is approximated by patterned popup panels and modeled back controls; a full AUI-equivalent frame system remains future work.

## Checklist

### Tests first

- [x] Add model tests proving `SPNewGameTribeScreen` has original 650x465 geometry, four 146x24 radio columns, 30-pixel row spacing, leader-name controls, gender radios, and back control.
- [x] Add model tests proving difficulty/map/map-size/map-shape/player/rules popup dimensions and key controls match `spnewgamepopups.ldl`.
- [x] Add tests proving popup switches/checkboxes carry original image asset metadata (`upba2317`, `upba2318`, disabled art) rather than fallback list rectangles.
- [x] Add tests proving graphics/sound/music switches and sliders expose original image assets and are rendered through immutable RIM entries.
- [x] Add headless smoke tests covering single-player -> nation picker -> select nation -> new game and options -> graphics/sound/music interactions.

### Implementation

- [x] Replace generic setup picker builder with LDL-specific builders for tribe, difficulty, map, map size, map shape, players, and rules screens.
- [x] Add reusable shell art metadata for checkbox/radio/toggle/small image-button controls.
- [x] Render shell image-backed controls from original RIM assets with magenta chroma-key transparency.
- [x] Render selectable popup radio/toggle controls with original image0/image1 families and correct enabled/selected state.
- [x] Render original-ish popup text fields/listbox patterns and warning labels instead of flat placeholder styling.
- [x] Update setup option labels/defaults to match original translated strings and source semantics.
- [x] Record unresolved differences that require deeper engine/UI work.

### Verification and delivery

- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run `cargo build`.
- [x] Run `./scripts/check-coverage-fast.sh` if present; record if unavailable (`coverage script unavailable: ./scripts/check-coverage-fast.sh`).
- [x] Run `cargo make lint-fix` before commit.
- [x] Commit the completed UI fidelity correction atomically.
