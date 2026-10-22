# Strings.txt launch recovery, then resume the main plan

Parent: [full-screen-reference-parity.md](full-screen-reference-parity.md) and top-level PLAN.md. This is an urgent launch fix, not a replacement completion target. Continue the unchecked parent work after the verified commit. Original behavior/style authority is local ctp2_source and installed retail assets, **not Apolyton/community source**.

- [x] Trace runtime installation selection and compare the actual configured manifests read-only. Bundled retail Strings.txt contains ordered imports; configured ctp2_gog manifest additionally contains standalone DoNotImportDefaults. No assets copied or changed.
- [x] Check local original StrDB.cpp::ParseAStringEntry and Token.cpp::Next: quoted ID/value entries and inline imports. Original source has no DoNotImportDefaults token; do not attribute the installed compatibility marker to original behavior or port community engine changes.
- [x] RED injected manifest with standalone marker followed by several imports/definitions, empty strings, duplicate first-definition precedence and marker text as a quoted translation.
- [x] RED malformed trailing ID/import errors retain actual file/ID context; do not suppress parse/read errors to hide the launch failure.
- [x] Replace blind chunks-of-two with ordered entry cursor: consume the installed standalone compatibility marker without treating it as a label/value. The client already reads only the selected language directory and performs no implicit default-language merge, so this flag requires no fallback engine.
- [x] Verify injected tests and actual configured retail/GOG manifests/layout loading read-only; never require an optional personal installation in tests or contact the server.
- [x] Review path/cycle/depth/file bounds and no-I/O-in-render guarantees; preserve original paired values/import order and no asset rewrites.
- [x] Run strict lint/full tests, >=80% LLVM coverage (fast script missing), dedicated build, lint-fix and normal unchanged commit hooks; document exact environment limitations.
- [ ] Update FIXES.txt and main/parent plans and commit coherent launch recovery.

Verification: strict lint/full tests/native build pass;583 library +601 binary +19 integration +1 example =1204 executions;85.30% LLVM lines, >=80% enforced. Fast script still absent(127), Windows target/sysroot limitations unchanged. Actual configured GOG production-render smoke exits0 and user confirms launch now works. Original installation/server trees unchanged; no test writes files. Found original fonts in GOG's uidata/fonts; defer font discovery correction until new immediate P0 production/synchronization concerns are resolved.

Once complete, first execute [immediate live-play recovery](immediate-live-play-recovery.md), then resume [Diplomacy continuation](diplomacy-manager-reference-parity.md), then **every remaining unchecked item** in [full-screen-reference-parity.md](full-screen-reference-parity.md) and PLAN.md. Never check off blocked/unverified complete-frame acceptance as success.
