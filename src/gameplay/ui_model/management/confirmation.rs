//! Cached original MessageBoxDialog controls for an immutable explicit question.
use super::{ManagementUi, ManagerCommand, ManagerIntent, ManagerWidget, tree};
use crate::gameplay::{assets::layout::LayoutNode, model::GameState, ui_model::{MenuRect, operations::OperationDialog}};

/*
 * Original released-source style/position/behavior provenance:
 * - ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::ClearButton calls Query
 *   with str_ldl_ClearQueueQuery/QueryClearQueue; ClearMessageCallback acts only
 *   on affirmative response and preserves AlreadyBoughtFront via ClearAllButHead.
 * - ctp2_source/ctp2_code/ui/interface/MessageBoxDialog.{cpp,h}::Query,
 *   constructor and LeftButtonActionCallback/RightButtonActionCallback define
 *   strongly-modal OK/Cancel behavior, hidden TextField and message placement.
 *   Header defaults are str_ldl_MB_OK/str_ldl_MB_CANCEL, not guessed Yes/No.
 * - ctp2/ctp2_data/english/uidata/layouts/messagebox.ldl::MessageBoxDialog is
 *   320x174, centered, with uptg29a..i background; MessageDisplay(30,30,260x70),
 *   LeftButton(220,109,width80), RightButton(20,109,width80), DontShowButton.
 * Client differences: source layout/fonts are cached at startup; command and
 * revision captured once, never local CityData mutation or implicit approval.
 * Destructive Don'tShow auto-approval is intentionally unavailable, and no
 * affirmative keyboard focus is invented. Owned Rust strings replace original
 * m_identifier's new[]/scalar-delete mismatch. Existing scaling is unchanged.
 */

/// Builds source controls entirely from cached data; opening never submits.
pub(super) fn dialog(ui: &ManagementUi, state: &GameState, command: ManagerCommand) -> Option<OperationDialog> {
    let root = ui.confirmation_layout.as_ref()?;
    let width = root.integer("widthpix").and_then(|width| u32::try_from(width).ok())?;
    let height = root.integer("heightpix").and_then(|height| u32::try_from(height).ok())?;
    let bounds = MenuRect { x: 800_i32.saturating_sub(i32::try_from(width).ok()?).div_euclid(2), y:600_i32.saturating_sub(i32::try_from(height).ok()?).div_euclid(2), width, height };
    let mut widgets = Vec::new();
    append(ui, root, "MessageBoxDialog", bounds, &command, &mut widgets);
    OperationDialog::confirmation(state, ui.book.label("str_ldl_ClearQueueQuery"), widgets, command)
}

/// Tail-to-head source painting; a hidden text field is not a semantic target.
fn append(ui: &ManagementUi, node: &LayoutNode, path: &str, bounds: MenuRect, command: &ManagerCommand, widgets: &mut Vec<ManagerWidget>) {
    if node.name == "TextField" { return; }
    let (label, intent, unavailable) = match node.name.as_str() {
        "MessageDisplay" => (ui.book.label("str_ldl_ClearQueueQuery").to_owned(), None, None),
        "LeftButton" => (ui.book.label("str_ldl_MB_OK").to_owned(), Some(ManagerIntent::Submit(command.clone())), None),
        "RightButton" => (ui.book.label("str_ldl_MB_CANCEL").to_owned(), Some(ManagerIntent::Close), None),
        "DontShowButton" => (ui.book.label("str_ldl_DontShowAgain").to_owned(), None, Some("Destructive requests always require explicit current confirmation".to_owned())),
        _ => (String::new(), None, None),
    };
    let mut flat = node.clone();
    flat.children.clear();
    widgets.push(ManagerWidget { id:path.to_owned(), label, rect:bounds, node:flat, intent, unavailable, selected:false });
    for child in node.children.iter().rev() {
        let rect = tree::child_rect(child, bounds);
        append(ui, child, &format!("{path}.{}", child.name), rect, command, widgets);
    }
}
