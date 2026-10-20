# Great Library workflow continuation

Parent: `plans/reference-interaction-fidelity.md`. This is a component work plan, not completion of the 33-reference request.

## Checklist

- [x] Parse installed article sections, safe internal links and non-executable variables; test malformed markup and Windows-1252 content without writing files.
- [x] Resolve active all-category `uniticon.txt` still pictures, icons and summary/statistics tags rather than obsolete advance art.
- [x] Implement bounded local article history and category/global-search models with pure tests.
- [x] Test native search focus, Unicode input handling, explicit Enter execution, categories, article/link navigation, Back/Forward, Gameplay/Historical tabs and close/return workflows through the production input dispatcher.
- [x] Clamp list/article scrolling to actual content, test immediate recovery after extreme scroll requests, and keep scrolling out of gameplay input.
- [x] Verify original Build/Science description binding and artwork; implemented dynamic numbers come from projected runtime definitions, with unknown expressions visibly unavailable.
- [ ] Complete remaining expression mappings only where the server field/unit semantics are verified; do not apply retail movement/fuel scales to synthetic or modified rules blindly.
- [x] Add independent in-memory SDL picture/hyperlink/edit-field/clip regressions for Library, Science and selected Build portraits.
- [x] Inspect native/reference-size Library diagnostic captures and record differences, without calling them accepted reference matches.
- [ ] Inspect updated populated Build/Science captures; reconcile per-control fonts, source runtime resizing, sort/index controls, scrollbars and full text-editor keyboard behavior.
- [ ] Implement the original advance dependency tree and its navigation from actual prerequisites. Its tab is explicitly disabled until integrated; this is unblocked client work, not a server issue.
  - Source located: `ui/aui_ctp2/chart.cpp` and `greatlibrary.ldl` `TechTree` template. Original chart uses 124x20 buttons in a 440x200 viewport, up to four prerequisites/follow-ups, research-state borders and connectors. Neighbor clicks recenter only the chart; the original center callback is empty. Avoid reproducing a fake-enabled no-op.
- [ ] Establish an authoritative persistent research-goal operation before enabling Set Goal; do not substitute immediate Change Research.
- [x] Run strict lint, full tests, >=80% line coverage and dedicated native build; document Windows environment blockers and review read-only asset/test boundaries. Results: 381 library + 399 binary + 19 integration executions; 82.09% LLVM line coverage. Windows MSVC remains blocked by GTK native-dependency cross-compilation pkg-config/sysroots; the requested fast coverage script is absent.
- [x] Update parent plan, `PLAN.md`, `FIXES.txt` and snapshot documentation; run `cargo make lint-fix` and prepare a normal verified commit.

## Source anchors and distinctions

- `greatlibrary.cpp`: constructor/Initialize and search-field focus near 1036/1168; `SetLibrary` near 1365; `HandleIndexButton` near 2054; `UpdateList` near 2161; `Search_Great_Library`/`Force_A_Search` near 2436/2520; Back/Forward near 2540.
- Search is explicitly executed and covers names/gameplay/history across databases. Merely displaying an article does not populate the index. Preserve this distinction and cache search results instead of rescanning all article text every frame.
- The original initialization occasionally inserts a random joke search query. Keep initialization deterministic and empty; do not reproduce that behavior. The installed layout also retains an obsolete negative-position SearchLabel and a missing uppercase title key; suppress the stray caption and use the installed localized Great Library title.
- Article expressions are data, not executable SLIC or filesystem paths. Only explicit supported mappings resolve against the current viewer's rules.
- `ScienceManagementDialog.cpp` binds both `CurrentResearch.AdvanceProgress.IconBorder.IconButton` and its radial child to research selection.
- `GreatLibrary`/`BuildEditorWindow`/`ScienceManagementDialog` source LDL paths and read-only installed pictures determine layout, not guessed coordinates.
- Static original Library descriptions are reference material, not proof that a server implements every described mechanic. No client-local optimization, research or queue mutation is permitted.

Once this component is finished, return to `plans/reference-interaction-fidelity.md` and complete its remaining manager/panel/animation and 33-scenario acceptance work.
