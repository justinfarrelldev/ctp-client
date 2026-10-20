# CTP2 gameplay rendering and visual checks

The gameplay renderer composes an **800×600 logical frame**, then uniformly scales the complete frame into the existing SDL window. At 640×480 this reproduces the reference's 80%-sized LDL controls and 94×72 native terrain art. Text and thin borders are scaled with the frame rather than separately rounded. The map centers normally; the accidental pan in the reference is not reproduced. The main-menu renderer is unchanged.

## Offline screenshots

These commands invoke the actual gameplay renderer with deterministic, explicitly synthetic data. They never construct a server client, change settings, or launch the intro. The example is an **explicit file-writing diagnostic**, not a test.

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/gameplay.png 640 480

SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/terrain.png 640 480 none terrain

SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/options.png 640 480 Options terrain
```

Replace `ctp2` with the installed game root. Use `800 600` for native-pixel inspection, or a larger 4:3 size to inspect whole-frame scaling. On Windows, omit the shell environment assignments; run with a normal video driver, or set the two variables using the shell's syntax. Optional menu titles are `Empire`, `Cities`, `Units`, `Diplomacy`, `Stats`, `Options`, or `none`.

**The `terrain` scene is a deliberately hand-built visual fixture.** It mixes biome edges, water, river connections, visible/explored/unknown cells, and a selected stack in one small area. Its visibility is not calculated from the settler's position and is not evidence about server sightlines. Its visibility layout has intentionally been left unchanged. The default scene instead uses the existing deterministic local preview generator. Neither scene is a playable or authoritative game.

## Directional motion inspection

The opt-in diagnostic also accepts `motion DIRECTION QUARTER`: clockwise screen directions 0–7 (north through northwest) and phases 0–4 (origin through arrival). It uses an explicitly synthetic, accepted single-unit path and the production animation renderer; no live server or settings are involved.

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/motion-nw.png 800 600 none motion 7 2
```

Tests invoke the same capture API entirely in memory. Additional SDL readback tests compare all eight directions at four phases for Settler/Warrior/Hoplite/Archer art against independently selected retail SPR frames, including western mirroring/hotspots, sequence indices, deltas and alpha. Cursor tests compare the original hand and all four edge cursors pixel-by-pixel, with exact LDL hotspots and transparent black. The software pointer is included in live F12 captures and uniformly scaled with the frame. OS cursor visibility is restored when gameplay returns or fails.

Movement and frame timing share one elapsed-time clock. Only the accepted prefix of a submitted path is presented; rejected commands do not animate, and network wait time does not consume the animation. The shell's unit speed/animation toggle is handed off without rereading settings. Edge scrolling uses original thresholds, vertical corner priority, the source default 300ms delay, 50ms acceleration and a bounded time-step; losing focus or opening an existing modal cancels it.

**This is not full unit/system parity.** Art now resolves all 72 installed retail unit records using `newsprite.txt`, with separate Hoplite/Archer portraits/icons/SPR groups and source fortification overlay 34. Unknown canonical types do not impersonate Warriors; the legacy three-family gameplay model is not a replacement for server capabilities. All catalogued actions/facings decode, but attack/death playback, end-turn actor events, wrap/boundary camera behavior, moving-stack hit testing and full UI workflow parity remain unfinished. Cost-aware path labels do not establish retained multi-turn order support. The `motion` fixture alone does not prove these features. See [the full reference inventory and remaining checklist](plans/reference-interaction-fidelity.md).

## Context, stack and manager diagnostics

Replace the last scene argument with `context`, `stack` or `info` for the corresponding component. Original-layout manager names are `build`, `science`, `research`, `empire`, `national`, `city`, `unit`, `army`, `trade`, `diplomacy`, `library`, `statistics`, `graphics` and `sound`:

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/manager-build.png 800 600 none build

SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/manager-science.png 640 480 none science
```

These scenes still use a synthetic map; manager bodies are not live game fixtures. The `library` diagnostic explicitly opens the installed Ballistics reference article; this does not invent a current research target in the game model. The manager engine reads installed LDL/templates/localization and composes original nine-piece frames, preserving native corners, source positions and image-derived dimensions. It shares control descriptors with pointer/keyboard and accessible ID activation. Native tests render fourteen roots and check source corner pixels, but this is **not** a fourteen-screen pixel-parity certification.

Implemented management workflows include independently selecting buildables/queue entries, Add, Rush Buy and explicit available-research selection. Isolated mock HTTP tests verify exact route/entity/metadata, session/revision fields and receipt/selection handoff; they do not contact a running server. Founding uses the server's recommended unique city ID/name and opens that accepted city's Build Manager once. Other original controls remain unavailable until integrated; there are no cancel/re-add queue workarounds that silently destroy production.

Library categories, explicit global search, internal hyperlinks, Gameplay/Historical text, Back/Forward and return to the selected Build window now use installed source articles. Source markup is non-executable; only verified expressions resolve against the viewer's current rule definitions. Unsupported expressions show an em dash. The dependency chart now uses current projected prerequisites and research-state borders: neighbors recenter locally, center opens its article, and more than four neighbors are paginated. Missing runtime metadata disables the chart; installed retail rules are never substituted. Set Goal remains disabled because persistent goal support is distinct from immediate Change Research.

Native tests independently compare Ballistics Library/Science and selected Settler Build artwork at source coordinates, hyperlink underlines and search-field pixels. They also cover clipping and portrait scaling regressions. Source-created age sorting and original ranger track/thumb/arrows now support actual category/range interactions. Query editing includes Unicode caret/selection/word deletion, undo/redo, injected clipboard shortcuts, pointer placement and IME composition. Source font styles and measured glyph/word regions share one session-owned font set; original LDL first-template/child precedence, tail-to-head painting, source shadows/right justification and long-title overhang have regressions. Explicit font fallback still prevents claiming exact text pixels.

For populated (still synthetic) public-shaped city/research data, use `build-populated`, `science-populated`, `library-populated`, or `chart`. Existing `build`/`science` previews remain empty rather than inventing missing authoritative facts:

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/build-populated.png 800 600 none build-populated
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/chart.png 640 480 none chart
```

Inspected `images/{library,build-populated,science-populated,chart}-reconciled-{native,reference}.png` at 800×600 and 640×480. Article requirements/installed picture framing, exact fonts, manager columns/progress/advisors and other runtime layout behavior still differ from the reference. Runtime list frame expansion/content/header/ranger placement now follows `ctp2_ListBox::DoneInstantiatingThis`, with source item templates and real Build stat columns. Science shows viewer-known discoveries rather than available research targets, with original own-player color blocks. Selector/dropdown/advisor lifecycle and remaining full-screen details are still unfinished. Do not assume all picture differences are asset-version issues. These are diagnostic captures, not accepted reference scenarios.

The inspected context/stack/info and manager captures demonstrate real SDL output, not complete reference matches. Full advisor panes, remaining runtime-created controls/data columns, option bindings, exact fonts and Apolyton-specific layout additions remain tracked work. Public community layouts are under `ctp2_data/default/uidata/layouts`, not `english`; locating them does not mean the deltas are implemented.

## Live capture

Press **F12** during gameplay to capture the newly composed framebuffer before presentation. The default destination is `images/gameplay.png`; set `CTP_GAMEPLAY_CAPTURE` to choose another PNG path. An explicit capture replaces that destination and creates its parent directory if necessary. Normal rendering does not write screenshots.

`FrameCapture.rgba` and `FrameCapture::png_bytes` retain pixels/PNG data in memory. `save_png` is the separate, opt-in filesystem sink. Tests exercise rendering and PNG round trips in memory and never call that sink. `/images/` is ignored; do not commit reference images, screenshots, or extracted game art.

## Original references and implementation

- `controlpanel.ldl` / `ctp_template.ldl`: layered panel, eight wheel shortcuts, overlapping tabs, unit portrait/stat boxes, spinner, 4×3 order grid, End Turn and radar frame.
- `ctp2_Menu.cpp`, `ctp2_MenuButton.cpp`, `aui_bitmapfont.cpp`, `primitives.cpp`: dropdown sizing/shortcuts, font metrics and quantized C3 bevels. Live and preview rendering share font size/hinting. Per-control Times/Arial face/point/bold/italic metadata selects session-owned fonts using source 96-DPI sizing; compatible Liberation variants are explicit, warned fallbacks. This host has no discovered Times TTFs, so no exact font rasterization certification is possible.
- `tileset.cpp` / `tiledraw.cpp`: ordered neighborhood transforms, four independent transition streams, textured `*99` fallback, terrain hats, shadow runs and connected river/mouth overlays. Complete original composition is used, not resized base-tile substitutes.
- `pixelutils.h`: explored terrain/resource shading uses the original 16/32 packed-channel blend with black, before RGB555/RGB565 channel expansion. Alpha silhouettes and shadow opacity remain intact; fog does not cover neighboring tile rectangles.
- `UnitActor.cpp` / `spritefile.cpp` / `goodactor.cpp`: native sprite canvas/hotspots, idle/default facing, shields/flags, original four selection brackets and resource actors. Original resource IDs map to installed artwork, never to invented deposit locations.

Terrain textures are keyed by visual composition and fog variant, not coordinates; the cache is bounded to 512 entries. Unknown authoritative neighbors are excluded from transition/coast selection. Empty projections remain empty/unknown, and disabling a local fog preference never fabricates unknown server geography. Stack commands and owner mapping use unit identity rather than shared tile position.

Accessible panel labels and pointer activation share the same rectangles/intents (`runtime/controls.rs`). Invisible Unit-tab orders cannot activate through another tab. Supported game orders use the existing authoritative dispatcher. Unsupported orders are disabled with reasons rather than pretending to be executable or changing the local world. Single/expanded unit health uses authoritative ratios and original color thresholds, not a fabricated full green bar. Distinct Sentinel/Sleep is never silently replaced by Fortify.

## Scope and remaining differences

This is an original-art fidelity pass, **not certification that every CTP2 screen is pixel-identical or that the full game is finished**. Radar projection/viewport presentation and the two lower radar glyphs still need exact reference reconciliation. Some unit statistics, non-Unit panel bodies, manager/library dialogs and order workflows remain partial. Save/Load and unintegrated menu workflows remain disabled where appropriate. Font installations and platform rasterizers can affect exact text pixels.

Live river/resource/border parity also requires authoritative data. The server currently omits rivers, resource IDs and territorial ownership from viewer tiles/fog memory; generated-world handoff also lacks rivers/resources. The client accepts optional `river` and `good_id`, including actual resource SPR rendering and remembered shading, but never guesses their locations when absent. Territorial ownership rendering remains unintegrated. The source-verified report and suggested visibility-safe contracts/tests are in `$CTPSERVER/client-discovered-server-issues.md` (documentation commit `5ed6237`), which also preserves a link to the earlier singular-named report. Additional findings cover missing city names, advertised-but-unregistered routes and unsafe/unsupported queue editing. No server implementation was modified. Unimplemented client windows and bindings for already-supported operations are not server defects.

## Validation

```bash
cargo make lint
cargo make test
cargo build
cargo llvm-cov --all-features --all-targets --summary-only
```

Latest runtime list/row checkpoint validation: `cargo make lint-fix`, strict lint, 419 library tests, 437 binary tests, 19 integration tests and the dedicated native build pass (875 executions; binary/library suites share tests). LLVM line coverage is **83.15%**. The Windows x86_64 MSVC check was repeated and stops before client compilation in GTK/GLib native dependencies (GLib/GIO/GObject/Cairo/Pango/GdkPixbuf/Graphene): pkg-config is not configured for cross-compilation/sysroots. No Windows binary validation is claimed.

Tests include in-memory semantic interactions, exact native artwork placement, reticle masks, both pixel formats, all retail terrain-pair/rule combinations, river/mouth masks, resource visibility/fog, malformed data, scaled framebuffers and PNG round trips. Installed art is read-only. Server tests use mock HTTP/SSE responses, not a running local server. The requested `./scripts/check-coverage-fast.sh` is absent; `cargo llvm-cov` provides the available line-coverage measurement instead. Windows source portability is reviewed, but Windows binaries require validation on a configured Windows/SDL build host.
