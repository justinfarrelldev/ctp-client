# ZFS Asset Notes

This document summarizes the purpose and expected hierarchy for the Call to Power archive files discovered in the bundled reference installations. Each archive path is expressed relative to the game root. Content categories are inferred from historical Call to Power asset usage and verified against the directory layout provided with the repository.

| Game Version | Archive Path | Description |
| --- | --- | --- |
| CTP1 | `ctp1/ctp_data/default/graphics/patterns/pat555.zfs` | Base RGB555 tile pattern atlas used for world terrain textures and UI backgrounds in non-localized builds. |
| CTP1 | `ctp1/ctp_data/default/graphics/patterns/pat565.zfs` | Base RGB565 tile pattern atlas providing higher fidelity terrain textures for 16-bit displays; complements `pat555.zfs`. |
| CTP1 | `ctp1/ctp_data/default/graphics/pictures/pic555.zfs` | Default-language sprite sheet (555 format) covering unit portraits, wonder imagery, and UI pictograms. |
| CTP1 | `ctp1/ctp_data/default/graphics/pictures/pic565.zfs` | Default-language sprite sheet (565 format) matching `pic555.zfs` assets for displays requiring 565 pixel layout. |
| CTP1 | `ctp1/ctp_data/default/sound/sound.zfs` | Default audio pack containing ambient effects, interface sounds, and generic speech referenced by scripts. |
| CTP1 | `ctp1/ctp_data/english/gamedata/gl/gl.zfs` | English localized game logic bundle with city lists, names, and other gameplay data required for localization. |
| CTP1 | `ctp1/ctp_data/english/graphics/patterns/pat555.zfs` | English-localized replacement pattern atlas (RGB555) containing language-specific variants, e.g., signage and interface labels. |
| CTP1 | `ctp1/ctp_data/english/graphics/patterns/pat565.zfs` | English-localized pattern atlas (RGB565) mirroring `pat555.zfs` in 565 layout. |
| CTP1 | `ctp1/ctp_data/english/graphics/pictures/pic555.zfs` | English-localized sprite sheet (RGB555) supplying text-bearing UI artwork. |
| CTP1 | `ctp1/ctp_data/english/graphics/pictures/pic565.zfs` | English-localized sprite sheet (RGB565) complementing `pic555.zfs`. |
| CTP1 | `ctp1/ctp_data/english/sound/sound.zfs` | English-localized audio assets such as narration, advisor clips, and voice lines. |
| CTP2 | `ctp2/ctp2_data/default/graphics/patterns/pat555.zfs` | Base Call to Power II pattern atlas (RGB555) for global textures and UI backgrounds. |
| CTP2 | `ctp2/ctp2_data/default/graphics/patterns/pat565.zfs` | RGB565 variant of the default pattern atlas optimized for 16-bit displays. |
| CTP2 | `ctp2/ctp2_data/default/graphics/pictures/pic555.zfs` | Default CTP2 sprite sheet (RGB555) with unit, wonder, and interface imagery. |
| CTP2 | `ctp2/ctp2_data/default/graphics/pictures/pic565.zfs` | RGB565 counterpart to the default sprite sheet. |
| CTP2 | `ctp2/ctp2_data/default/sound/sound.zfs` | Core CTP2 audio collection with ambient, interface, and generalized sound effects. |
| CTP2 | `ctp2/ctp2_data/english/graphics/patterns/pat555.zfs` | English-localized pattern atlas (RGB555) containing language-specific textures. |
| CTP2 | `ctp2/ctp2_data/english/graphics/patterns/pat565.zfs` | English-localized pattern atlas (RGB565). |
| CTP2 | `ctp2/ctp2_data/english/graphics/pictures/pic555.zfs` | English-localized sprite sheet (RGB555) providing UI art with localized text. |
| CTP2 | `ctp2/ctp2_data/english/graphics/pictures/pic565.zfs` | English-localized sprite sheet (RGB565). |
| CTP2 | `ctp2/ctp2_data/english/sound/sound.zfs` | English-language voice and interface sound pack. |

## Extraction Hierarchy Expectations

All archives preserve internal relative paths. During extraction the target directory should mirror the original `ctp?_data` layout so that downstream subsystems can reference assets without additional remapping. For example, `ctp2/ctp2_data/default/graphics/patterns/pat555.zfs` should unpack into `<target>/ctp2_data/default/graphics/patterns/`. Retaining this structure ensures compatibility with scripts and configuration files that expect assets in their canonical locations.
