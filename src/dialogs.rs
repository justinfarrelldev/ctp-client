//! GTK-native user-facing dialogs used by the game launch workflow.
//!
//! The helpers in this module keep dialog text and response handling testable
//! without opening real windows during automated tests.

use crate::asset_extraction::{
    ExtractionError, build_confirmation_prompt, describe_extraction_error,
};
use gtk4::glib;
use gtk4::prelude::*;
use std::path::Path;

/// Testable text displayed by a modal dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogText {
    /// Window title for the dialog.
    title: String,
    /// Main body copy shown to the player.
    description: String,
}

impl DialogText {
    /// Creates dialog text from a title and description.
    #[must_use]
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }

    /// Returns the title shown in the dialog header.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns the descriptive body text shown in the dialog.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

/// Builds the confirmation copy shown before game data extraction.
#[must_use]
pub fn extraction_confirmation_text(
    destination: &Path,
    archive_count: usize,
    installation_name: &str,
) -> DialogText {
    DialogText::new(
        "Prepare game data",
        build_confirmation_prompt(destination, archive_count, installation_name),
    )
}

/// Builds the error copy shown when game data extraction cannot continue.
#[must_use]
pub fn extraction_error_text(error: &ExtractionError) -> DialogText {
    let (title, description) = describe_extraction_error(error);
    DialogText::new(title, description)
}

/// Returns whether the selected dialog response should proceed.
#[must_use]
pub fn is_affirmative_response(response: gtk4::ResponseType) -> bool {
    match response {
        gtk4::ResponseType::Accept | gtk4::ResponseType::Yes | gtk4::ResponseType::Ok => true,
        gtk4::ResponseType::None
        | gtk4::ResponseType::Reject
        | gtk4::ResponseType::DeleteEvent
        | gtk4::ResponseType::Cancel
        | gtk4::ResponseType::Close
        | gtk4::ResponseType::No
        | gtk4::ResponseType::Apply
        | gtk4::ResponseType::Help => false,
        gtk4::ResponseType::Other(custom_response) => {
            log::debug!("Treating custom dialog response {custom_response} as negative");
            false
        }
        gtk4::ResponseType::__Unknown(raw_response) => {
            log::debug!("Treating unknown dialog response {raw_response} as negative");
            false
        }
    }
}

/// Shows a GTK-native confirmation dialog and returns whether to proceed.
#[must_use]
pub fn run_confirmation_dialog(parent: Option<&gtk4::Window>, text: &DialogText) -> bool {
    let dialog = build_dialog(parent, text);
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
    dialog.add_button("Continue", gtk4::ResponseType::Accept);
    dialog.set_default_response(gtk4::ResponseType::Accept);

    is_affirmative_response(run_modal_dialog(&dialog))
}

/// Shows a GTK-native message dialog and waits until the user dismisses it.
pub fn run_message_dialog(parent: Option<&gtk4::Window>, text: &DialogText) {
    let dialog = build_dialog(parent, text);
    dialog.add_button("OK", gtk4::ResponseType::Ok);
    dialog.set_default_response(gtk4::ResponseType::Ok);
    let selected_response = run_modal_dialog(&dialog);
    log::debug!("Message dialog dismissed with {selected_response}");
}

/// Creates a common modal dialog shell with wrapped, selectable body text.
fn build_dialog(parent: Option<&gtk4::Window>, text: &DialogText) -> gtk4::Dialog {
    let dialog = parent.map_or_else(
        || {
            gtk4::Dialog::builder()
                .title(text.title())
                .modal(true)
                .build()
        },
        |parent_window| {
            gtk4::Dialog::builder()
                .title(text.title())
                .modal(true)
                .transient_for(parent_window)
                .build()
        },
    );

    let body = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    body.set_margin_top(20);
    body.set_margin_bottom(20);
    body.set_margin_start(20);
    body.set_margin_end(20);

    let description = gtk4::Label::new(Some(text.description()));
    description.set_wrap(true);
    description.set_selectable(true);
    description.set_xalign(0.0);
    body.append(&description);

    dialog.content_area().append(&body);
    dialog
}

/// Runs a GTK dialog until GTK emits a response for a button or close action.
fn run_modal_dialog(dialog: &gtk4::Dialog) -> gtk4::ResponseType {
    let response = glib::MainContext::default().block_on(dialog.run_future());
    dialog.close();
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn extraction_confirmation_text_contains_launch_context() {
        let destination = PathBuf::from("/tmp/ctp-client/assets/ctp2/test");
        let text = extraction_confirmation_text(&destination, 10, "Call to Power II");

        assert_eq!(
            text.title(),
            "Prepare game data",
            "confirmation dialog should explain the extraction step"
        );
        assert!(
            text.description().contains("Call to Power II"),
            "confirmation copy should include the installation name"
        );
        assert!(
            text.description().contains("10 data archives"),
            "confirmation copy should include the archive count"
        );
        assert!(
            text.description()
                .contains("/tmp/ctp-client/assets/ctp2/test"),
            "confirmation copy should include the managed destination"
        );
    }

    #[test]
    fn extraction_error_text_uses_user_facing_description() {
        let missing_path = PathBuf::from("ctp2_data/default/sound/sound.zfs");
        let error = ExtractionError::MissingArchives {
            missing: vec![missing_path.clone()],
        };
        let text = extraction_error_text(&error);

        assert_eq!(
            text.title(),
            "Game data missing",
            "missing archive errors should use the user-facing title"
        );
        assert!(
            text.description()
                .contains(&missing_path.display().to_string()),
            "missing archive descriptions should list the missing archive"
        );
        assert!(
            text.description().contains("Verify the installation path"),
            "missing archive descriptions should give recovery guidance"
        );
    }

    #[test]
    fn affirmative_response_mapping_rejects_dismissals() {
        assert!(
            is_affirmative_response(gtk4::ResponseType::Accept),
            "Accept should proceed"
        );
        assert!(
            is_affirmative_response(gtk4::ResponseType::Yes),
            "Yes should proceed"
        );
        assert!(
            is_affirmative_response(gtk4::ResponseType::Ok),
            "Ok should proceed"
        );
        assert!(
            !is_affirmative_response(gtk4::ResponseType::Cancel),
            "Cancel should not proceed"
        );
        assert!(
            !is_affirmative_response(gtk4::ResponseType::Close),
            "Close should not proceed"
        );
        assert!(
            !is_affirmative_response(gtk4::ResponseType::DeleteEvent),
            "Window-manager close should not proceed"
        );
        assert!(
            !is_affirmative_response(gtk4::ResponseType::Other(1)),
            "Unexpected custom responses should not proceed"
        );
    }
}
