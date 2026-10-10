# Full Game Parity Correction Plan

Branch: `singleplayer-in-game`

## Non-negotiable correction

- [x] Replace placeholder gameplay graphics with real CTP2 terrain/UI/unit assets where decoded in this pass; remaining city/goods/improvement art stays tracked below.
- [x] Treat the local/remote server as the gameplay authority; no silent local-only gameplay when a server is required.
- [x] Ask the user to start the local server for interactive verification and surface a clear in-client error if it is unavailable.
- [x] Keep all original assets referenced/decoded from the installation only; never copy assets into this repository.

## Server-authoritative launch

- [x] Add client-side server configuration, defaulting to `OPEN_CTP_SERVER_URL` or `http://localhost:9000`.
- [x] Add a tested HTTP client for `/health`, setup endpoints, gameplay commands, and `/game/{id}/events` resync.
- [x] On Launch, check server health before gameplay; if unavailable, fail with a clear message asking to start `$CTPSERVER`.
- [x] Add bootstrap flow for local server-managed single-human game: account/session, create game, select country, add AI, start game, fetch projection.
- [x] Convert server visibility projections into renderable client state.
- [ ] Route all actions through server commands: movement, settling, sentinel/fortify, and end turn now submit to the server; production, diplomacy, and full options mutation remain follow-up.

## Actual CTP2 rendering

- [x] Decode `gtset565.til`/`gtset555.til` base tiles from `ctp2_data/default/graphics/tiles`.
- [x] Render map with original CTP2 isometric tile dimensions (`94x48`, `24px` headroom) and camera math.
- [x] Load in-game UI RIMs from `pic565.zfs`/`pic555.zfs`, including `uptb00.rim`, `upfg01.rim`, `upfg03.rim`, `upfg04.rim`, and `upcb14*.rim`.
- [x] Render top menu/control-panel/end-turn using original RIM textures, not colored rectangles.
- [x] Render unit SPR frames onto the isometric map using decoded original `.SPR` frame data; facing/animation advancement remains follow-up.
- [ ] Render city/goods/ruin/improvement sprites from original assets.
- [x] Add fog/explored overlays using the client visibility projection/fog state.

## Verification

- [x] Tests for tile decoding from synthetic `.til` bytes and optional local CTP2 tile-set decoding.
- [x] Tests for RIM UI lookup using pure fixture helpers and optional local CTP2 RIM decoding.
- [x] Tests for server health/bootstrap request shapes using an in-process fake HTTP server.
- [x] Tests for original settler `.SPR` decode when local CTP2 assets exist.
- [x] Headless SDL render smoke remains deterministic without network/file mutation.
- [x] `cargo make lint-fix`
- [x] `cargo make lint`
- [x] `cargo make test`
- [x] `cargo build`
- [ ] Coverage script check: `./scripts/check-coverage-fast.sh` is absent in this checkout.
- [ ] Commit normally.
