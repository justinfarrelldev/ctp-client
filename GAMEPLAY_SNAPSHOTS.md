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

Movement and frame timing share one elapsed-time clock. A provisional first step may be presented while a command is pending, but canonical units never change optimistically; rejection/recovery rolls it back. Confirmed animation follows only a submitted/retained path and its authoritative endpoint. Queued hybrid intent is not completed movement, and network wait time does not consume accepted animation. The shell's unit speed/animation toggle is handed off without rereading settings. Edge scrolling uses original thresholds, vertical corner priority, the source default 300ms delay, 50ms acceleration and a bounded time-step; losing focus or opening an existing modal cancels it.

**This is not full unit/system parity.** Art now resolves all 72 installed retail unit records using `newsprite.txt`, with separate Hoplite/Archer portraits/icons/SPR groups and source fortification overlay 34. Unknown canonical types do not impersonate Warriors; the legacy three-family gameplay model is not a replacement for server capabilities. All catalogued actions/facings decode, but attack/death playback, wrapping, moving-stack hit testing and full UI workflow parity remain unfinished. Contract-v2 journeys now set/cancel server-retained destinations, restore on reconnect and animate confirmed continuations; non-wrapped public bounds constrain camera/input without revealing cells. These have separate protocol/interaction tests: the synthetic `motion` fixture alone proves neither retained orders nor whole-game parity. See [the full reference inventory and remaining checklist](plans/reference-interaction-fidelity.md).

## Context, stack and manager diagnostics

Replace the last scene argument with `context`, `stack` or `info` for the corresponding component. Original-layout manager names are `build`, `science`, `research`, `empire`, `national`, `city`, `unit`, `army`, `trade`, `diplomacy`, `library`, `statistics`, `graphics` and `sound`:

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/manager-build.png 800 600 none build

SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/manager-science.png 640 480 none science
```

These scenes still use a synthetic map; manager bodies are not live game fixtures. The `library` diagnostic explicitly opens the installed Ballistics reference article; this does not invent a current research target in the game model. The manager engine reads installed LDL/templates/localization and composes original nine-piece frames, preserving native corners, source positions and image-derived dimensions. It shares control descriptors with pointer/keyboard and accessible ID activation. Native tests render fourteen roots and check source corner pixels, but this is **not** a fourteen-screen pixel-parity certification.

Implemented management workflows include independently selecting content/stable queue entries, atomic Add/Insert/Remove/Up/Down/Clear/custom replacement, Rush Buy, explicit research/goals, city rename/disband, production governors and original policy/rate controls. Isolated mock HTTP tests verify exact route/entity/metadata, session/revision fields and receipt/selection handoff; they do not contact a running server. Founding uses the server's recommended unique city ID/name and opens that accepted city's Build Manager once. Other original controls remain unavailable until integrated; there are no cancel/re-add queue workarounds that silently destroy production.

Library categories, explicit global search, internal hyperlinks, Gameplay/Historical text, Back/Forward and return to the selected Build window now use installed source articles. Source markup is non-executable; only verified expressions resolve against the viewer's current rule definitions. Unsupported expressions show an em dash. The dependency chart now uses current projected prerequisites and research-state borders: neighbors recenter locally, center opens its article, and more than four neighbors are paginated. Missing runtime metadata disables the chart; installed retail rules are never substituted. Set Goal now binds the advertised persistent-goal actions, including explicit clearing; it remains distinct from immediate Change Research.

Native tests independently compare Ballistics Library/Science and selected Settler Build artwork at source coordinates, hyperlink underlines and search-field pixels. They also cover clipping and portrait scaling regressions. Source-created age sorting and original ranger track/thumb/arrows now support actual category/range interactions. Query editing includes Unicode caret/selection/word deletion, undo/redo, injected clipboard shortcuts, pointer placement and IME composition. Source font styles and measured glyph/word regions share one session-owned font set; original LDL first-template/child precedence, tail-to-head painting, source shadows/right justification and long-title overhang have regressions. Explicit font fallback still prevents claiming exact text pixels.

For populated (still synthetic) public-shaped city/research data, use `build-populated`, `science-populated`, `library-populated`, or `chart`. Use `city-chooser` for a separate two-city projection with its source selector open. Existing `build`/`science` previews remain empty rather than inventing missing authoritative facts:

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/build-populated.png 800 600 none build-populated
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/chart.png 640 480 none chart
```

Inspected `images/{library,build-populated,science-populated,chart}-reconciled-{native,reference}.png` at 800×600 and 640×480. Article requirements/installed picture framing, exact fonts, manager columns/progress/advisors and other runtime layout behavior still differ from the reference. Runtime list frame expansion/content/header/ranger placement now follows `ctp2_ListBox::DoneInstantiatingThis`, with source item templates and real Build stat columns. Science shows viewer-known discoveries rather than available research targets, with original own-player color blocks. Build/City selectors now have source single-caption/button/detached-popup behavior, five-part artwork, exact owned identities and bounded keyboard/pointer/scroll/cancellation. Advisor lifecycle and remaining full-screen controls/art/font details are still unfinished. Do not assume all picture differences are asset-version issues. These are diagnostic captures, not accepted reference scenarios.

Use `empire-populated` for a separate explicit current-policy fixture (science50, PW10, workday1, wages-1, rations0); other populated/empty fixtures stay unchanged. Source social slider thumbs/grooves and percentage arrows/display share native/accessible value geometry. Capture only previews a value; release/keyboard submits once with the original displayed revision and untouched companion policies. Receipt values drive the display even when they differ from the request. Policy tests use pure fixtures, memory-only SDL and isolated HTTP stubs, never live optimizer probes.

Inspected `images/policy-empire-{native,reference}.png`. The retail root/columns/spinner positions do not yet match Apolyton's full-screen reference; overlapping summary/detail labels and absent installed text keys still require lifecycle/layout reconciliation. Independent installed thumb/groove pixel tests are component evidence, not acceptance of that screen. Government comparison/enactment and unavailable economic/physical-policy values remain tracked work.

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

The independently implemented server contract v2 now supplies viewer-safe river/resource/ownership observations and inspection. The client consumes these cached facts, preserving legacy omission as unknown and stale records as remembered, rather than consulting canonical maps or guessing deposits/territory. Political edges require explicit observed neighboring ownership, never nearby-city inference. Historical issue reports document earlier omissions; they are not the current server contract. Architectural culture/style, optimizer and unsupported barbarian-ruin profiles remain unavailable. This integration modifies no server implementation. Unimplemented client windows and bindings for already-supported operations remain client work.

## UI-first continuation captures

Additional explicit lower-panel scenes are `panel-empire`, `panel-city`, `panel-city-empty`, `panel-msgs` and `panel-tile`. They use the same cached installed controls and source fonts as live gameplay. Populated values are explicitly synthetic viewer data; the empty City scene has no city/portrait/name. No test writes captures. For example:

```bash
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  cargo run --example gameplay-snapshot -- ctp2 images/panel-city.png 640 480 none panel-city
```

Native/reference-sized manager and reference-sized panel captures were inspected during the UI pass. Empire now uses the reference's full-height columns and relocated original policy spinners; Graphics has twelve switches/two native sliders, and Sound retains gold corners and centered thumbs. Lower Empire/City/Msgs/Tile bodies now render original progress pictures, selectors, parchment/ranger and five palette radios. Palette switching is view-only; Farm artwork does not assert affordability or authorize an improvement. Missing city economy, research turns, pollution, advisor results and notifications are not fabricated.

Native tests reproduce short three-piece caps losing their bottom pixels and verify declared-height fitting, icon-layer offsets, progress pictures, option ticks/thumbs and packed-channel column bevels. Separate in-memory tests read the supplied Graphics/Sound references with independently measured window-decoration crops, compare only 14×14 gold-corner masks with capture-quantization tolerance, and reject a displaced frame. These masks do **not** certify complete screen equality. Remaining advisor/manager details, typography, message/city/map workflows and all 33 full reference acceptances remain open in the existing plan.

The reported original `Units.txt` launch failure is fixed in the **client**: unit/resource/sound readers now accept original Windows-1252 as well as modern UTF-8. No installation files were changed or converted. Fileless injected-reader tests exercise all unit-art inputs, original punctuation, Unicode mods and contextual I/O failures.

## Reported city/path/cursor repair

The blue city rectangle is replaced with original GC sprites, source hotspots and remembered-fog variants. The installed city recipe catalogue is read-only; observed age/capital and private production forecasts now drive presentation, while unprovided architectural culture/style use explicit default styling, never inferred canonical culture. Native SDL tests compare full building pixels against an independently loaded GC028 idle frame, reject fresh foreign city metadata on stale terrain, and verify projected caption precedence. `images/city-building-repaired.png` is an inspected opt-in diagnostic, not a copied asset or complete-reference acceptance.

Path lines now cross ground centres rather than the bottom of the headroom-inclusive tile canvas (the old error was 24 native pixels), use thin strokes and scale turn boxes with map zoom. A real SDL event test reproduced exact pointer450 truncating to449 at 1.2x. Native-frame presentation and mouse translation now share one rounded integer viewport transform, including high DPI and letterboxing; OS visibility is reasserted each frame. The user confirmed their screenshot's bottom crop was not a game defect; no viewport crop compensation was introduced.

Current contract integration is tracked in [client-contract-v2.md](plans/client-contract-v2.md). Names/age/capital, river/good/owner/yields, phase-aware End Turn and asynchronous viewer synchronization are now integrated. Bounded semantic results drive Msgs/read state, gold-frame ruin/production messages and known-location eye navigation; the exact confirmed gold text is `Exploration reward: 250 gold.` for a 250-gold result. Previously received calendar dates are cached, never extrapolated for older notices. Statistics graphs use actual own completed-turn samples, never fabricated foreign ranks. No client-side ruin roll, governor simulation or local retained-route replay occurs. Native art/component/interaction tests are not acceptance of all 33 complete reference scenarios.

## Full-screen continuation diagnostics

`unit-populated` now uses a separate explicit two-settler, no-city, War-readiness snapshot, not Build's unrelated Hoplite/city fixture. Its known actors remain in the projection while the diagnostic camera examines an opaque part of the public map bounds; this is not hidden terrain generation or a live game. Both native/reference-sized `images/parity-unit-{native,reference}.png` were inspected during the continuation. Source item columns/small icons, three-position readiness, own counts and attached advisor now render; missing strategic advice/applied nonzero support is explicitly unavailable, not forged reference text.

Native inspection exposed real shared rendering/loading bugs: HyperTextBox is intrinsically left-wrapped, implicit total-value width cannot inherit an entire window, and `Strings.txt` imports the addon/tooltip databases omitted by the old loader. Advisor and Library title strings do exist in the installation. Canonical first-definition precedence and bounded read-only imports now replace the earlier incorrect missing-title assumption. This is not full 33-scenario screenshot acceptance; follow [the active continuation](plans/full-screen-reference-parity.md).

The `trade-populated` profile shares that explicit two-settler/no-city start. Native/reference-sized `images/parity-trade-{native,reference}.png` were inspected after restoring three source tabs and six advisor totals. No owned cities in a complete v2 view proves the installed no-opportunity advice and zero transit/piracy sums; populated-city sums stay unavailable without effective accounting. Source/destination/payload market IDs do not alias Export/Import route selections. Accurate server fixtures never invent a `rules.goods` resource-definition contract. Current Trade complete-frame comparison at independently menu-calibrated origin (3,21) still differs in 137010 of 307200 pixels above channel delta 8 (raw error 11983726); it remains unaccepted.

`national-populated` also uses the explicit complete-v2 no-city/two-settler start. The restored three-tab source revision now renders eleven resource columns and the independent Specialists pane. Native/reference outputs `images/parity-national-{native,reference}.png` were inspected. Both original archives lack newer `upic26`/`upic27`; native regression first reproduced the missing-image failure, then semantic Conv/Fran fallback headers preserve usability and explicit missing-art markers. No copyrighted art is copied or silently replaced with an unrelated image. Complete viewport comparison calibrates game origin (5,21): 112223 of 307200 pixels exceed channel delta 8 (raw RGB error 9866586). This is still an explicit failed/unaccepted full frame. Follow [National's detailed checklist](plans/national-manager-reference-parity.md), then all remaining parent workstreams.

### Read-only full-frame comparison

`SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo run --example gameplay-reference -- ctp2 images/ctp2_references/unit-manager-opened.png unit-populated 2 21 scan`

This example does not save images or talk to a server. It compares every 640x480 viewport pixel against an explicit original-image crop and reports dense 32x32 spatial cells (including partial edges), all raw RGB error and pixels exceeding declared capture quantization. Optional `scan` calibrates decoration only from the unchanged menu row, never a failing manager/map/panel; the final report still includes those regions. Alpha is not a visible screenshot color. There are no masks or automatic acceptance budgets. Current Unit output is explicitly not identical (137571 of 307200 pixels exceed channel delta 8); normalization, actual fixture content and incomplete original behaviors remain work, not evidence that the reference is accepted.

## Validation

```bash
cargo make lint
cargo make test
cargo build
cargo llvm-cov --all-features --all-targets --summary-only
```

Latest v2 checkpoint validation: 514 library tests, 532 binary tests, 19 integration tests and the dedicated native build pass (1065 executions; binary/library suites share tests). LLVM line coverage is **84.27%**, with the 80% threshold enforced. Windows GNU checking stops before client compilation: the target Rust standard library is missing and GTK/GLib/GObject pkg-config has no cross sysroot. No Windows binary validation is claimed. Strict lint/lint-fix and the native build pass; normal unchanged commit hooks verify the final slice. Evidence is recorded in the integration ledger.

Tests include in-memory semantic interactions, exact native artwork placement, reticle masks, both pixel formats, all retail terrain-pair/rule combinations, river/mouth masks, resource visibility/fog, malformed data, scaled framebuffers and PNG round trips. Installed art is read-only. Server tests use mock HTTP/SSE responses, not a running local server. The requested `./scripts/check-coverage-fast.sh` is absent; `cargo llvm-cov` provides the available line-coverage measurement instead. Windows source portability is reviewed, but Windows binaries require validation on a configured Windows/SDL build host.
