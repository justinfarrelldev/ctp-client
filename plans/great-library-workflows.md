# Great Library workflow continuation

Parent: `plans/reference-interaction-fidelity.md`. This is a component work plan, not completion of the 33-reference request.

## Checklist

- [x] Parse installed article sections, safe internal links and non-executable variables; test malformed markup and Windows-1252 content without writing files.
- [x] Resolve active all-category `uniticon.txt` still pictures, icons and summary/statistics tags rather than obsolete advance art.
- [x] Implement bounded local article history and category/global-search models with pure tests.
- [x] Test native search focus, Unicode input handling, explicit Enter execution, categories, article/link navigation, Back/Forward, Gameplay/Historical tabs and close/return workflows through the production input dispatcher.
- [x] Clamp list/article scrolling to actual content, test immediate recovery after extreme scroll requests, and keep scrolling out of gameplay input.
- [x] Verify original Build/Science description binding and artwork; implemented dynamic numbers come from projected runtime definitions, with unknown expressions visibly unavailable.
- [x] Add verified unit costs/stats, production effects/costs and government mappings, including original SLIC float scaling, checked effect sums and inverse wonder crime labels. Missing fields, movement/fuel scales and government ranks stay unavailable; never apply retail scales blindly.
- [x] Add independent in-memory SDL picture/hyperlink/edit-field/clip regressions for Library, Science and selected Build portraits.
- [x] Inspect native/reference-size Library diagnostic captures and record differences, without calling them accepted reference matches.
- [x] Inspect updated populated Build/Science and Library/chart captures at 800×600 and 640×480; record remaining differences, not accepted matches.
- [x] Implement source-created Sort by Age/index controls using current projected age associations; unknown ages have explicit headings, not invented age zero.
- [x] Replace arrow glyphs with original ranger track/thumb/arrows; implement bounded arrows/pages/dragging/keyboard values, capture cleanup and native arrow pixel tests.
- [x] Implement UTF-8-safe caret/selection/word movement, Delete/Backspace, undo/redo, injected clipboard shortcuts, bounded paste, pointer selection and separate IME composition. Failed/stale cuts, AltGr typing and focus loss cannot cause unintended clipboard/game commands.
- [x] Preserve source font identities/sizes and 96-DPI raster sizing; use one session-owned font set for actual painting and measured link/editor hit regions. Test original first-template-wins LDL precedence, inherited child defaults and tail-to-head painting. Honor right justification, title overhang and declared shadows.
- [x] Implement the original advance dependency chart from current projected prerequisites, research-state borders and connectors. Neighbors recenter only the chart; center opens Gameplay content rather than reproducing the empty source callback. Paginate overflow beyond four neighbors; disable missing metadata rather than substitute retail rules.
- [ ] Complete full Library/Build/Science screenshot acceptance: installed article content/rendered picture framing differ from the reference, Times fonts are absent (explicit Liberation fallback), and manager data columns/progress/advisor/runtime details still need reconciliation. Do not assume every art difference is a version mismatch rather than a remaining renderer issue. Runtime list frame/pane/header/ranger geometry and cached source Build/Science/Research item columns are now integrated (see `manager-runtime-geometry.md`). Selector/dropdown lifecycle, advisors and other full-runtime details still require reconciliation. These are not all server blockers.
  - Source: `ui/aui_ctp2/chart.cpp`, `ldl/ldl_data.cpp`, `aui_region.cpp`, `aui_textbase.cpp`, `aui_bitmapfont.cpp`; original chart 440×200, buttons 124×20. Public community `greatlibrary.ldl` adds regular header text, sort/index positioning and sizes absent from the installed older layout. Runtime metadata overrides do not modify installed assets.
- [ ] Establish an authoritative persistent research-goal operation before enabling Set Goal; do not substitute immediate Change Research.
- [x] Run strict lint, full tests, >=80% line coverage and dedicated native build; document Windows environment blockers and review read-only asset/test boundaries. Results: 381 library + 399 binary + 19 integration executions; 82.09% LLVM line coverage. Windows MSVC remains blocked by GTK native-dependency cross-compilation pkg-config/sysroots; the requested fast coverage script is absent.
- [x] Update parent plan, `PLAN.md`, `FIXES.txt` and snapshot documentation; run `cargo make lint-fix` and prepare a normal verified browsing commit.
- [x] Repeat strict lint/lint-fix, full tests, ≥80% coverage, native build, Windows/fast-script attempts and final source/authority/read-only review for the chart/editor/ranger/typography continuation. Results: 410 library + 428 binary + 19 integration = 857 executions; 82.91% LLVM line coverage. Windows still stops in GTK/GLib native cross-compilation setup; fast script remains absent.
- [x] Prepare the verified chart/editor/ranger/typography continuation for a normal commit; required pre-commit lint-fix passed and staged source/documentation paths were reviewed.

## Source anchors and distinctions

- `greatlibrary.cpp`: constructor/Initialize and search-field focus near 1036/1168; `SetLibrary` near 1365; `HandleIndexButton` near 2054; `UpdateList` near 2161; `Search_Great_Library`/`Force_A_Search` near 2436/2520; Back/Forward near 2540.
- Search is explicitly executed and covers names/gameplay/history across databases. Merely displaying an article does not populate the index. Preserve this distinction and cache search results instead of rescanning all article text every frame.
- The original initialization occasionally inserts a random joke search query. Keep initialization deterministic and empty; do not reproduce that behavior. The installed layout also retains an obsolete negative-position SearchLabel and a missing uppercase title key; suppress the stray caption and use the installed localized Great Library title.
- Article expressions are data, not executable SLIC or filesystem paths. Only explicit supported mappings resolve against the current viewer's rules.
- `ScienceManagementDialog.cpp` binds both `CurrentResearch.AdvanceProgress.IconBorder.IconButton` and its radial child to research selection.
- `GreatLibrary`/`BuildEditorWindow`/`ScienceManagementDialog` source LDL paths and read-only installed pictures determine layout, not guessed coordinates.
- Static original Library descriptions are reference material, not proof that a server implements every described mechanic. No client-local optimization, research or queue mutation is permitted.
- `Player::SetResearchGoal`/`StartResearchingAdvanceForGoal` persist `m_researchGoal` and select successive prerequisites across completed advances. The inspected server exposes immediate `set-research` only. This contract remains blocked under the no-server-implementation-changes instruction; it cannot truthfully be checked off by relabeling Change Research.
- Reviewed captures: `images/{library,build-populated,science-populated,chart}-reconciled-{native,reference}.png` (ignored). Build/Science now have explicit synthetic public-shaped projections; empty scenes stay empty. Source art framing/content, exact fonts, available-advance columns, queue counts and advisors differ from references. No scenario acceptance is certified.

Once this component is finished, return to `plans/reference-interaction-fidelity.md` and complete its remaining manager/panel/animation and 33-scenario acceptance work.
