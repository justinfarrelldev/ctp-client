//! Startup asset preparation with explicit, fileless-testable service boundaries.

use crate::asset_extraction::{
    ArchiveExtractor, ExtractionError, ExtractionOutcome, ExtractionSettings,
    build_confirmation_prompt, default_destination_root, log_extraction_progress,
    required_archives,
};
use crate::config::GameInstallation;
use crate::dialogs;
use std::path::{Path, PathBuf};

/// Fileless startup success, cancellation, failure and cached-manifest regressions.
#[cfg(test)]
mod tests;

/// Prepares assets with the same native dialogs, extractor settings and progress
/// callback used by the original binary-owned startup flow. Tests inject these
/// operations below; they never invoke this filesystem/native-dialog adapter.
pub(super) fn prepare(installation: &GameInstallation) -> bool {
    let extractor = ArchiveExtractor::new(installation);
    prepare_with(
        installation,
        |description| {
            let text = dialogs::DialogText::new("Prepare game data", description.to_owned());
            dialogs::run_confirmation_dialog(None, &text)
        },
        |error| {
            let text = dialogs::extraction_error_text(error);
            dialogs::run_message_dialog(None, &text);
        },
        || default_destination_root(installation),
        || extractor.validate().map(drop),
        |destination| {
            let mut progress = log_extraction_progress;
            extractor.extract_with_settings(
                destination,
                ExtractionSettings::default(),
                Some(&mut progress),
            )
        },
    )
}

/// Runs the original resolve -> validate -> confirm -> extract sequence, stopping
/// at the first failure/refusal. Injecting both archive operations prevents tests
/// from constructing/writing archives, manifests or user configuration. Outcomes
/// remain extractor-owned; this function only decides whether launch may proceed.
fn prepare_with(
    installation: &GameInstallation,
    prompt: impl FnOnce(&str) -> bool,
    on_error: impl FnOnce(&ExtractionError),
    resolve_destination: impl FnOnce() -> Result<PathBuf, ExtractionError>,
    validate: impl FnOnce() -> Result<(), ExtractionError>,
    extract: impl FnOnce(&Path) -> Result<ExtractionOutcome, ExtractionError>,
) -> bool {
    let destination = match resolve_destination() {
        Ok(path) => path,
        Err(error) => {
            log::error!("Failed to resolve extraction target: {error}");
            on_error(&error);
            return false;
        }
    };
    if let Err(error) = validate() {
        log::error!("Archive validation failed: {error}");
        on_error(&error);
        return false;
    }
    let description = build_confirmation_prompt(
        &destination,
        required_archives(installation.version).len(),
        &installation.display_name,
    );
    if !prompt(&description) {
        log::info!("User canceled extraction for {}", installation.display_name);
        return false;
    }
    match extract(&destination) {
        Ok(outcome) => {
            if outcome.already_up_to_date {
                log::info!(
                    "Game data for {} is already up to date at {}",
                    installation.display_name,
                    destination.display()
                );
            } else {
                let files_written: usize = outcome
                    .archives
                    .iter()
                    .map(|summary| summary.files_written)
                    .sum();
                let bytes_written: u64 = outcome
                    .archives
                    .iter()
                    .map(|summary| summary.bytes_written)
                    .sum();
                log::info!(
                    "Extracted {} archives ({} files, {} bytes) to {}",
                    outcome.archives.len(),
                    files_written,
                    bytes_written,
                    destination.display()
                );
            }
            true
        }
        Err(error) => {
            log::error!("Extraction failed: {error}");
            on_error(&error);
            false
        }
    }
}
