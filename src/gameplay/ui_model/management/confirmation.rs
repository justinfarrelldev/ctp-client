//! Cached original MessageBoxDialog controls for source Query and TextQuery.
use super::{ManagementUi, ManagerCommand, ManagerIntent, ManagerWidget, tree};
use crate::gameplay::{
    assets::layout::LayoutNode,
    model::GameState,
    ui_model::{MenuRect, operations::OperationDialog},
};

/*
 * Original released source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::
 * ClearButton/ClearMessageCallback, LoadCallback/LoadQueryCallback and SaveButton
 * call MessageBoxDialog.{cpp,h}::{Query,TextQuery,constructor,LeftButtonActionCallback,
 * RightButtonActionCallback,TextFieldActionCallback}. messagebox.ldl root320x174
 * centers uptg29a..i, MessageDisplay(30,30,260x70), LeftButton(220,109,width80),
 * RightButton(20,109,width80), TextField(30,92,260x24), DontShowButton. Defaults are
 * str_ldl_MB_OK/str_ldl_MB_CANCEL. TextQuery shows field/hides Don'tShow; Query
 * hides field and cannot destructively auto-approve via Don'tShow replay here.
 * Keep source font inheritance/tail-to-head paint/hits and current scaling.
 * The source TextField/button overlap is preserved, not fixed by screenshot offsets.
 * Commands and local template captures use owned immutable data, not original
 * m_identifier's new[]/scalar-delete mismatch, mutable filename or static buffers.
 */
/// Source Clear question captures one validated exact current command.
pub(super) fn dialog(
    ui: &ManagementUi,
    state: &GameState,
    command: ManagerCommand,
) -> Option<OperationDialog> {
    question(ui, state, ui.book.label("str_ldl_ClearQueueQuery"), command)
}
/// Shared source Query; opening never submits or re-reads captured local files.
pub(super) fn question(
    ui: &ManagementUi,
    state: &GameState,
    message: &str,
    command: ManagerCommand,
) -> Option<OperationDialog> {
    let widgets = controls(ui, message, ManagerIntent::Submit(command.clone()), false)?;
    OperationDialog::confirmation(state, message, widgets, command)
}
/// Original cached source descriptors; local `TextQuery` carries no fake server action.
pub(super) fn controls(
    ui: &ManagementUi,
    message: &str,
    accept: ManagerIntent,
    text: bool,
) -> Option<Vec<ManagerWidget>> {
    let root = ui.confirmation_layout.as_ref()?;
    let width = root
        .integer("widthpix")
        .and_then(|width| u32::try_from(width).ok())?;
    let height = root
        .integer("heightpix")
        .and_then(|height| u32::try_from(height).ok())?;
    let bounds = MenuRect {
        x: 800_i32
            .saturating_sub(i32::try_from(width).ok()?)
            .div_euclid(2),
        y: 600_i32
            .saturating_sub(i32::try_from(height).ok()?)
            .div_euclid(2),
        width,
        height,
    };
    let mut widgets = Vec::new();
    Presentation {
        ui,
        message,
        accept,
        text,
    }
    .append(root, "MessageBoxDialog", bounds, &mut widgets);
    Some(widgets)
}
/* Released ui/interface/MessageBoxDialog.cpp::{Information,constructor,
 * RightButtonActionCallback}, under ctp2_source/ctp2_code/, uses the same
 * messagebox.ldl root but hides LeftButton/TextField/DontShow and labels the
 * remaining RightButton with str_ldl_MB_OK. It is strongly modal; dismissal
 * acknowledges information only, never confirms/retries an operation.
 */
/// Startup failure information shares original cached controls, not a fake game command.
pub(in crate::gameplay) fn information(ui:&ManagementUi,message:&str)->Option<Vec<ManagerWidget>> {
    let mut widgets=controls(ui,message,ManagerIntent::Close,false)?;
    widgets.retain(|widget|!widget.id.split('.').any(|part|matches!(part,"LeftButton"|"TextField"|"DontShowButton")));
    for widget in &mut widgets {if widget.node.name=="RightButton" {ui.book.label("str_ldl_MB_OK").clone_into(&mut widget.label);widget.intent=Some(ManagerIntent::Close);}}
    Some(widgets)
}
/// Cohesive traversal context, not eight recursive independent parameters.
struct Presentation<'ui> {
    /// Cached source styles/strings.
    ui: &'ui ManagementUi,
    /// Captured source localized question.
    message: &'ui str,
    /// Exact command or local filename approval intent.
    accept: ManagerIntent,
    /// Original `TextQuery`, not destructive Don'tShow behavior.
    text: bool,
}
impl Presentation<'_> {
    /// Tail-to-head source controls; hidden elements are not semantic targets.
    fn append(
        &self,
        node: &LayoutNode,
        path: &str,
        bounds: MenuRect,
        widgets: &mut Vec<ManagerWidget>,
    ) {
        if (node.name == "TextField" && !self.text) || (node.name == "DontShowButton" && self.text)
        {
            return;
        }
        let (label, intent, unavailable) = match node.name.as_str() {
            "MessageDisplay" => (self.message.to_owned(), None, None),
            "LeftButton" => (
                self.ui.book.label("str_ldl_MB_OK").to_owned(),
                Some(self.accept.clone()),
                None,
            ),
            "RightButton" => (
                self.ui.book.label("str_ldl_MB_CANCEL").to_owned(),
                Some(ManagerIntent::Close),
                None,
            ),
            "TextField" => (
                String::new(),
                Some(ManagerIntent::Focus(path.to_owned())),
                None,
            ),
            "DontShowButton" => (
                self.ui.book.label("str_ldl_DontShowAgain").to_owned(),
                None,
                Some(
                    "Destructive requests always require explicit current confirmation".to_owned(),
                ),
            ),
            _ => (String::new(), None, None),
        };
        let mut flat = node.clone();
        flat.children.clear();
        widgets.push(ManagerWidget {
            id: path.to_owned(),
            label,
            rect: bounds,
            node: flat,
            intent,
            unavailable,
            selected: false,
        });
        for child in node.children.iter().rev() {
            let rect = tree::child_rect(child, bounds);
            self.append(child, &format!("{path}.{}", child.name), rect, widgets);
        }
    }
}
