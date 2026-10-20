//! Original MULTIPLE_SELECTION portrait-grid geometry and stable semantic selection.

use super::{ControlPanelTab, InGameUiModel, MenuRect};
use crate::gameplay::model::{GameState, PlayerId, UnitId};

/// One clickable army/individual entry; wire identity never depends on a slot index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackEntry {
    /// Top unit whose portrait represents this army.
    pub unit: UnitId,
    /// Stable accessible label, disambiguating otherwise identical unit types.
    pub label: String,
    /// Original 32×24 control in `controlpanel.ldl`.
    pub rect: MenuRect,
}

/// Builds at most twelve source-order entries, deduplicating actual army IDs.
#[must_use]
pub fn entries(state: &GameState) -> Vec<StackEntry> {
    let Some(selected) = state.selected_unit() else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut armies = Vec::new();
    for unit in state.units.iter().filter(|unit| {
        unit.alive && unit.owner == PlayerId::Human && unit.position == selected.position
    }) {
        let source = state.viewer.as_ref().and_then(|viewer| {
            viewer
                .units
                .iter()
                .find(|source| UnitId::from_server_id(&source.id) == unit.id)
        });
        if let Some(army) = source
            .and_then(|source| source.facts["army_id"].as_str())
            .filter(|id| !id.is_empty())
        {
            if armies.contains(&army) {
                continue;
            }
            armies.push(army);
        }
        let index = result.len();
        if index >= 12 {
            break;
        }
        let columns = [18_i32, 53_i32, 88_i32, 122_i32];
        let row = i32::try_from(index.div_euclid(4)).unwrap_or(0_i32);
        let rect = MenuRect {
            x: 388_i32.saturating_add(columns[index.rem_euclid(4)]),
            y: 494_i32.saturating_add(row.saturating_mul(27)),
            width: 32,
            height: 24,
        };
        result.push(StackEntry {
            unit: unit.id,
            label: source.map_or_else(
                || format!("Select {} {}", unit.kind.label(), unit.id.0),
                |source| format!("Select {} ({})", source.type_id, source.id),
            ),
            rect,
        });
    }
    result
}

/// Returns source health-bar width and RGB color using viewer-exposed health.
///
/// Omitted health with a known maximum represents Go's zero value; an unknown
/// maximum is not invented. Wide integer arithmetic avoids overflow and floating
/// threshold drift (`UnitControlPanel::HealthBarActionCallback`, 0.333/0.666).
#[must_use]
pub fn health_bar(
    source: Option<&crate::gameplay::server::ServerUnitProjection>,
    width: u32,
) -> Option<(u32, (u8, u8, u8))> {
    let facts = &source?.facts;
    let maximum = u128::from(
        facts["max_health"]
            .as_u64()
            .filter(|value| *value > 0_u64)?,
    );
    let health = u128::from(facts["health"].as_u64().unwrap_or(0_u64)).min(maximum);
    let filled =
        u32::try_from(health.saturating_mul(u128::from(width)).div_euclid(maximum)).ok()?;
    let fraction = health.saturating_mul(1000_u128);
    let color = if fraction > maximum.saturating_mul(666_u128) {
        (0_u8, 255_u8, 0_u8)
    } else if fraction > maximum.saturating_mul(333_u128) {
        (255, 255, 0)
    } else {
        (255, 0, 0)
    };
    Some((filled, color))
}

/// Accessible and physical clicks share this stable identity-based selection.
pub fn activate(state: &mut GameState, ui: &mut InGameUiModel, label: &str) -> bool {
    if !ui.stack_expanded || ui.active_tab != ControlPanelTab::Unit {
        return false;
    }
    let Some(id) = entries(state)
        .iter()
        .find(|entry| entry.label == label)
        .map(|entry| entry.unit)
    else {
        return false;
    };
    if !state.select_unit(id) {
        return false;
    }
    ui.stack_expanded = false;
    true
}

#[cfg(test)]
mod tests {
    use super::{activate, entries, health_bar};
    use crate::gameplay::model::GameState;
    use crate::gameplay::server::ServerProjection;
    use crate::gameplay::ui_model::InGameUiModel;
    #[test]
    fn health_uses_authoritative_ratio_original_thresholds_and_wide_safe_arithmetic() {
        let mut source = crate::gameplay::server::ServerUnitProjection::default();
        for (health, width, color) in [
            (25_u64, 24_u32, (255_u8, 0_u8, 0_u8)),
            (50, 48, (255, 255, 0)),
            (75, 72, (0, 255, 0)),
            (150, 96, (0, 255, 0)),
        ] {
            source.facts = serde_json::json!({"health":health,"max_health":100_u64});
            assert_eq!(
                health_bar(Some(&source), 96_u32),
                Some((width, color)),
                "correct partial health and source color"
            );
        }
        source.facts = serde_json::json!({"health":u64::MAX,"max_health":u64::MAX});
        assert_eq!(
            health_bar(Some(&source), 96_u32).map(|bar| bar.0),
            Some(96_u32),
            "ratio multiplication cannot saturate into a nearly empty full-health bar"
        );
        source.facts = serde_json::json!({"max_health":100_u64});
        assert_eq!(
            health_bar(Some(&source), 96_u32).map(|bar| bar.0),
            Some(0_u32),
            "Go omitempty health zero is not full health"
        );
        source.facts = serde_json::json!({});
        assert!(
            health_bar(Some(&source), 96_u32).is_none(),
            "unknown health is never fabricated full"
        );
        assert!(
            health_bar(None, 96_u32).is_none(),
            "missing source is unknown"
        );
    }

    #[test]
    fn expanded_grid_selects_second_exact_identity_and_collapses_without_order() {
        let projection:ServerProjection=serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":[{"id":"a","x":0,"y":0,"terrain":"plains"}],"units":[{"id":"first","owner_id":"human","type_id":"UNIT_HOPLITE","tile_id":"a"},{"id":"second","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"a"}]}"#).expect("stack");
        let mut state = GameState::from_server_projection(&projection, 1, 42);
        let mut ui = InGameUiModel {
            stack_expanded: true,
            ..InGameUiModel::default()
        };
        let choices = entries(&state);
        assert_eq!(choices.len(), 2_usize, "both solo armies selectable");
        let second = choices[1_usize].unit;
        let before = state.units.clone();
        assert!(
            activate(&mut state, &mut ui, &choices[1_usize].label),
            "semantic selection"
        );
        assert_eq!(
            state.selected_unit,
            Some(second),
            "second identity selected"
        );
        assert!(
            !ui.stack_expanded,
            "source selection returns to single portrait"
        );
        assert_eq!(state.units, before, "selection is not a game order");
    }
}
