# Music Track Selection and Auto Repeat Fidelity

## Research Findings

- Original `MusicTrackScreen` list items are zero-based (`s_trackNum = i`) and `SoundManager::PickNextTrack` maps user selection to CD track `m_userTrack + 2`; therefore the first visible row, "A Fortiori", should play `Track02.ogg`, not `Track01.ogg`.
- Original `SoundManager::ProcessRedbook` advances when redbook playback stops by calling `PickNextTrack()` and then `StartMusic()` when `m_curTrack != -1`.
- Original `Auto Repeat` is not a per-track SDL-style loop for playlist/random playback:
  - Playlist style advances through tracks and only stops at the end when Auto Repeat is off.
  - Random style picks another random track when each track ends; Auto Repeat does not stop random advancement.
  - User-selected style repeats the chosen track only when Auto Repeat is on; otherwise it plays once.

## Checklist

- [x] Add tests for the A Fortiori mapping so picker row 1 resolves to `Track02.ogg`.
- [x] Add tests for original auto-repeat semantics, especially Random Order + Music On + Auto Repeat Off continuing after Amaethon.
- [x] Fix music file mapping from UI list row to original `Track02`..`Track11` files.
- [x] Add explicit playback-style/advance logic matching original playlist/random/user semantics.
- [x] Poll/advance music when SDL_mixer reports natural track completion.
- [x] Preserve immediate user-selected track playback from the music picker.
- [x] Run targeted tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
  - `./scripts/check-coverage-fast.sh` is not present/executable, so coverage could not be run.
- [x] Final review and commit.
