//! Current-revision, phase-aware manager authorization; no presentation mutation.
use super::{ManagerCommand, policy};
use crate::gameplay::{model::GameState, server::interface::LegalAction};

impl ManagerCommand {
    /// Rejects stale confirmations and commands with no current authority.
    #[must_use]
    pub fn valid_for(&self, state: &GameState) -> bool {
        if state.network != crate::gameplay::model::NetworkStatus::Ready {
            return false;
        }
        let Some(viewer) = state
            .viewer
            .as_ref()
            .filter(|viewer| viewer.revision == self.revision)
        else {
            return false;
        };
        if let Some(valid) = policy::validate(viewer, &self.route, &self.payload) {
            return valid;
        }
        crate::gameplay::server::commands::valid(viewer, &self.route, &self.payload)
    }
    /// Carries through an advertised action without reconstructing its metadata.
    pub(in crate::gameplay::ui_model) fn advertised(
        state: &GameState,
        action: &LegalAction,
    ) -> Option<Self> {
        let command = Self {
            revision: state.viewer.as_ref().map_or(0, |viewer| viewer.revision),
            route: action.route.clone(),
            payload: action.payload.clone(),
        };
        (action.http_supported() && command.valid_for(state)).then_some(command)
    }
}
