//! Source-style explicit input/target dialogs. Opening a form never submits it.
use serde_json::{Value,json};
use super::{MenuRect,management::ManagerCommand};
use super::library::editor::{TextEditor,ClipboardRequest,metrics::EditMetrics};
use crate::gameplay::model::GameState;
use crate::gameplay::server::interface::LegalAction;

/// Parameterized original operation requiring explicit local editing.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum InputKind {
    /// City name input, scoped to the selected canonical city.
    Rename,
    /// Complete custom queue: comma-separated content IDs or @existing-entry IDs.
    Queue,
}
/// An explicit projected target/priority choice with a captured revision.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Choice {
    /// Human-readable operation and viewer-known target.
    pub label:String,
    /// Exact authorized operation, not a first-candidate fallback.
    pub command:ManagerCommand,
}
/// Weakly-modal local form. The server remains responsible for every outcome.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct OperationDialog {
    /// Source-style popup title.
    pub title:String,
    /// Revision that authorized this form.
    pub revision:i64,
    /// Exact city for parameterized input.
    pub city_id:String,
    /// Optional bounded Unicode field.
    pub input:Option<InputKind>,
    /// Local field value, never canonical city/queue state.
    pub text:String,
    /// Shared Unicode selection/undo/IME behavior.
    pub editor:TextEditor,
    /// Native field pointer/caret measurements.
    pub metrics:EditMetrics,
    /// Explicit clipboard effect handled outside pure models/tests.
    pub clipboard:Option<ClipboardRequest>,
    /// Current authorized action/priority choices.
    pub choices:Vec<Choice>,
    /// First visible choice index, page-sized and bounded.
    pub start:usize,
    /// Explicit keyboard target selection.
    pub focused:usize,
}
impl OperationDialog {
    /// Original `NameTheCityPopup` uses a 400-wide popup and a 200x20 text field.
    /// The footer extends that source geometry for explicit accessible confirmation.
    #[must_use]
    pub const fn rect()->MenuRect { MenuRect {x:200,y:180,width:400,height:270} }
    /// Original 200x20 input, offset x100/y80 within the popup.
    #[must_use]
    pub const fn field()->MenuRect { MenuRect {x:300,y:260,width:200,height:20} }
    /// Source-sized explicit confirmation control.
    #[must_use]
    pub const fn accept()->MenuRect { MenuRect {x:430,y:410,width:70,height:24} }
    /// Source-sized explicit cancellation control.
    #[must_use]
    pub const fn cancel()->MenuRect { MenuRect {x:510,y:410,width:70,height:24} }
    /// Creates a form only when its exact city currently has the matching route.
    #[must_use]
    pub fn input(state:&GameState,city:&str,kind:InputKind)->Option<Self> {
        let viewer=state.viewer.as_ref()?;
        let route=match kind {InputKind::Rename=>"rename-city",InputKind::Queue=>"edit-production"};
        viewer.interface.legal_actions.iter().find(|action| action.route == route && action.entity_id == city)?;
        let source=viewer.cities.iter().find(|source| source.id == city && !source.stale)?;
        let text=match kind {
            InputKind::Rename=>source.facts["name"].as_str().unwrap_or("").to_owned(),
            InputKind::Queue=>source.facts["queue"].as_array().map_or_else(String::new,|queue| queue.iter().filter_map(|item| item["entry_id"].as_str()).map(|id| format!("@{id}")).collect::<Vec<_>>().join(", ")),
        };
        let mut dialog=Self::blank(match kind {InputKind::Rename=>"Rename City",InputKind::Queue=>"Custom Production Queue"},viewer.revision);
        city.clone_into(&mut dialog.city_id);
        dialog.input=Some(kind);
        dialog.editor.reset(&text);
        dialog.editor.select_all(&text);
        dialog.text=text;
        Some(dialog)
    }
    /// Builds a pinned governor focus chooser, including explicit disable.
    #[must_use]
    pub fn governor(state:&GameState,city:&str)->Option<Self> {
        let viewer=state.viewer.as_ref()?;
        let mut dialog=Self::blank("City Mayor — production priority",viewer.revision);
        let priorities=viewer.interface.rules.as_ref()?.get("governor_priorities")?.as_array()?;
        for priority in priorities.iter().filter_map(Value::as_str) {
            dialog.add(state,format!("Enable mayor: {priority}"),"set-governor",json!({"city_id":city,"enabled":true,"priority":priority}));
        }
        dialog.add(state,"Turn mayor OFF".to_owned(),"set-governor",json!({"city_id":city,"enabled":false}));
        (!dialog.choices.is_empty()).then_some(dialog)
    }
    /// Enumerates exact authorized targets; no guessed identity or hidden lookup.
    #[must_use]
    pub fn actions(state:&GameState,title:&str,actions:impl Iterator<Item=LegalAction>)->Option<Self> {
        let mut dialog=Self::blank(title,state.viewer.as_ref()?.revision);
        for action in actions.filter(LegalAction::http_supported) {
            let target=state.viewer.as_ref().and_then(|viewer| viewer.cities.iter().find(|city| city.id == action.target_id)).and_then(|city| city.facts["name"].as_str()).unwrap_or(&action.target_id);
            let label=if action.route == "set-research-goal" && action.payload["advance_id"].as_str() == Some("") { "Clear Research Goal".to_owned() } else if target.is_empty() { action.route.replace('-'," ") } else { format!("{} — {target}",action.route.replace('-'," ")) };
            dialog.add(state,label,&action.route,action.payload);
        }
        (!dialog.choices.is_empty()).then_some(dialog)
    }
    /// Creates one validated operation from the edited field at the captured revision.
    #[must_use]
    pub fn command(&self,state:&GameState)->Option<ManagerCommand> {
        let (route,payload)=match self.input? {
            InputKind::Rename=>("rename-city",json!({"city_id":self.city_id,"name":self.text.trim()})),
            InputKind::Queue=>{
                let entries=self.text.split(',').map(str::trim).filter(|part| !part.is_empty()).map(|part| part.strip_prefix('@').map_or_else(|| json!({"item_id":part}),|id| json!({"entry_id":id}))).collect::<Vec<_>>();
                ("edit-production",json!({"city_id":self.city_id,"entries":entries}))
            }
        };
        let command=ManagerCommand {revision:self.revision,route:route.to_owned(),payload};
        command.valid_for(state).then_some(command)
    }
    /// Accessible activation shares the exact native confirmation/choice payload.
    #[must_use]
    pub fn activate(&self,label:&str,state:&GameState)->Option<ManagerCommand> {
        if label.eq_ignore_ascii_case("OK") { return self.command(state); }
        self.choices.iter().find(|choice| choice.label.eq_ignore_ascii_case(label) && choice.command.valid_for(state)).map(|choice| choice.command.clone())
    }
    /// Choice geometry is identical for keyboard, accessibility and pointer input.
    #[must_use]
    pub fn row(&self,index:usize)->Option<MenuRect> {
        if index < self.start || index >= self.start.saturating_add(8) { return None; }
        self.choices.get(index)?;
        Some(MenuRect {x:220,y:225_i32.saturating_add(i32::try_from(index.saturating_sub(self.start)).ok()?.saturating_mul(20)),width:360,height:20})
    }
    /// Initializes presentation without reading any installation or live service.
    fn blank(title:&str,revision:i64)->Self {
        Self {title:title.to_owned(),revision,city_id:String::new(),input:None,text:String::new(),editor:TextEditor::default(),metrics:EditMetrics::default(),clipboard:None,choices:Vec::new(),start:0,focused:0}
    }
    /// Discards unsupported/unavailable parameter choices instead of enabling no-ops.
    fn add(&mut self,state:&GameState,label:String,route:&str,payload:Value) {
        let command=ManagerCommand {revision:self.revision,route:route.to_owned(),payload};
        if command.valid_for(state) { self.choices.push(Choice {label,command}); }
    }
}
