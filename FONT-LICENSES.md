# Client font policy

## Original appearance

The supplied original-game images under `images/ctp2_references/` (Build, Science,
Graphics and Library) show Times-style serif text: mostly bold controls/body text,
with regular serif titles and headers. The authoritative installed English
`ctp_template.ldl` explicitly selects:

| Original template/face | Common open alternative |
| --- | --- |
| `CTP2_STANDARD_FONT`: `timesbd.ttf` (Times New Roman Bold) | Liberation Serif Bold |
| `CTP2_HEADER_FONT`, `CTP_GREY_TITLE_FONT`: `times.ttf` | Liberation Serif Regular |
| `controlpanel.ldl::ICON_VALUE_PAIR.Value`: Arial Bold10 stats | Liberation Sans Bold |
| `civ3.ldl::TiledMapFontStringTable`: Arial Bold9 map labels | Liberation Sans Bold |

Regular, bold, italic and bold-italic remain distinct. Source-backed gameplay
controls retain their existing per-control sizes and measurements. Shell controls
now resolve their inherited face, rather than giving every title the button font;
existing shell sizes/scaling are unchanged. Map city/population/production labels
now use the original localized Arial Bold9 table, and Unit numeric values use their
inherited Arial Bold10 template instead of bypassing it with generic Times. Client-only
GTK launcher, settings,
entries and dialogs use the same Times/Liberation Serif family stack while
retaining platform accessibility sizes/weights and script fallback. OS titlebars
and external system dialogs remain OS-controlled; video artwork is not re-typeset.
No full-screen or pixel-identical reference acceptance is implied. Original
`aui_TextField::InitCommonLdl/InitCommon` accidentally passes TTF filenames to
Win32 `EnumFonts` (which expects family names) and reads `fontname` instead of
`fontheight`. The client honors the intended declared face rather than copying
that accidental stock-font fallback. Original path-turn glyphs use GDI primitives;
client destination-only annotations use the portable map family, without claiming
GDI raster equivalence or inventing path turns.

## Licensing research and decision

Primary sources checked for this change:

1. [Microsoft Font redistribution FAQ](https://learn.microsoft.com/en-us/typography/fonts/font-faq):
   “A Windows application can use the fonts to render content to a screen”.
   However, document embedding permissions **do not authorize embedding fonts in
   games/apps**. Microsoft also prohibits redistributing/converting Windows font
   files without the appropriate separate permission. The FAQ expressly says fonts
   supplied by other apps require reviewing those apps' licenses.
2. [Liberation upstream](https://github.com/liberationfonts/liberation-fonts) identifies
   Liberation Serif/Sans as metric-compatible alternatives to Times New Roman/Arial.
   Its [current license](https://github.com/liberationfonts/liberation-fonts/blob/main/LICENSE)
   is **SIL Open Font License 1.1**. It permits use, embedding and redistribution
   subject to its conditions (including preserving notices/license and reserved-name
   rules). “Metric-compatible” does not mean identical glyphs or rasterization.

**Decision:** ship no font files, extracted glyph atlases, or copied game assets.
Use properly licensed system-installed Times New Roman/Arial if present; otherwise
prefer the familiar, widely packaged Liberation families. Same-family/style
DejaVu, then GNU FreeFont, are last-resort system-installed alternatives, never an
unrelated bold sans face for all missing fonts. Discovery itself cannot certify a
font's license or make proprietary fonts public domain.

The Windows FAQ is **not** evidence that GOG/retail game-bundled font licenses allow
use by this separate client on Linux. Consequently the client does **not** read
those game-only fonts by default. Only if you independently have the necessary
license/permission, opt in with `CTP_USE_GAME_FONTS=1`; then discovery checks the
selected installation's English/default `uidata/fonts` before system fonts, as the
original `ctp2_source/ctp2_code/ctp/civ3_main.cpp::ui_Initialize` did. This opt-in is
an acknowledgement, not a license grant or automatic license verification.

If fonts are bundled in a future release, use an appropriately licensed open font
and include its exact corresponding license/notices; the client's Unlicense does
not replace a third-party font license. Do not copy Windows/game font files into
this repository or a release based on their document-embedding flags.

## Availability

Windows normally supplies Times New Roman and Arial. Linux users should install
Liberation from their distribution, for example:

- Debian/Ubuntu: `sudo apt install fonts-liberation2` (or `fonts-liberation`, per distro).
- Fedora: `sudo dnf install liberation-serif-fonts liberation-sans-fonts`.

No runtime download or privileged font installation is performed. User-installed
fonts, Windows' actual `WINDIR/Fonts`, common Debian/Fedora directories and standard
macOS desktop font directories are supported. Platform discovery tests are
fileless, not proof of a native Windows/macOS build. Old minimal systems with no
usable fonts still require installing fonts/SDL_ttf; emergency diagnostics are not
an appearance-equivalent font substitute.
