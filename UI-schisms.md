# UI Schisms

This file is the current CTP2 UI fidelity audit.  Source references are the immutable base-game LDL/source files under `ctp2/` and `ctp2_source/`; no assets were copied.

## Audit sources and global styling rules

- Common fonts from `ctp_template.ldl`:
  - `CTP2_STANDARD_FONT`: `timesbd.ttf`, 10 pt, RGB `(50,50,50)`.
  - `CTP_GREY_TITLE_FONT`: `times.ttf`, 14 pt, RGB `(90,70,60)`, shadow RGB `(255,245,245)`.
  - `CTP_GOLD_TITLE_FONT`: `timesbd.ttf`, 16 pt, RGB `(60,60,40)`, shadow RGB `(200,200,100)`.
- Shell button art:
  - `SP_BUTTON_BIG`: `upbt07[a-c][X/U/D/H].tga`, chromakey, shadow off, base 99x20.
  - `SP_BUTTON_BIG_STONE`: `upbt06[a-c][X/U/D/H].tga`, chromakey, shadow off, base 99x20.
  - `c3_PopupClose` / `CTP2_BUTTON_SMALL`: `upbt01[a-c][X/U/D/H].tga`, 70x24, placed by `c3_PopupWindow::AddClose` at `(width - 87, height - 41)`.
- Popup and switch art:
  - Popup pattern: `uptg20e.tga`.
  - Popup radio/toggle family: `upba2317.tga`, `upba2318.tga`, disabled `upba2323.tga`, `upba2324.tga`.
  - Graphics checkbox family: `upbt11u0.tga`, `upbt11u1.tga`, disabled `upbt11x0.tga`, `upbt11x1.tga`.
  - Sliders use thumb `upba2513.tga`.
- Current renderer status: original background/button/switch/pattern art is loaded when present, but text rendering is still a system-font approximation and does not yet reproduce AUI font metrics, grey/gold title colors, title shadows, text blit modes, or text-field caret/selection drawing. This styling schism affects every rendered label and text field.

## Removed or quarantined unoriginal in-game UI

- Removed from the active model: the previous `WorldTypePicker` (`RANDOM MAP`, `CONTINENTS`, `ISLANDS`, `CUSTOM MAP`). `SPNewGameWindow.MapButton` maps to `spnewgame.ldl:465` and `spnewgamescreen.cpp:389`, which opens `custommapscreen_displayMyWindow()`, not `SPNewGameMapScreen`. Until `CustomMapScreen` is implemented, the visible button remains as original `MAP` and is disabled.
- Removed from the difficulty popup: the fifth barbarian-risk row, `No Danger Here`. `SPNewGameDiffScreen` LDL contains the row, but `spnewgamediffscreen.cpp` instantiates only `k_NUM_RISKBOXES = 4` radios (`RiskOne`..`RiskFour`).
- Removed from the default new-game screen: `ScenarioStaticText` and `ScenarioName`. `SPNewGameWindow::Update` hides both when no scenario is active; only `ScenarioButton` remains visible as `Select Scenario`.

## Non-game bootstrap UI with no CTP2 shell equivalent

These are current app UI surfaces, but they are not base-game CTP2 UI. They are retained only as pre-game client/bootstrap scaffolding and must not be considered part of the in-game shell.

| Current UI | Base-game mapping | Styling status / schism |
|---|---|---|
| GTK application window, header title, settings icon, "Select Installation" title, CTP1/CTP2 installation sections, install status labels, Play buttons, Browse buttons, separator, Exit button | No base-game equivalent; the original executable starts from installed game data directly. | GTK/Adwaita styling, margins, icons, and layout do not match CTP2. CTP1 controls are especially outside the current CTP2 target. |
| Add-installation dialogs and native folder picker | No CTP2 shell equivalent. | GTK/rfd native styling; not original. |
| Settings dialog, language search, combo box, Save/Cancel | No CTP2 shell equivalent. | GTK styling; not original. |
| Asset-extraction confirmation/error dialogs | No CTP2 shell equivalent. | GTK modal styling; not original. |
| External ffplay/VLC intro-video window title and skip-instruction overlay metadata | Base game plays the intro movie without this auxiliary instruction UI. | External-player/window-manager styling; not original. |

## CTP2 shell inventory and mapping

### `InitPlayWindow` landing page (`spnewgame.ldl`)

| Current element | Base-game element | Original styling | App status |
|---|---|---|---|
| Background | `InitPlayWindow.Background`, `upsg001.tga` | Full 800x600 shell art | Uses `upsg001.rim` when available. |
| Multiplayer | `MPButton`, `(36,455,290,32)` | `SP_BUTTON_BIG` / `upbt07*`, text `MULTIPLAYER` | Mapped. Text font metrics approximate. |
| Single Player | `SPButton`, `(478,455,290,32)` | `SP_BUTTON_BIG`, text `SINGLE PLAYER` | Mapped. |
| Credits | `CreditsButton`, `(36,558,140,32)` | `SP_BUTTON_BIG_STONE` / `upbt06*`, text `CREDITS` | Mapped. |
| Quit | `QuitButton`, `(624,558,140,32)` | `SP_BUTTON_BIG_STONE`, text `QUIT` | Mapped. |
| Version | `VersionString`, `(720,515,60,24)` | RGB `(100,100,255)`, text `1.1` | Mapped; font metrics approximate. |

`SpriteTestButton`, `EmailButton`, and `HotseatButton` are original LDL/source controls but are not current visible app UI; they are therefore not current schisms.

### `SPWindow` single-player hub (`spnewgame.ldl`)

All current hub controls map to original LDL/source positions: `NewGameButton`, `LoadGameButton`, `ScenarioButton`, `MapEditorButton`, `OptionsButton`, off-screen `TutorialButton` at `x=850`, `ReturnButton`, `QuitButton`, `InstaStartButton`, and background `upsg002.tga`. Button art maps to `SP_BUTTON_BIG` or `SP_BUTTON_BIG_STONE`; `InstaStartButton` is the original static/font control using `error.tga`. Remaining schism: text rendering/color/shadow is approximate.

### `SPNewGameWindow` (`spnewgame.ldl`, `spnewgamewindow.cpp`)

Mapped current controls: `Title`, `CivilizationLabel`, `TribeButton`, `LeaderNameLabel`, editable `Name`, `DifficultyLabel/Button`, `PlayersLabel/Button`, `RulesLabel/Button`, `WorldShapeLabel/Button`, `MapSizeLabel/Button`, `MapButtonLabel/MapButton`, `EditorButtonLabel/EditorButton`, `ScenarioButton`, `StartButton`, `ReturnButton`, `QuitButton`, `InstaStartButton`, and background `upsg003.tga`.

Recent corrections made in this pass: label text/geometry restored for `# EMPIRES`, `MAP SIZE`, `CUSTOMIZE MAP`; button values now use original visible strings (`Medium`, `Regular`, `Earth World`, numeric player count, `Bloodlust - Off`); hidden scenario caption/name removed from default UI; `MapButton` no longer opens the non-original world-type picker; editor-launch controls now display user-facing text (`SCENARIO EDITOR`, `Launch Editor`) instead of raw LDL tokens.

Remaining schism: text field drawing is a beige rectangle/border approximation of `FIELD`/`c3_EditField` (`bevelwidth 1`, `beveltype 2`, `maxfieldlen 23`).

Resolved in this pass: `MapButton` now opens a modeled `CustomMapWindow` based on `custommap.ldl` instead of being disabled or opening a non-original world-type picker.

### Options root (`OptionsWindow`, `options.ldl`, `optionswindow.cpp`)

Mapped current controls: title `Name`, `ConfigHeader`, `GamePlayButton`, `GraphicsButton`, `SoundButton` (`str_ldl_SFX` renders as `SOUND`), `MusicButton`, `KeyboardButton`, `GameHeader`, disabled `SaveGameButton`, disabled `LoadGameButton`, disabled `RestartButton`, disabled `NewGameButton`, disabled `MapEditorButton` (`CHEAT MODE`), and generated `c3_PopupClose`. `QuitToShellButton` is not exposed in the active base-game options popup.

Remaining schism: popup chrome is only the tiled `uptg20e` pattern plus generated Close button; the full `c3_PopupWindow` frame/title renderer is not cloned yet.

### Graphics options (`GraphicsWindow`, `graphics.ldl`, `graphicsscreen.cpp`)

Mapped current controls: title `GRAPHICS`, `UnitSpeedSlider`, `UnitSpeedName` (`Unit Speed:`), `WalkButton`, `TradeButton`, `WonderButton`, `CityNamesButton`, `PoliticalBordersButton`, `TradeRoutesButton`, `CityInflenceButton`, `GridButton`, `ResolutionButton` (`Screen Size`), and generated Close.

Original styles: `GRAPHICS_CHECKBOX` art `upbt11u0/u1/x0/x1`, 300x24; slider range 0..9, horizontal, quantized, thumb `upba2513.tga`; popup pattern `uptg20e.tga`.

Remaining schisms:
- Slider track/ticks are approximate; only the thumb art maps.
- Text colors/case now use original strings, but AUI font metrics/shadows are not exact.

### Screen resolution picker (`ScreenResScreen`, `graphics.ldl`, `graphicsresscreen.cpp`)

Current mapped controls: title `Screen Size`, row items for `640 x 480`, `800 x 600`, `1024 x 768`, and generated Close.

Resolved in this pass: `ScreenResScreen` now models `ResList` listbox chrome and the red restart warning label (`str_ldl_RES_REBOOT_WARNING`) while keeping selectable resolution rows inside the original list area.

### Sound options (`SoundWindow`, `sound.ldl`, `soundscreen.cpp`)

Mapped current controls: title `SOUND`, labels `Sound Volume`, `Voice Volume`, `Music Volume`, sliders `SFXSlider`, `VoiceSlider`, `MusicSlider`, and generated Close. Sliders use original 0..10 ranges and thumb art.

Remaining schism: slider tracks/ticks are approximated rather than the full AUI slider/pattern renderer.

Resolved in this pass: original min/max labels (`0`/`10`) are modeled beside each sound slider.

### Music options (`MusicScreen`, `musicscreen.ldl`, `musicscreen.cpp`)

Mapped current controls: title `MUSIC`, `SelectTrackButton` (`Select Track`), `AutoRepeatSwitch`, `RandomOrderSwitch`, `MusicOnSwitch`, and generated Close. Toggle art maps to `upba2317/upba2318` and labels use `MS_Strings`/music-on stringtable text.

Remaining schism: original switch state is read from `SoundManager`/profile at display time. Current default state is modeled locally.

### Music track picker (`MusicTrackScreen`, `musicscreen.ldl`, `musictrackscreen.cpp`)

Current rows map to original `TrackNames` strings: `A Fortiori`, `Pacari Tampu`, `Amaethon`, `A'tugewinu`, `Summum Bonum`, `Casus Belli`, `Ankh Djet`, `Shaolin San`, `Quiet Outback`, `Kali Dance`.

Resolved in this pass: `MusicTrackScreen` now models the original 177x100 `TrackList` listbox chrome and keeps the `TrackNames` rows inside it.

Remaining schism: popup title uses current text rendering instead of original `CTP2_STANDARD_FONT` title styling.

### New-game picker popups (`spnewgamepopups.ldl`)

| Current popup | Base-game mapping | Current status |
|---|---|---|
| Nation picker | `SPNewGameTribeScreen`; title `Empire`, 41 tribe radios in four original columns, leader-name label/field, Male/Female radios, generated Close | Geometry and art mapped. Text field styling approximate. |
| Difficulty picker | `SPNewGameDiffScreen`; title `Difficulty`, six difficulty radios, four source-instantiated risk radios, generated Close | Fifth LDL-only risk row removed. |
| Map size picker | `SPNewGameMapSizeScreen`; `Small`, `Regular`, `Huge`, `Gigantic`, generated Close | Geometry/art mapped; text metrics approximate. |
| World shape picker | `SPNewGameMapShapeScreen`; `Earth World`, `Doughnut World`, `Flat World` plus description labels, generated Close | Geometry/art mapped; description wrapping/font approximate. |
| Players picker | `SPNewGamePlayersScreen`; numeric rows `3`..`8`, generated Close | Geometry/art mapped. |
| Rules picker | `SPNewGameRulesScreen`; `Bloodlust - Off`, `Pollution - Off`, generated Close | Controls map; switches now update the setup state in place and keep the popup open like the original. |

## Current outstanding style mismatches to fix next

1. Replace fallback/system text rendering with the original AUI font/color/shadow/text-blit behavior for `CTP2_STANDARD_FONT`, `CTP_GREY_TITLE_FONT`, and `CTP_GOLD_TITLE_FONT`.
2. Implement full `c3_PopupWindow` frame/title chrome, not just pattern fill + Close.
3. Implement AUI text fields and sliders precisely, including bevels, caret/selection, ticks, min/max labels, and pattern-backed tracks.
4. Promote the modeled resolution/music-track listbox chrome to a fuller renderer with header/ranger/scroller behavior and clipping.
5. Persist custom-map slider values into profile/server-backed setup state.
6. Connect in-place rule switch behavior to profile/server-backed state.
