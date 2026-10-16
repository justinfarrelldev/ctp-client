# CTP2 terrain pixel seam repair

## Goal and scope

Fix exposed background along flat terrain edges using the actual gameplay map, original tile decoding, SDL texture upload, camera projection, and pixel readback. Grassland, plains and ocean must meet on all four edges. Do not require mountain/forest overhang silhouettes to fill their bounding boxes. Keep server authority and menu behavior unchanged; never copy game assets or write files from new tests.

## Checklist

- [x] Trace terrain selection/decoding, camera geometry and rendering against original CTP2 source and installed assets (read-only).
- [x] RED: add deterministic application-generated-map pixel tests through the production SDL renderer, with diagnostics for terrain, edge, zoom and failing pixels.
  - [x] Cover grassland/plains/water, all four edge directions, native and supported zoom levels, and full visible ground interiors.
  - [x] Demonstrate failure for the previous rendering and detect deliberately removed edge pixels; exclude legitimate overhang transparency.
  - [x] Use only in-memory rendering/readback and read-only local art, with no live server, screenshots or golden copyrighted assets.
- [x] GREEN: implement source-grounded tile-local fog modulation and scaled-bounds culling.
- [x] Additional RED/GREEN: expose fallback rectangular-ground holes with the same oracle, then draw original diamond scanlines at the current zoom.
- [x] REFACTOR: isolate terrain rendering in `render/terrain.rs`; keep the mathematical pixel oracle independent of decoder scanlines; reuse production texture loading and drawing.
- [x] Record bug and correction in `FIXES.txt`; link results from `PLAN.md`.
- [x] Validate: focused tests, `cargo make lint`, `cargo make test`, dedicated `cargo build`, coverage script attempt and alternative measurement, `git diff --check`.
- [x] Final review: maintainability, performance, test side effects, file sizes, restricted paths, Linux/Windows portability.
- [x] Run `cargo make lint-fix` before the normal atomic commit; re-run strict lint and focused pixel tests, with all passing.

## Source references and diagnosis

- `ctp2_source/ctp2_code/gfx/tilesys/tileset.h`: 94x72 images, 94x48 ground, 24px headroom.
- `tileutils.cpp`, `tileutils_BorkifyTile`: raw diamond staircase scanlines.
- `tiledraw.cpp`, `DrawTransitionTile` and `DrawBlendedTile`: ground pixels are shaded within the terrain diamond, not across the whole image rectangle.
- Installed `gtset565.til`: base terrain already decodes to opaque full ground diamonds. The observed cut-offs are in composition/culling, not missing image width.

The old fog rectangle covered neighboring ground and transparent headroom. Modulating each terrain texture before its copy preserves its decoded alpha silhouette; visible cells explicitly reset modulation to prevent shared-texture tint leaks. This also removes an extra overlay draw per fogged terrain cell. Culling now uses the actual scaled destination rectangle, retaining partially offscreen tiles at zoom 2/3. Asset-free fallback terrain now draws a scaled diamond rather than a shrunken rectangle.

## Red/green evidence

First SDL regression run, before implementation:

- Native mixed fog: visible water edge at map `(22, 5)` incorrectly darkened to `[2, 6, 10, 255]`.
- Zoom 2 coverage: plains `(10, 3)` exposed the sentinel background at screen `(852, 0)`.
- Single-pixel negative control passed: the oracle rejected missing pixels independently for every one of the twelve flat-terrain/edge combinations.

Second cycle:

- Asset-free fallback at zoom 1 exposed background at screen `(1021, 0)` on water `(22, 5)`.
- Replacing fallback rectangles with original diamond scanlines made that same contract pass at all three zooms, including generated fog shading.

Final five pixel tests pass. They generate real `GameState::new` maps (seeds 42/88), retain generated terrain unchanged, suppress actors/ruins that could mask missing pixels, use launch-centered and map-border views, and check interiors plus every edge band. Fog tests compare the same artwork with/without shading, including unchanged generated launch visibility and deliberately mixed visibility. The mathematical pixel-center diamond oracle does not reuse decoder scanline bounds or depend on artwork colors. Missing CTP2 installation art is an explicit failure rather than a silent skip.

Run the focused suite with:

```sh
cargo test --lib gameplay::runtime::render::pixel_tests
```

The original installation must be available at `ctp2/` beneath the crate root. SDL uses a hidden dummy-driver software canvas and RGBA32 readback, with no GPU/display dependency. All rendering, negative controls and missing-art stubs are memory-only; no new test writes assets, screenshots, config or other files. SDL access is serialized and environment variables are restored on returned failures. No server is contacted. The implementation uses portable SDL APIs, with no platform-specific rendering changes; Windows runtime execution was not available on this Linux host.

## Validation

- `cargo make lint-fix`: passed before commit; strict lint and focused pixel tests also re-run successfully afterwards.
- `cargo make lint`: passed.
- `cargo make test`: passed (266 library, 284 binary, 19 integration tests).
- `cargo build`: passed.
- `git diff --check`: passed.
- `./scripts/check-coverage-fast.sh`: unavailable (not present; scripts are restricted/read-only).
- `cargo llvm-cov --all-targets --summary-only --fail-under-lines 80`: all tests passed; the repository-wide threshold failed at **76.56%**. Unrelated existing GTK/video/server-projection paths remain under-tested; this focused repair does not modify them or coverage/lint rules.
- Coverage for `render/terrain.rs`: **93.88% lines**, **86.36% regions**, **100% functions**; `textures.rs`: **91.94% lines**. The new terrain implementation exceeds the 80% target.

All touched Rust files are below 800 lines. Restricted paths and assets are unchanged. No `allow` attributes were added. Menu layout, server authority, movement and non-flat overhang artwork are unchanged.
