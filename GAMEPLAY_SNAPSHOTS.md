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

Tests include in-memory semantic interactions, exact native artwork placement, reticle masks, both pixel formats, all retail terrain-pair/rule combinations, river/mouth masks, resource visibility/fog, malformed data, scaled framebuffers and PNG round trips. Installed art is read-only. Server tests use mock HTTP/SSE responses, not a running local server. The requested `./scripts/check-coverage-fast.sh` is absent; `cargo llvm-cov` provides the available line-coverage measurement instead. Windows source portability is reviewed, but Windows binaries require validation on a configured Windows/SDL build host.
