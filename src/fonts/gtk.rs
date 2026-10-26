//! Font-family policy for client-owned native launcher/settings/error windows.

/// User-priority CSS covers descendants, popovers and dialogs, not OS decorations.
/* Original authority: ctp2/ctp2_data/english/uidata/layouts/ctp_template.ldl::
 * CTP2_STANDARD_FONT and CTP_GREY_TITLE_FONT select the Times family; original
 * ui/aui_common/aui_textbase.cpp::InitCommonLdl consumes that family per control.
 * There is no retail GTK installation picker/settings counterpart. Applying the
 * same familiar serif family to these client-only windows is an explicit client
 * consistency decision, not a copied retail layout. Keep platform accessibility
 * sizes/weights and font fallback for scripts outside the Latin game fonts.
 * Microsoft font-faq permits installed-font rendering, not redistribution;
 * Liberation Serif (OFL-1.1) is the preferred open lookalike. No bytes are bundled.
 */
pub(super) const FAMILY_CSS: &str =
    "* { font-family: \"Times New Roman\", \"Liberation Serif\", \"DejaVu Serif\", serif; }";

/// Installs once before the native launch flow; failure retains readable system UI.
pub fn install() {
    if let Err(error) = gtk4::init() {
        log::warn!("Native font styling unavailable: {error}");
        return;
    }
    if let Some(display) = gtk4::gdk::Display::default() {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(FAMILY_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
