//! Independent presentation selection; authority is revalidated at runtime dispatch.

use super::{ManagerView, selectors};

/// City/queue/buildable/chart state cannot overwrite unrelated independent choices.
pub(super) fn apply(view: &mut ManagerView, key: String, value: String) -> bool {
    if let Some(applied) = super::build::select(view, &key, &value) { return applied; }
    if key == "trade-filter" {
        view.selections.remove("trade-market");
        view.scroll.remove(super::trade::MARKET);
    }
    if key == "city" {
        selectors::change_city(view, &value);
    }
    if key == "government" {
        selectors::dismiss(view);
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
    if key == "advance" {
        super::research::clear_description_scroll(view);
    }
    view.selections.insert(key, value);
    true
}
