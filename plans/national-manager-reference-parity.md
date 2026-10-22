# National Manager reference continuation

Parent: [full-screen-reference-parity.md](full-screen-reference-parity.md). Client-only, TDD, no copied/edited original assets/source, no server changes, no test filesystem writes/live requests. This slice does not certify any complete screenshot.

## Source/authority audit

- [x] Inspect `national-manager-opened.png`, installed `citystatus.ldl`, original `NationalManagementDialog.cpp`, public current citystatus/NationalManagementDialog source, and server v2 `CityProjection`, `RulesProjection`, `projectCity`, `Yield` and existing forecast/governor contracts.
- [x] Identify reference revision differences: three tabs (Resources/Status/Specialists), eleven resource headers, ten specialist columns, narrower lists, four bottom actions; old installed layout has two tabs/nine resource columns and no Specialists.
- [x] Preserve original meanings: gross food/production, net city gold, applied science/pollution, crime percent, conversion gold and franchise loss are not interchangeable with raw `city.yields` or production forecast rate. Worker/slave/combat counts cannot be invented from total population or all colocated units. Current schema directly supplies population, happiness/crime/rioting, specialist map, governor, queue and revision-bound forecast/quote.

## Red–green implementation

- [x] Write source-geometry/header/tab tests first; capture RED; restore three-tab intrinsic art from installed template, reference rectangles and original icon IDs without copied art. Missing upic26/upic27 is detected read-only with explicit semantic/missing-art fallback, not concealed parity.
- [x] Write exact row-column/value tests first (own-only, source column identities, absent/null/zero/legacy/stale data); capture RED; cache source item templates and render independent columns rather than names-only lists.
- [x] Reuse actual received population/happiness/crime/specialists/governor/queue/forecast facts. Leave unavailable effective-resource/worker/slave/combat/conversion/franchise accounting unavailable with provenance; never run CityWindow::Project/local economy simulation.
- [x] Test exact selected-city Build/City return lifecycle, mayor toggle/priority, current affordable rush quote, legal confirmation and stale/foreign ownership guards. Bind only current supported operations; no first-city targeting or batch-command impersonation.
- [x] Add original source resource/status/specialist sorting/selection/focus where supported, preserving city identity across tabs and dropping removed/captured targets on receipt. Keep absent authoritative states distinct from zero.
- [x] Native control/art/header/list readback, accessible interaction tests and accurate isolated gateway envelope tests. Tests hold frames in memory and only read installed resources.
- [x] Add explicit no-city National reference diagnostic without Build's unrelated city/queue; inspect native/reference-size output and complete-frame report. Record failure rather than accepting any unfinished or blocked area.
- [x] Update parent/PLAN/FIXES/gap report with actual evidence and exact remaining contracts.

Evidence: semantic RED for missing source tabs/columns; quote/control RED for stale-quote rush and missing X/priority captions; sorting RED; empty queue versus unavailable RED; canonical riot color RED; real native optional-art RED (`upic26.rim` missing), then injected present/absent art GREEN and actual Resources/Specialists frame switch GREEN. Accurate gateway checks use server governors.go's true-priority versus disabled payload field sets and exact rush city_id payload. An early navigation fixture lacked authenticated owner remapping; corrected it to the actual account/session projection mapping before accepting the end-to-end check. No network call occurs in local navigation tests.

Native/reference diagnostics were inspected; complete-frame origin (5,21), 112223/307200 pixels exceed channel delta 8, raw RGB error 9866586. This is **not accepted parity**. Missing original Times faces, missing upic26/upic27, capture/geometry/typography differences and unavailable effective accounting stay explicit. The parent reference-18 checkbox remains unchecked.

## Verification/commit

- [x] Strict `cargo make lint`, complete tests and dedicated native build; fix every failure.
- [x] Run absent fast coverage script honestly; enforce >=80% using LLVM full-target coverage. All Rust files <800 lines (split before crossing).
- [x] Review privacy, exact authority/revision identities, no per-frame file IO, native clipping and cross-platform implications. Report missing original Times fonts/Windows target/sysroot; do not claim unavailable validation.
- [x] Run `cargo make lint-fix`, stage only intended client files and commit with normal unchanged hooks. No bypass.

Final validation: 566 library + 584 binary + 19 integration + 1 example = 1170 executions; 85.09% LLVM lines with >=80% enforced. Strict lint, full standard tests and dedicated native build pass. Fast script is absent (127). Fresh Windows GNU check fails before client compilation: target core missing plus GLib/GObject pkg-config cross environment absent; no Windows success is claimed. All Rust files are below 800 (largest: runtime.rs 799, management.rs 798, actions.rs 797, National 762). Server remains clean at 15ada0164180b8e6b8c280029ba55ef5cac88168. Stage/commit normally after lint-fix; fix any hook failure before calling the commit complete.

After this slice, continue **all** remaining workstreams and the 33 still-unaccepted complete frames in the parent plan. National component tests are not complete-game/whole-screen acceptance.
