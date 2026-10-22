//! Stable semantic manager identities and source-root/action mapping.
use crate::gameplay::ui_model::InGameAction;

use super::ManagerKind;
impl ManagerKind {
    /// All implemented source-layout roots, including nested research selection.
    pub const ALL: [Self; 16] = [
        Self::Build,
        Self::Empire,
        Self::National,
        Self::City,
        Self::Unit,
        Self::Army,
        Self::Trade,
        Self::Diplomacy,
        Self::DiplomacyDetails,
        Self::DiplomacyReview,
        Self::Science,
        Self::Research,
        Self::Library,
        Self::Statistics,
        Self::Graphics,
        Self::Sound,
    ];
    /// Diagnostic/automation name, independent of localized dialog titles.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Empire => "empire",
            Self::National => "national",
            Self::City => "city",
            Self::Unit => "unit",
            Self::Army => "army",
            Self::Trade => "trade",
            Self::Diplomacy => "diplomacy",
            Self::DiplomacyDetails => "diplomacy-details",
            Self::DiplomacyReview => "diplomacy-review",
            Self::Science => "science",
            Self::Research => "research",
            Self::Library => "library",
            Self::Statistics => "statistics",
            Self::Graphics => "graphics",
            Self::Sound => "sound",
        }
    }
    /// Parses an explicit diagnostic name without falling back to another manager.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name().eq_ignore_ascii_case(name))
    }
    /// Original LDL instance whose inherited children form this dialog.
    #[must_use]
    pub const fn root(self) -> &'static str {
        match self {
            Self::Build => "BuildEditorWindow",
            Self::Empire => "DomesticDialog",
            Self::National => "CityStatusWin",
            Self::City => "CityWindow",
            Self::Unit => "UnitManager",
            Self::Army => "ArmyManager",
            Self::Trade => "TradeManager",
            Self::Diplomacy => "DiplomacyWindow",
            Self::DiplomacyDetails => "DiplomacyDetails",
            Self::DiplomacyReview => "DipWizard",
            Self::Science => "ScienceManagementDialog",
            Self::Research => "SciAdvanceScreen",
            Self::Library => "GreatLibrary",
            Self::Statistics => "InfoDialog",
            Self::Graphics => "GraphicsWindow",
            Self::Sound => "SoundWindow",
        }
    }
}
/// Resolves distinct original manager entry points without collapsing them to tabs.
#[must_use]
pub(super) const fn kind_for_action(action: InGameAction) -> Option<ManagerKind> {
    match action {
        InGameAction::OpenCityBuild => Some(ManagerKind::Build),
        InGameAction::OpenCityManager => Some(ManagerKind::City),
        InGameAction::OpenNationalManager => Some(ManagerKind::National),
        InGameAction::OpenTradeManager => Some(ManagerKind::Trade),
        InGameAction::OpenCivilizationStatus => Some(ManagerKind::Empire),
        InGameAction::OpenUnitManager => Some(ManagerKind::Unit),
        InGameAction::OpenArmyManager => Some(ManagerKind::Army),
        InGameAction::OpenDiplomacy => Some(ManagerKind::Diplomacy),
        InGameAction::OpenScienceStatus => Some(ManagerKind::Science),
        InGameAction::OpenGreatLibrary => Some(ManagerKind::Library),
        InGameAction::OpenInfo | InGameAction::OpenScore | InGameAction::OpenWonders => {
            Some(ManagerKind::Statistics)
        }
        InGameAction::OpenGraphicsOptions => Some(ManagerKind::Graphics),
        InGameAction::OpenSoundOptions => Some(ManagerKind::Sound),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ManagerKind, kind_for_action};
    use crate::gameplay::ui_model::InGameAction;
    use alloc::collections::BTreeSet;
    #[test]
    fn source_manager_identity_names_are_unique_round_trip_and_keep_distinct_entry_actions() {
        let names: BTreeSet<_> = ManagerKind::ALL
            .into_iter()
            .map(ManagerKind::name)
            .collect();
        assert_eq!(
            names.len(),
            ManagerKind::ALL.len(),
            "stable accessible IDs do not alias"
        );
        for kind in ManagerKind::ALL {
            assert_eq!(
                ManagerKind::from_name(kind.name()),
                Some(kind),
                "round-trip semantic identity"
            );
            assert!(
                !kind.root().is_empty(),
                "every manager keeps its actual source root"
            );
        }
        assert_eq!(
            kind_for_action(InGameAction::OpenNationalManager),
            Some(ManagerKind::National),
            "not City or Empire"
        );
        assert_eq!(
            kind_for_action(InGameAction::OpenCityManager),
            Some(ManagerKind::City),
            "distinct city dialog"
        );
        assert_eq!(
            kind_for_action(InGameAction::OpenCityBuild),
            Some(ManagerKind::Build),
            "distinct production dialog"
        );
    }
}
