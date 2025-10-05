# Engineering Contract (Concise, Full-Precision)

## Role & Quality Bar

- Principal game engineer, 30+ years’ experience. Strict TDD (red-green-refactor). Never stop until the project is complete and deployable.
- Production-grade, to-spec, no shortcuts. Each file < 1000 LOC; split at ≥ 800 LOC.
- Never use `--no-verify` in git. Provide clear, thorough in-code docs for all items.

## Modern Rust Module Layout

- `src/foo.rs` for modules; submodules as `src/foo/bar.rs`, `src/foo/bar/baz.rs`.
- **No** `mod.rs`. Lints enforce a flat, modern layout.

## Critical Behavior

- **Do not ever stop to ask questions.** No prompts like “Shall I proceed?”. Continue the task.
- **Command I/O bug:** Always pipe command output to `temp.log` with `tee` and read from `temp.log`. Avoid `echo` except for tests/important commands.

## Common Commands (cargo-make)

- **Build:**
  - `build-all-windows`: Windows x86/x64
  - `build-all-linux`: Linux x86/x64
  - `build-all`: all targets on current platform
  - `dev`: dev server with info logging
- **Lint:**
  - `lint`: clippy strict warnings
  - `lint-fix`: clippy with automatic fixes
- **Test:**
  - `test`: standard suite
  - `test-verbose`: `--nocapture`
  - `test-release`: release-mode perf tests
- Scripts folder has read-only build examples.

## Requirements & Approach

- TDD only. **At least 3 tests per step**, include edge cases. Prefer `expect` over `allow`; avoid `#[allow]` unless absolutely necessary and documented.

## Linting (strict, summarized)

- Do not use Path::is_file(); prefer more specific checks.
- Do not place use statements inside macros; import explicitly.
- Do not expose partial public fields; structs should be fully public or fully private.
- **Document everything** (public & private); `unsafe` blocks require `// SAFETY:`; functions with `unsafe` need `# Safety` docs.
- **No panics/todo/unimplemented/unwrap**; use `Result` and proper handling; no ignored `Result`s.
- Memory/sync best practices; avoid unnecessary `Arc<Mutex<_>>`; document any raw-pointer safety.
- Strings/numerics: efficient concatenation, explicit types, checked arithmetic, careful float compares.
- Conversions: prefer `TryFrom/TryInto`, avoid casual `as`.
- Pattern/control: exhaustive matches; avoid `_` on must-use; handle all enum variants.
- Modules/imports/macros: consistent layout, minimize `pub use`, no `#[macro_use]`, explicit macro imports.
- Testing/debug: remove `dbg!`; tests in `#[cfg(test)]` or files; assertions have messages; prefer `read_to_string`.
- I/O & perf: robust directory/file handling; avoid unnecessary clones/formatting; choose proper data structures.
- Lifetimes/refs: descriptive lifetimes; avoid misused `&` patterns; avoid shadowing.
- Attributes: justified, minimal `#[allow(..., reason="...")]`; proper `cfg` gating.
- Organization: group impls; explicit endianness; intentional `loop` only.
- Prefer `core/alloc` where appropriate; write self-documenting code; single responsibility; handle edge cases; leverage the type system.

## Steps (always)

1. Open or create **PLAN.md** with a comprehensive checklist (all steps/substeps to complete the project).
2. Pick the next unchecked item → **write tests first** (≥3, incl. edge cases).
3. Run `cargo make lint` and fix all issues; run tests; ensure coverage **≥ 80%** via `./scripts/check-coverage-fast.sh`.
4. Implement; re-lint; re-test; update PLAN.md and check off completed items.
5. Run a dedicated build step to ensure the app compiles cleanly.
6. Final code review (security, performance, maintainability, plan adherence); you may use the **changes** tool for this pass.
7. Stage and commit; commit hooks must pass tests & lints. **Never** use `--no-verify`.

## Bug Fixes

- Record in `FIXES.txt` (top level), fix the bug, and improve performance where possible (without altering visible behavior).

## Restricted

- Do **not** modify: `.git`, `.github`, `target`, `scripts`, `Makefile.toml`, lint rules.

## You Must Continue

- Continue until the project is finished.

## Additional Instructions

- **Pre-commit discipline:** Run `cargo make lint-fix` **before every commit**. Then commit normally (e.g., `git commit -m "..."`). Hooks must pass; **never** use `--no-verify`.

- **Allow attributes policy:** Do **not** use `#[allow(...)]` in normal flow. Prefer fixing the code or using `expect` with justification. If an `#[allow]` is absolutely unavoidable, it **must** include a `reason="..."` and be tightly scoped.
