# Gameplay Viewport and Movement Visual Repair Plan

## Goal

Make the SDL gameplay runtime fill the menu-owned viewport with the same minimal-border scaling as the rest of the client, and make CTP2-style movement feedback visibly render: active-unit green reticle plus dragged path markers.

## Checklist

- [x] Add a failing runtime test proving gameplay configures the reused SDL canvas for scaled 1024x768 logical rendering instead of drawing in the physical top-left corner.
- [x] Add/adjust rendering tests so the active-unit reticle and planned path markers have visibly thick green geometry, not only a single-pixel line.
- [x] Configure the gameplay canvas logical size/viewport before event handling and rendering so mouse coordinates and visuals share gameplay coordinates in reused windows.
- [x] Strengthen reticle/path rendering to use visible CTP2-green markers that survive scaling and original terrain art.
- [x] Run targeted tests.
- [x] Run `cargo make lint-fix` before commit.
- [x] Run `cargo make lint`.
- [x] Run `cargo make test`.
- [x] Run `cargo build`.
- [x] Attempt `./scripts/check-coverage-fast.sh`; record if absent: script is missing in this checkout.
- [x] Run `git diff --check`.
- [x] Review file sizes/restricted files/no asset copying/no file-mutating tests.
- [x] Commit with a clear message.
