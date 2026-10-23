//! Original Save-name form and immutable local-template capture; no filesystem I/O.
use super::super::{ManagementUi, ManagerIntent, confirmation};
use super::{Template, codec, custom, valid_name};
use crate::gameplay::ui_model::operations::OperationDialog;
#[cfg(test)]
mod tests;

/// Stable original field path shared by drawing, native and accessible input.
pub const FIELD: &str = "MessageBoxDialog.DialogBackground.TextField";
/// Original affirmative is visually right, despite its source name.
pub const OK: &str = "MessageBoxDialog.DialogBackground.LeftButton";
/// Original negative is visually left.
pub const CANCEL: &str = "MessageBoxDialog.DialogBackground.RightButton";
/// Original GetFieldText(text,255) bound, applied without cutting UTF-8 sequences.
pub const MAX_NAME: usize = 255_usize;

/// Captured local work, independent of canonical gameplay revisions and permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// Exact authoring lifetime.
    pub epoch: u64,
    /// Exact local version when Save was opened.
    pub version: u64,
    /// Complete ordered original grammar data, never canonical queue entry IDs.
    pub template: Template,
    /// Approved literal basename; Some means one deferred create request, not success.
    pub name: Option<String>,
}
impl Capture {
    /// Receipt-safe local identity; remote ticks don't authorize or invalidate a template.
    #[must_use]
    pub fn current(&self, ui: &ManagementUi) -> bool {
        ui.view.as_ref().is_some_and(custom::active)
            && ui.saved_queues.custom.draft.as_ref().is_some_and(|draft| {
                draft.epoch == self.epoch
                    && draft.version == self.version
                    && custom::template(draft).as_ref() == Some(&self.template)
            })
    }
}
/*
 * Original ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::SaveButton ->
 * MessageBoxDialog.cpp::{TextQuery,constructor,LeftButtonActionCallback,
 * RightButtonActionCallback,TextFieldActionCallback} -> SaveNameResponse/Save.
 * messagebox.ldl::MessageBoxDialog is320x174 centered, TextField(30,92,260x24),
 * MessageDisplay(30,30,260x70), str_code_SaveQueueAsColon and MB_OK/MB_CANCEL.
 * TextQuery shows the field and hides DontShowButton. Preserve the source's
 * seven-pixel field/button vertical overlap and tail-to-head painting/hits; no
 * screenshot-fit/scaling offsets. Client explicitly focuses the Unicode field.
 * Capture owned template/lifetime/name instead of source static saveFileName and
 * mutable m_customBuildList callback data. Do not truncate255-byte C buffers,
 * ignore fopen/write failures, swap CAP/INF or leave Custom before real success.
 * This create-only workflow never overwrites an existing file or installation;
 * destructive overwrite/delete require their separate versioned confirmations.
 */
/// Creates actual cached source `TextQuery`, not a fake server command/form.
#[must_use]
pub fn dialog(ui: &ManagementUi) -> Option<OperationDialog> {
    ui.view.as_ref().filter(|view| custom::active(view))?;
    let draft = ui.saved_queues.custom.draft.as_ref()?;
    let template = custom::template(draft)?;
    codec::encode(&template).ok()?;
    let capture = Capture {
        epoch: draft.epoch,
        version: draft.version,
        template,
        name: None,
    };
    let title = ui.book.label("str_code_SaveQueueAsColon");
    if title == "str_code_SaveQueueAsColon" {
        return None;
    }
    let widgets = confirmation::controls(
        ui,
        title,
        ManagerIntent::Select("queue-save".to_owned(), "approve".to_owned()),
        true,
    )?;
    if ![FIELD, OK, CANCEL]
        .iter()
        .all(|id| widgets.iter().any(|widget| widget.id == *id))
    {
        return None;
    }
    Some(OperationDialog::queue_name(title, widgets, capture))
}
/// Bounded flat literal name; user whitespace/percent are not format directives.
#[must_use]
pub fn name_valid(name: &str) -> bool {
    name.len() <= MAX_NAME && valid_name(name)
}
