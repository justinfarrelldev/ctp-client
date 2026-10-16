# CTP2 gameplay screenshot fidelity

## Reference and scope

The supplied target is 640×480, with 80%-sized original LDL artwork (an 800×600 game frame). Compare at that reference size, then scale the complete logical frame uniformly to the reused application window. Do not change the main menu, copy installed game assets, or contact/change the server in tests.

## Checklist

- [x] Inspect both screenshots and current gameplay renderer.
- [x] Cross-check original LDL, menu layout, radar, tile/sprite placement, selection map icons, order buttons, and font behavior against the release source.
- [x] Write failing tests for frame geometry, hidden terrain, complete control-panel art, actor anchoring, radar placement, menu colors, and UI input hit regions.
- [x] Replace placeholder chrome with original layered bottom-right panel, shortcut wheel, tabs, unit portrait/stats, order grid, status bar, and bottom-left radar/filter controls.
- [x] Match menu/resource layout, text rendering, and dropdown behavior to original artwork.
- [x] Correct map scale/centering, unexplored black fog, unit frame/hotspot/stack shield, and original green selection icon rendering.
- [x] Provide opt-in, deterministic application snapshots (no live server) and interactive framebuffer capture; tests exercise buffers/sinks only, never write files.
- [x] Inspect generated snapshots at reference and larger resolutions; iterate against supplied reference and document remaining differences honestly.
- [x] Final re-run: `cargo make lint-fix`, strict `cargo make lint`, full `cargo make test` (300 library, 318 binary, 19 integration tests), `cargo build`, and LLVM coverage all pass. Line coverage is 80.24%; required fast coverage script was attempted but is absent.
- [x] Review correctness, server-authority preservation, Windows portability, file sizes (<800 lines for changed Rust files), no restricted edits, and no copyrighted assets staged. Native Linux build passes; no Windows binary validation is claimed.
- [x] Update `PLAN.md`, `FIXES.txt`, README and `GAMEPLAY_SNAPSHOTS.md`; complete pre-commit `cargo make lint-fix` and normal commit (no verification bypass).

## Red/green evidence and authority review

- Type-13 terrain shadow runs failed before implementing alpha shadows (RGB555 and RGB565).
- Stacked settlers reproduced wrong ownership/command targeting; identity-based matching and selection retention pass.
- An empty authoritative projection incorrectly generated local armies/geography; it now remains featureless and unknown.
- Disabling local fog exposed fabricated unknown terrain; authoritative unknowns now remain black.
- Hidden authoritative neighbors cannot influence transition/coast artwork.
- The independent native-pixel fog oracle failed on the old 135/255 tint; packed-channel 16/32 shading now passes at zoom 1–3.
- Original elephant SPR parsing failed on the unit-only reader; goods use their source one-action layout without a unit shield tail. Every defined retail resource sprite decodes read-only. Resource visibility/shading/cached-reveal checks use memory-only framebuffers.
- No-selection order cells incorrectly looked active; regression now requires all twelve original disabled backgrounds.
- Semantic button tests exercise movement activation, next-unit wrapping/reset, tabs and unsupported commands without world mutation. Hidden orders are not actionable.
- No tests call the screenshot file sink or a real server. Images are ignored. Only the explicitly authorized report was changed in the server repository.

## Honest remaining scope

See `GAMEPLAY_SNAPSHOTS.md`. The terrain fixture's intentionally artificial visibility is unchanged; it is not a settler-vision simulation. This pass is not certification of all-screen pixel parity. Radar viewport/map presentation and two lower toolbar glyphs still need reconciliation; several dialogs/non-Unit bodies/stats and order workflows remain incomplete. Live rivers/resources/borders need the server metadata described in `$CTPSERVER/client-discovered-server-issue.md`. No server implementation was changed. Windows binary validation needs a configured Windows build host.
