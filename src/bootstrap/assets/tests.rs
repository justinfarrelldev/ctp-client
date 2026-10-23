//! Startup extraction orchestration with in-memory traces, never real archives.

use super::prepare_with;
use crate::asset_extraction::{
    ArchiveExtractionSummary, ExtractionError, ExtractionOutcome, build_confirmation_prompt,
    required_archives,
};
use crate::config::{GameInstallation, GameVersion};
use core::cell::RefCell;
use std::path::{Path, PathBuf};

/// One injected outcome at the actual startup service boundaries.
#[derive(Clone, Copy)]
enum Scenario {
    /// Destination resolution fails before archive validation.
    DestinationFailure,
    /// Required archives are unavailable.
    ValidationFailure,
    /// The user explicitly declines preparation.
    Declined,
    /// Extraction fails after approval.
    ExtractionFailure,
    /// Extraction and verification complete normally.
    Extracted,
    /// The existing verified manifest requires no extraction.
    UpToDate,
}

/// Per-test callback records with no shared state, filesystem or native dialogs.
#[derive(Default)]
struct Harness {
    /// Ordered external operations performed by startup.
    calls: RefCell<Vec<&'static str>>,
    /// Exact question passed to the confirmation boundary.
    prompts: RefCell<Vec<String>>,
    /// Formatted errors passed to the error-dialog boundary.
    errors: RefCell<Vec<String>>,
}

impl Harness {
    /// Executes production orchestration using only the injected operation results.
    fn run(&self, installation: &GameInstallation, scenario: Scenario) -> bool {
        prepare_with(
            installation,
            |description| {
                self.calls.borrow_mut().push("prompt");
                self.prompts.borrow_mut().push(description.to_owned());
                !matches!(scenario, Scenario::Declined)
            },
            |error| {
                self.calls.borrow_mut().push("error");
                self.errors.borrow_mut().push(error.to_string());
            },
            || {
                self.calls.borrow_mut().push("resolve");
                if matches!(scenario, Scenario::DestinationFailure) {
                    Err(ExtractionError::PathResolution {
                        message: "resolver failure".to_owned(),
                    })
                } else {
                    Ok(installation.path.join("destination"))
                }
            },
            || {
                self.calls.borrow_mut().push("validate");
                if matches!(scenario, Scenario::ValidationFailure) {
                    Err(ExtractionError::MissingArchives {
                        missing: vec![installation.path.join("missing.zfs")],
                    })
                } else {
                    Ok(())
                }
            },
            |destination| {
                self.calls.borrow_mut().push("extract");
                assert_eq!(
                    destination,
                    installation.path.join("destination"),
                    "only the resolved destination reaches extraction"
                );
                if matches!(scenario, Scenario::ExtractionFailure) {
                    Err(ExtractionError::Archive {
                        path: installation.path.join("corrupt.zfs"),
                        message: "corrupt archive".to_owned(),
                    })
                } else {
                    Ok(ExtractionOutcome {
                        destination_root: destination.to_path_buf(),
                        archives: vec![ArchiveExtractionSummary {
                            archive: PathBuf::from("fixture.zfs"),
                            files_written: 2_usize,
                            bytes_written: 9_u64,
                            files_skipped: 0_usize,
                            bytes_skipped: 0_u64,
                        }],
                        already_up_to_date: matches!(scenario, Scenario::UpToDate),
                        verified: true,
                    })
                }
            },
        )
    }

    /// Verifies both the service order and whether an error was presented.
    fn assert_trace(&self, expected: &[&str], expected_errors: usize) {
        assert_eq!(
            self.calls.borrow().as_slice(),
            expected,
            "startup must short-circuit without later filesystem/dialog operations"
        );
        assert_eq!(
            self.errors.borrow().len(),
            expected_errors,
            "each failure is presented exactly once; cancellation is not an error"
        );
    }
}

/// A symbolic installation; no path is created, probed or modified by this harness.
fn installation(version: GameVersion) -> GameInstallation {
    GameInstallation::new(
        Path::new("mock-installation").to_path_buf(),
        version,
        "Test Installation".to_owned(),
        None,
    )
}

/// Preserves the original destination-resolution failure regression.
#[test]
fn prepare_assets_returns_false_when_destination_resolution_fails() {
    let harness = Harness::default();
    assert!(
        !harness.run(
            &installation(GameVersion::CTP2),
            Scenario::DestinationFailure
        ),
        "destination failure aborts launch"
    );
    harness.assert_trace(&["resolve", "error"], 1_usize);
    assert!(
        harness.prompts.borrow().is_empty(),
        "no question on failure"
    );
    assert!(
        harness.errors.borrow()[0_usize].contains("resolver failure"),
        "preserve destination error context"
    );
}

/// Preserves the original missing-archive validation regression.
#[test]
fn prepare_assets_returns_false_when_validation_fails() {
    let harness = Harness::default();
    assert!(
        !harness.run(
            &installation(GameVersion::CTP2),
            Scenario::ValidationFailure
        ),
        "validation failure aborts launch"
    );
    harness.assert_trace(&["resolve", "validate", "error"], 1_usize);
    assert!(
        harness.prompts.borrow().is_empty(),
        "no invalid-archive question"
    );
    assert!(
        harness.errors.borrow()[0_usize].contains("missing.zfs"),
        "preserve exact missing-archive context"
    );
}

/// Preserves cancellation and proves no extraction/manifest operation is requested.
#[test]
fn prepare_assets_stops_when_user_declines() {
    let harness = Harness::default();
    assert!(
        !harness.run(&installation(GameVersion::CTP2), Scenario::Declined),
        "explicit refusal aborts launch"
    );
    harness.assert_trace(&["resolve", "validate", "prompt"], 0_usize);
    assert_eq!(
        harness.prompts.borrow().len(),
        1_usize,
        "one explicit question"
    );
}

/// Preserves the post-approval extraction-error regression without corrupt files.
#[test]
fn prepare_assets_reports_extraction_failure() {
    let harness = Harness::default();
    assert!(
        !harness.run(
            &installation(GameVersion::CTP2),
            Scenario::ExtractionFailure
        ),
        "extraction failure aborts launch"
    );
    harness.assert_trace(
        &["resolve", "validate", "prompt", "extract", "error"],
        1_usize,
    );
    assert!(
        harness.errors.borrow()[0_usize].contains("corrupt archive"),
        "preserve extraction failure context"
    );
}

/// Preserves successful preparation and exact version-specific confirmation text.
#[test]
fn prepare_assets_succeeds_with_valid_setup() {
    for version in [GameVersion::CTP1, GameVersion::CTP2] {
        let installation = installation(version);
        let harness = Harness::default();
        assert!(
            harness.run(&installation, Scenario::Extracted),
            "verified preparation permits launch for {version:?}"
        );
        harness.assert_trace(&["resolve", "validate", "prompt", "extract"], 0_usize);
        assert_eq!(
            harness.prompts.borrow().as_slice(),
            &[build_confirmation_prompt(
                &installation.path.join("destination"),
                required_archives(version).len(),
                &installation.display_name,
            )],
            "the original prompt uses this installation, archive count and destination"
        );
    }
}

/// Covers the existing no-write/up-to-date production outcome as a distinct success.
#[test]
fn prepare_assets_reuses_up_to_date_installation() {
    let harness = Harness::default();
    assert!(
        harness.run(&installation(GameVersion::CTP2), Scenario::UpToDate),
        "an existing verified manifest still permits launch"
    );
    harness.assert_trace(&["resolve", "validate", "prompt", "extract"], 0_usize);
}
