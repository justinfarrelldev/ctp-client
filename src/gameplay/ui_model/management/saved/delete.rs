//! Original explicit Delete queue question; profile namespace and raw version only.
use super::super::{ManagementUi, ManagerIntent, confirmation};
use super::{active, prompt, save};
use crate::gameplay::ui_model::operations::OperationDialog;

/// One source Delete authorization, never a canonical game command or raw path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// Exact catalog namespace+basename; installed sources are never mutable.
    pub key: String,
    /// Exact portable literal profile basename, not another case/content match.
    pub name: String,
    /// Complete raw bytes (including comments/BOM), not a normalized template hash.
    pub expected: Vec<u8>,
    /// Explicit affirmative queued for one effect, not evidence of deletion.
    pub approved: bool,
}
impl Capture {
    /// Revalidates original visible file/selection/cache without rebasing its contents.
    #[must_use]
    pub fn current(&self, ui: &ManagementUi) -> bool {
        let Some(view) = ui.view.as_ref().filter(|view| active(view)) else {
            return false;
        };
        let Some(file)=ui.saved_queues.files.get(&self.key) else{return false;};
        if file.id!=self.key {return false;}
        if file.name!=self.name {return false;}
        if file.bytes.as_ref()!=Some(&self.expected) {return false;}
        let folded=self.name.to_lowercase();
        save::name_valid(&self.name)
            && self.expected.len()<=super::codec::MAX_BYTES
            && self.key == format!("profile:{}", self.name)
            && view.selections.get("saved-queue") == Some(&self.key)
            && ui.saved_queues.error.is_none()
            && !view.selections.contains_key(super::REFRESH)
            && ui
                .saved_queues
                .files
                .values()
                .filter(|candidate| {
                    candidate.id.starts_with("profile:")
                        && candidate.name.to_lowercase() == folded
                })
                .count()
                == 1_usize
    }
}
/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{DeleteCallback,
 * GetSelectedQueueName,DeleteQueryCallback} captures selected filename and calls
 * MessageBoxDialog::Query("QueryDeleteQueue"), then remove/UpdateFileLists only
 * on explicit approval. editqueue.ldl::LoadBox.DeleteButton and messagebox.ldl's
 *320x174 source OK/Cancel/fonts/DontShow controls supply styling/geometry.
 * str_ldl_EditQueueReallyDelete is missing in the bundled English installation;
 * use exactly the original C++ fallback, not a different game's/community string.
 * Client captures owned key/name/raw version, revalidates cache + backend bytes,
 * handles remove errors, forbids installed-source changes and destructive
 * Don'tShow auto-approval. Byte equality is a named-profile content version,
 * not an OS inode/unique historical identity or hostile-filesystem global CAS.
 * No neighboring/first file selection, private game state or local queue change.
 */
/// Original source Query with no implicit affirmative and no parameter field.
#[must_use]
pub fn dialog(ui: &ManagementUi) -> Option<OperationDialog> {
    let view = ui.view.as_ref().filter(|view| active(view))?;
    let key = view.selections.get("saved-queue")?;
    let file = ui.saved_queues.files.get(key)?;
    let capture = Capture {
        key: key.clone(),
        name: file.name.clone(),
        expected: file.bytes.clone()?,
        approved: false,
    };
    if !capture.current(ui) {
        return None;
    }
    let id = "str_ldl_EditQueueReallyDelete";
    let localized = ui.book.label(id);
    let format = if localized == id {
        "Permanently delete Queue %s?"
    } else {
        localized
    };
    let message = prompt(format, &[&capture.name])?;
    let widgets = confirmation::controls(
        ui,
        &message,
        ManagerIntent::Select("queue-delete".to_owned(), "approve".to_owned()),
        false,
    )?;
    Some(OperationDialog::queue_delete(&message, widgets, capture))
}
