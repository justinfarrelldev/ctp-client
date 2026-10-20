//! Independent presentation selection; authority is revalidated at runtime dispatch.

use super::{ManagerView, selectors};

/// City/queue/buildable/chart state cannot overwrite unrelated independent choices.
pub(super) fn apply(view: &mut ManagerView, key: String, value: String) -> bool {
    if key == "city" {
        selectors::change_city(view, &value);
    }
    if key == "dropdown" && value.is_empty() {
        return selectors::dismiss(view);
    }
    if key == "chart-focus" {
        view.selections.remove("chart-prerequisites-page");
        view.selections.remove("chart-successors-page");
    }
    if matches!(key.as_str(), "buildable" | "queue") {
        view.selections
            .insert("description".to_owned(), value.clone());
    }
    view.selections.insert(key, value);
    true
}
