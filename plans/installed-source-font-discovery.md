# Original installation font discovery correction

Parent: [full-screen-reference-parity.md](full-screen-reference-parity.md), after urgent launch/sync recovery. Use only local original ctp2_source as behavioral authority; never community/Apolyton source. No font/art copying, system installs, game asset edits or filesystem-writing tests.

- [x] Read original civ3_main.cpp C3DIR_FONTS search-path order (selected-language, default, then Windows), civpaths.txt uidata/fonts and aui_bitmapfont.cpp resource lookup/96 DPI. Actual configured GOG installation contains eight original-named Times/Arial fonts; current client searches only system directories. Earlier blanket missing-font blocker was incorrect.
- [x] RED pure injected discovery: selected installation/language/default precede system fonts, exact regular/bold/italic names retained; explicit platform fallback remains available and marked in asset-free installs.
- [x] RED unsafe/absolute/traversal/control source names never escape authorized font directories or perform reads; missing exact faces preserve fallback warning/no false exactness.
- [x] Share one bounded discovery path between session management fonts and map/HUD body font; cache during startup/capture, never per frame. Preserve source point-size/hinting/style semantics, no arbitrary per-control resizing.
- [x] Keep live/snapshot callers on their actual selected installation; no hardcoded personal path and no reliance on another installation. In-memory tests inject probes; existing native face/measurement tests remain read-only.
- [x] Production-render actual GOG assets read-only at native/reference sizes, inspect/compare complete frames without masks; exact named face availability is not screenshot certification.
- [x] Update previously false font-blocker documentation with actual discovery limitation versus remaining rendering/capture differences. Keep historical evidence labeled historical.
- [x] Strict lint/full tests >=80% LLVM, dedicated native build, honest Windows/script outcome, review <800 modules/no assets/server edits; lint-fix and normal verified commit.

Verification: local original CivPaths.cpp::GetSpecificPath confirms TRUE means selected/localized directory and FALSE default; original civ3_main appends Windows fonts afterward. Pure tests cover five named faces, original search order, explicit nonexact bold-italic fallback, missing candidates and unsafe cross-platform names before any probe. Existing native per-style measurement/drawing pointer tests pass. Actual selected GOG production render uses installed fonts without copying files; ignored manual images/parity-unit-installed-fonts-reference.png inspected. Complete Unit frame remains failed: origin2,21;137592/307200 pixels exceed delta8;raw error13623603;identical:false. Correct discovery does NOT certify glyph/capture/layout/data parity.

Strict lint/full tests/dedicated build pass;593 library +611 binary +19 integration +1 example =1224 executions;85.34% LLVM lines, >=80% enforced. Fast script absent127; Windows sysroot/target limitation unchanged. Runtime root remains799 lines (single import edit); no asset/server changes. Run lint-fix/normal hooks, then continue next unchecked work immediately.

Then resume [Diplomacy original proposal/workflow continuation](diplomacy-manager-reference-parity.md), every remaining parent implementation and all33 complete-frame acceptances. Do not stop on this component fix.
