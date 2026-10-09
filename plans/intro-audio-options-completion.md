# Intro Sizing, Button Audio, and Complete Options Functionality Plan

## Scope

Fix the CTP2 launch/menu runtime so the intro movie opens at the same configured application resolution, interactive buttons play the original CTP2 button-click sound, and every options-screen control has a modeled state, a runtime effect where applicable, and persisted settings across restarts.

## Required option checklist before implementation

### Gameplay options (`GamePlayOptionsWindow`)

- [x] Tutorial Advice should toggle whether tutorial/advice prompts are enabled.
- [x] Tutorial Advice should be persisted across restarts.
- [x] Auto Cycle Turns should toggle automatic turn cycling.
- [x] Auto Cycle Turns should be persisted across restarts.
- [x] Auto Cycle Units should toggle automatic unit selection, first-unit selection, and deselection as a single original-game option.
- [x] Auto Cycle Units should be persisted across restarts.
- [x] Auto Center should toggle automatic map centering on selected units/cities.
- [x] Auto Center should be persisted across restarts.
- [x] Auto Tab Select should toggle automatic control-panel tab switching.
- [x] Auto Tab Select should be persisted across restarts.
- [x] Autosave should toggle automatic save creation.
- [x] Autosave should be persisted across restarts.
- [x] Battle View should toggle zoomed combat display.
- [x] Battle View should be persisted across restarts.
- [x] Show All Battles should toggle always showing zoomed combat.
- [x] Show All Battles should be persisted across restarts.
- [x] Enemy Moves should toggle showing enemy movement animations.
- [x] Enemy Moves should be persisted across restarts.
- [x] Left Handed Mouse should toggle left-handed mouse behavior.
- [x] Left Handed Mouse should be persisted across restarts.
- [x] Mouse Speed should adjust pointer sensitivity where the runtime has a mouse backend.
- [x] Mouse Speed should be persisted across restarts.

### Graphics options (`GraphicsWindow` and `ScreenResScreen`)

- [x] Unit Speed should adjust unit animation speed.
- [x] Unit Speed should be persisted across restarts.
- [x] Unit Animations should toggle unit animation display.
- [x] Unit Animations should be persisted across restarts.
- [x] Animated Trade Routes should toggle animated trade-route display.
- [x] Animated Trade Routes should be persisted across restarts.
- [x] Wonder Movies should toggle wonder movie playback.
- [x] Wonder Movies should be persisted across restarts.
- [x] City Names should toggle city-name display.
- [x] City Names should be persisted across restarts.
- [x] Political Borders should toggle political-border display.
- [x] Political Borders should be persisted across restarts.
- [x] Trade Routes should toggle trade-route visibility.
- [x] Trade Routes should be persisted across restarts.
- [x] City Influence should toggle city-influence overlay display.
- [x] City Influence should be persisted across restarts.
- [x] Grid should toggle map grid lines.
- [x] Grid should be persisted across restarts.
- [x] Screen Size should list base-game legal resolutions: 800x600 plus modes at least 1024x768, excluding 640x480.
- [x] Screen Size should resize the SDL application window when selected.
- [x] Screen Size should be persisted across restarts.
- [x] The intro movie should use the persisted/current Screen Size so it opens at the same size as the application.

### Sound options (`SoundWindow`)

- [x] Sound Volume should adjust the volume of all sound effects, including button clicks.
- [x] Sound Volume should be persisted across restarts.
- [x] Voice Volume should adjust the volume intended for voice channels when voice playback is added.
- [x] Voice Volume should be persisted across restarts.
- [x] Music Volume should adjust menu/music playback volume.
- [x] Music Volume should be persisted across restarts.

### Music options (`MusicScreen` and `MusicTrackScreen`)

- [x] Select Track should open the original track picker.
- [x] Selecting a track should play the matching `TrackNN.ogg` file.
- [x] Selected Track should be persisted across restarts.
- [x] Auto Repeat should toggle repeat behavior for selected music.
- [x] Auto Repeat should be persisted across restarts.
- [x] Random Order should toggle random/playlist music ordering state.
- [x] Random Order should be persisted across restarts.
- [x] Music On should pause/resume music playback.
- [x] Music On should be persisted across restarts.

### Keyboard mapping options (`KmScreen`)

- [x] Keyboard should open the original keyboard mapping screen instead of being a dead action.
- [x] Basic Commands category should show the original basic command key list.
- [x] Unit Controls category should show the original unit-control key list.
- [x] Screen Controls category should show the original screen-control key list.
- [x] Map Controls category should show the original map-control key list.
- [x] Game Options category should show the original game-options key list.
- [x] Reset Defaults should restore the default key mapping.
- [x] Key category selection should be persisted across restarts.
- [x] Key mappings should be persisted across restarts when remapping support is wired.

### Options root and shared behavior

- [x] Game Play should open the gameplay options screen.
- [x] Graphics should open the graphics options screen.
- [x] Sound should open the sound options screen.
- [x] Music should open the music options screen.
- [x] Keyboard should open the keyboard mapping screen.
- [x] Disabled right-side game-management controls should remain disabled in the pre-game shell.
- [x] Unavailable Cheat Mode should be disabled instead of being a dead enabled button.
- [x] Every interactive enabled button/switch/list item should play the original CTP2 execute click sound (`SOUND_ID_BUTTONCLICK` -> `CCLICK1.WAV`) when clicked.
- [x] Button-click audio should respect Sound Volume.

## Implementation checklist

- [x] Add red tests for intro playback command sizing.
- [x] Refactor external video process launch arguments to open sized 800x600/configured-resolution windows rather than fullscreen players.
- [x] Add red tests proving the original CTP2 button-click sound is resolved from `sound.zfs` as `CCLICK1.WAV`.
- [x] Load and play the original button-click WAV through SDL_mixer for landing and shell clicks.
- [x] Add persistent CTP2 options structures to application config with defaults and TOML round-trip tests.
- [x] Extend shell option state to hydrate from/persist to config.
- [x] Add gameplay options model, labels, controls, state transitions, persistence effects, and tests.
- [x] Correct resolution option list to base-game legal choices and add tests excluding 640x480.
- [x] Queue and apply runtime effects for all sound, music, gameplay, and graphics options with tests.
- [x] Add keyboard mapping screen model/category switching/reset-default state with tests.
- [x] Ensure option effects save to config in production without tests touching real user files.
- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run `./scripts/check-coverage-fast.sh` if present; record unavailability if missing. Coverage note: no coverage script matching `scripts/*coverage*` is present in this checkout.
- [x] Run a dedicated build (`cargo build`).
- [x] Run `cargo make lint-fix` before committing.
- [x] Update this plan with completed checks.
- [x] Commit the finished changes.
