# Manager runtime geometry and populated rows

Parent: `reference-interaction-fidelity.md`; component: `great-library-workflows.md`.

This checklist is implementation work, not permission to certify whole screenshots or skip the rest of the parent plan.

- [x] Cross-check original list instantiation, pane/header/ranger geometry, item templates, selection/highlight clipping and original research/production display calculations.
- [x] Reproduce incorrect list frame/content bounds and ranger placement with pure tests, including zero/negative/large dimensions and unchanged authoritative snapshots.
- [x] Implement source runtime list frame expansion without conflating frame, content and hit regions; keep scroll capacity/offsets consistent.
- [x] Render dynamic Build/Science/Research rows from their actual installed item templates, preserving numeric columns, independent selection, exact identities and unavailable values.
- [x] Verify projected field/unit semantics for every populated stat/forecast before displaying it; absent facts must not become invented zeroes/turn counts.
- [x] Add independent native pixel/clip regressions and production semantic/pointer scrolling/selection workflows using only memory and mocked HTTP.
- [x] Inspect updated native/reference-size populated Build/Science/Library captures and record unresolved differences honestly.
- [x] Run lint-fix, strict lint, full tests, >=80% line coverage and a dedicated build; report environment-only validation blockers precisely.
- [x] Update parent/component plans, PLAN.md, FIXES.txt and snapshot documentation, review restricted/server boundaries, and prepare a normal checkpoint commit after pre-commit lint-fix. The main plan remains the completion target.

Evidence: `ctp2_ListBox::DoneInstantiatingThis`, `aui_ListBox::RepositionRangers`, installed `eq_UnitItem`/`eq_BuildingWonderItem`/`eq_QueueListItem`/`ScienceAdvanceListItem`/`SciListItem`, `EditQueue::AddChoiceItem`, `ScienceManagementDialog::UpdateAdvanceItem` and ColorBox/ColorHeader callbacks. Science displays owned discoveries rather than the available research picker; both lists deduplicate exact IDs and use projected costs for ordering. Foreign discoveries are not projected and must not be inferred from embassy identities alone.

Do not preserve source border-overfill: page capacity uses the content pane, not its expanded border. Zero-sized panes cannot create phantom rows and zero-sized SDL controls are rejected before SDL coerces them to one pixel. Selection/child glyph/pattern clips are bounded to real row content.

Verified current Go schema: omitted unit `ranged`/`firepower` scalar fields mean zero in an existing runtime definition; missing definitions or malformed values remain unavailable. Movement has no declared universal 100-point scale. `CityReport` contains identities/buildings/effects, not complete yields, and research projection has no authoritative rate. Production/research turns therefore remain unavailable rather than reconstructing hidden economy.

Validation: 419 library + 437 binary + 19 integration = 875 executions; 83.15% LLVM line coverage (39,007 lines; 6,572 missed). Strict lint/lint-fix, native build and in-memory native frame/arrow/knowledge/clip tests pass. Fast coverage script remains absent; Windows MSVC stops in GLib/GTK cross-compiling pkg-config setup before client compilation. Ignored native/reference-size `runtime-{build-populated,science-populated,library}-{native,reference}.png` were inspected. Fonts, picture framing/content, advisor/selector controls and other full-reference differences remain. No reference acceptance is certified.

After this component, continue the unchecked items in `reference-interaction-fidelity.md`; commits are checkpoints, not a stopping condition.
