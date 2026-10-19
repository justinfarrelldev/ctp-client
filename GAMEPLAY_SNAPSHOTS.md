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

Tests invoke the same capture API entirely in memory. Additional SDL readback tests compare all eight directions at four phases for the supported Settler/Warrior art against independently selected retail SPR frames, including western mirroring/hotspots, sequence indices, deltas and alpha. Cursor tests compare the original hand and all four edge cursors pixel-by-pixel, with exact LDL hotspots and transparent black. The software pointer is included in live F12 captures and uniformly scaled with the frame. OS cursor visibility is restored when gameplay returns or fails.

Movement and frame timing share one elapsed-time clock. Only the accepted prefix of a submitted path is presented; rejected commands do not animate, and network wait time does not consume the animation. The shell's unit speed/animation toggle is handed off without rereading settings. Edge scrolling uses original thresholds, vertical corner priority, the source default 300ms delay, 50ms acceleration and a bounded time-step; losing focus or opening an existing modal cancels it.

**This is not full unit/system parity.** The model still recognizes only Settler, Warrior and the Warrior-art barbarian family. Hoplite/Archer and other canonical unit art/type mappings, attack/death/fortification state, animation of moves occurring during end-turn, wrap/boundary camera behavior, moving-stack hit testing and full UI workflow parity remain unfinished. The `motion` fixture is not evidence that these features are supported. See [the full reference inventory and remaining checklist](plans/reference-interaction-fidelity.md).

## Live capture

Press **F12** during gameplay to capture the newly composed framebuffer before presentation. The default destination is `images/gameplay.png`; set `CTP_GAMEPLAY_CAPTURE` to choose another PNG path. An explicit capture replaces that destination and creates its parent directory if necessary. Normal rendering does not write screenshots.

`FrameCapture::read` and `FrameCapture::png_bytes` retain pixels/PNG data in memory. `save_png` is the separate, opt-in filesystem sink. Tests exercise rendering and PNG round trips in memory and never call that sink. `/images/` is ignored; do not commit reference images, screenshots, or extracted game art.

## Original references and implementation

- `controlpanel.ldl` / `ctp_template.ldl`: layered panel, eight wheel shortcuts, overlapping tabs, unit portrait/stat boxes, spinner, 4×3 order grid, End Turn and radar frame.
- `ctp2_Menu.cpp`, `ctp2_MenuButton.cpp`, `aui_bitmapfont.cpp`, `primitives.cpp`: dropdown sizing/shortcuts, font metrics and quantized C3 bevels. Live and preview rendering share font size/hinting. Installed Times New Roman Bold is preferred, with Liberation Serif Bold fallback; different installed fonts can still change rasterization.
- `tileset.cpp` / `tiledraw.cpp`: ordered neighborhood transforms, four independent transition streams, textured `*99` fallback, terrain hats, shadow runs and connected river/mouth overlays. Complete original composition is used, not resized base-tile substitutes.
- `pixelutils.h`: explored terrain/resource shading uses the original 16/32 packed-channel blend with black, before RGB555/RGB565 channel expansion. Alpha silhouettes and shadow opacity remain intact; fog does not cover neighboring tile rectangles.
- `UnitActor.cpp` / `spritefile.cpp` / `goodactor.cpp`: native sprite canvas/hotspots, idle/default facing, shields/flags, original four selection brackets and resource actors. Original resource IDs map to installed artwork, never to invented deposit locations.

Terrain textures are keyed by visual composition and fog variant, not coordinates; the cache is bounded to 512 entries. Unknown authoritative neighbors are excluded from transition/coast selection. Empty projections remain empty/unknown, and disabling a local fog preference never fabricates unknown server geography. Stack commands and owner mapping use unit identity rather than shared tile position.

Accessible panel labels and pointer activation share the same rectangles/intents (`runtime/controls.rs`). Invisible Unit-tab orders cannot activate through another tab. Supported game orders use the existing authoritative dispatcher. Unsupported orders produce status text rather than changing the local world.

## Scope and remaining differences

This is an original-art fidelity pass, **not certification that every CTP2 screen is pixel-identical or that the full game is finished**. Radar projection/viewport presentation and the two lower radar glyphs still need exact reference reconciliation. Some unit statistics, non-Unit panel bodies, manager/library dialogs and order workflows remain partial. Save/Load and unintegrated menu workflows remain disabled where appropriate. Font installations and platform rasterizers can affect exact text pixels.

Live river/resource/border parity also requires authoritative data. The server currently omits rivers, resource IDs and territorial ownership from viewer tiles/fog memory; generated-world handoff also lacks rivers/resources. The client accepts optional `river` and `good_id`, including actual resource SPR rendering and remembered shading, but never guesses their locations when absent. Territorial ownership rendering remains unintegrated. The source-verified report and suggested visibility-safe contracts/tests are in `$CTPSERVER/client-discovered-server-issue.md`; no server implementation was modified.

## Validation

```bash
cargo make lint
cargo make test
cargo build
cargo llvm-cov --all-features --all-targets --summary-only
```

Latest validation for the reference-interaction pass: strict lint, 326 library tests, 344 binary tests, 19 integration tests and the dedicated native build pass. LLVM line coverage is **80.88%**. The Windows x86_64 check was attempted but is blocked before client compilation by `gio-sys`/GLib pkg-config cross-compilation setup; no Windows binary validation is claimed.

Tests include in-memory semantic interactions, exact native artwork placement, reticle masks, both pixel formats, all retail terrain-pair/rule combinations, river/mouth masks, resource visibility/fog, malformed data, scaled framebuffers and PNG round trips. Installed art is read-only. Server tests use mock HTTP/SSE responses, not a running local server. The requested `./scripts/check-coverage-fast.sh` is absent; `cargo llvm-cov` provides the available line-coverage measurement instead. Windows source portability is reviewed, but Windows binaries require validation on a configured Windows/SDL build host.
