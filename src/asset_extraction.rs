//! Asset extraction pipeline for Call to Power installations.
//!
//! This module verifies the presence of required ZFS archives and
//! orchestrates their extraction into an application-managed data directory.

extern crate alloc;

use crate::config::{GameInstallation, GameVersion};
#[cfg(test)]
use core::cell::Cell;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::Disks;
use unciv::{ZfsEntry, ZfsFile};

/// Describes a required ZFS archive relative to the installation root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveDescriptor {
    /// Relative path to the archive from the installation root.
    pub relative_path: &'static str,
}

/// Summary of a single archive extraction operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveExtractionSummary {
    /// Relative path of the source archive processed.
    pub archive: PathBuf,
    /// Number of files written during extraction.
    pub files_written: usize,
    /// Total bytes written during extraction.
    pub bytes_written: u64,
    /// Number of files skipped because matching assets already existed.
    pub files_skipped: usize,
    /// Number of bytes skipped due to existing assets.
    pub bytes_skipped: u64,
}

/// Aggregated result of an extraction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionOutcome {
    /// Destination root where assets were written.
    pub destination_root: PathBuf,
    /// Summaries for each processed archive.
    pub archives: Vec<ArchiveExtractionSummary>,
    /// Indicates that the destination already contained up-to-date assets.
    pub already_up_to_date: bool,
    /// Reflects whether a post-extraction verification pass succeeded.
    pub verified: bool,
}

/// Plan describing an archive that needs to be extracted.
#[derive(Debug)]
pub struct ArchivePlan {
    /// Descriptor originating from the static manifest.
    pub descriptor: &'static ArchiveDescriptor,
    /// Absolute path to the archive file on disk.
    pub source: PathBuf,
    /// Relative directory (excluding the archive filename) under the installation root.
    pub relative_parent: PathBuf,
}

/// Default filename for the persisted extraction manifest.
const MANIFEST_FILE_NAME: &str = ".ctp-asset-manifest.json";
/// Manifest format version for forward-compatibility checks.
const MANIFEST_VERSION: u32 = 1;

/// Serialized record describing a completed extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ExtractionManifest {
    /// Manifest schema version for future migrations.
    version: u32,
    /// Hash fingerprint derived from the source installation path.
    installation_fingerprint: u64,
    /// Per-archive metadata captured during extraction.
    archives: Vec<ArchiveManifestEntry>,
}

/// Metadata for an individual archive recorded in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArchiveManifestEntry {
    /// Relative archive path within the installation.
    relative_path: String,
    /// Size of the archive on disk (compressed bytes).
    len_bytes: u64,
    /// Last modification timestamp recorded for the archive, seconds since Unix epoch.
    modified_seconds: u64,
}

/// Settings that influence how extraction is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractionSettings {
    /// When true, existing assets described by the manifest are reused.
    pub skip_if_up_to_date: bool,
    /// When true, extracted files are validated against archive metadata.
    pub verify_integrity: bool,
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            skip_if_up_to_date: true,
            verify_integrity: true,
        }
    }
}

/// Control directive emitted by progress callbacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressControl {
    /// Continue extraction.
    Continue,
    /// Abort extraction with an explanatory message.
    Abort { message: String },
}

/// Event data supplied to progress callbacks.
#[derive(Debug, Clone)]
pub struct ExtractionProgressEvent {
    /// Discrete stage update describing current state of the extractor.
    pub stage: ExtractionProgressStage,
}

/// Enumeration of progress stages emitted during extraction.
#[derive(Debug, Clone)]
pub enum ExtractionProgressStage {
    /// Archive validation is in progress.
    Validating { archive_count: usize },
    /// Existing assets satisfied the request; extraction is skipped.
    UpToDate,
    /// Disk space availability is being evaluated.
    DiskSpaceCheck {
        required_bytes: u64,
        available_bytes: u64,
        destination: PathBuf,
    },
    /// Extraction of a specific archive has started.
    ArchiveStarted {
        archive_index: usize,
        archive_total: usize,
        archive_path: PathBuf,
        entry_count: usize,
        expected_bytes: u64,
    },
    /// A single archive entry has been processed.
    EntryProcessed {
        archive_index: usize,
        entry_index: usize,
        entry_total: usize,
        entry_name: String,
        entry_size: u64,
        written_bytes_total: u64,
        expected_total_bytes: u64,
    },
    /// Extraction has completed successfully.
    Completed,
}

#[cfg(test)]
thread_local! {
    /// Per-test-thread disk space override used to avoid host filesystem coupling.
    static DISK_SPACE_OVERRIDE: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Computes a stable fingerprint for the provided installation.
fn installation_fingerprint(installation: &GameInstallation) -> u64 {
    let mut hasher = DefaultHasher::new();
    installation.path.hash(&mut hasher);
    hasher.finish()
}

/// Resolves the manifest file path within the destination directory.
fn manifest_path(destination_root: &Path) -> PathBuf {
    destination_root.join(MANIFEST_FILE_NAME)
}

/// Converts a `SystemTime` into seconds relative to the Unix epoch.
fn encode_system_time(time: SystemTime) -> u64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(error) => error.duration().as_secs(),
    }
}

/// Obtains file size and modification timestamp metadata for the given archive.
fn archive_metadata_signature(path: &Path) -> Result<(u64, u64), ExtractionError> {
    let metadata = fs::metadata(path).map_err(|source| ExtractionError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let len_bytes = metadata.len();
    let modified = metadata.modified().map(encode_system_time).unwrap_or(0);
    Ok((len_bytes, modified))
}

/// Loads the persisted manifest if it exists.
fn load_manifest(path: &Path) -> Result<Option<ExtractionManifest>, ExtractionError> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read(path).map_err(|source| ExtractionError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    serde_json::from_slice(&content)
        .map(Some)
        .map_err(|error| ExtractionError::Manifest {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

/// Persists the manifest to disk, ensuring the destination directory exists.
fn save_manifest(path: &Path, manifest: &ExtractionManifest) -> Result<(), ExtractionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ExtractionError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let content =
        serde_json::to_vec_pretty(manifest).map_err(|error| ExtractionError::Manifest {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

    fs::write(path, content).map_err(|source| ExtractionError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Computes the available disk space at the destination and enforces requirements.
fn ensure_disk_space(destination: &Path, required_bytes: u64) -> Result<u64, ExtractionError> {
    #[cfg(test)]
    {
        let override_value = DISK_SPACE_OVERRIDE.with(Cell::get);
        if let Some(value) = override_value {
            if value < required_bytes {
                return Err(ExtractionError::DiskSpace {
                    destination: destination.to_path_buf(),
                    required_bytes,
                    available_bytes: value,
                });
            }

            return Ok(value);
        }
    }

    let absolute_destination = destination
        .canonicalize()
        .unwrap_or_else(|_| destination.to_path_buf());

    let disks = Disks::new_with_refreshed_list();
    let mut selected_available = None;
    let mut selected_mount_len = 0_usize;

    for disk in &disks {
        let mount = disk.mount_point();
        if absolute_destination.starts_with(mount) {
            let mount_len = mount.components().count();
            if mount_len >= selected_mount_len {
                selected_mount_len = mount_len;
                selected_available = Some(disk.available_space());
            }
        }
    }

    let available = selected_available
        .filter(|available| *available > 0)
        .unwrap_or_else(|| {
            log::warn!(
                "Unable to determine available disk space for {}; continuing extraction",
                destination.display()
            );
            u64::MAX
        });

    if available < required_bytes {
        return Err(ExtractionError::DiskSpace {
            destination: destination.to_path_buf(),
            required_bytes,
            available_bytes: available,
        });
    }

    Ok(available)
}

/// Sends a progress event to the callback if one is configured.
fn emit_progress(
    progress: &mut Option<&mut dyn FnMut(ExtractionProgressEvent) -> ProgressControl>,
    stage: ExtractionProgressStage,
) -> Result<(), ExtractionError> {
    progress.as_deref_mut().map_or(Ok(()), |callback| {
        match callback(ExtractionProgressEvent { stage }) {
            ProgressControl::Continue => Ok(()),
            ProgressControl::Abort { message } => Err(ExtractionError::Aborted { message }),
        }
    })
}

#[cfg(test)]
fn set_disk_space_override(value: Option<u64>) {
    DISK_SPACE_OVERRIDE.with(|override_cell| override_cell.set(value));
}

/// Errors that can occur during asset extraction.
#[derive(Debug)]
pub enum ExtractionError {
    /// One or more archives are missing.
    MissingArchives { missing: Vec<PathBuf> },
    /// Failed to resolve the destination path.
    PathResolution { message: String },
    /// Filesystem operation failed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Archive-specific failure.
    Archive { path: PathBuf, message: String },
    /// Not enough disk space is available at the destination.
    DiskSpace {
        destination: PathBuf,
        required_bytes: u64,
        available_bytes: u64,
    },
    /// Extraction aborted by the caller through the progress callback.
    Aborted { message: String },
    /// Persistence of the extraction manifest failed.
    Manifest { path: PathBuf, message: String },
}

impl Display for ExtractionError {
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Borrowed enum matching keeps non-Copy error fields available for formatting"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::MissingArchives { missing } => {
                if missing.is_empty() {
                    write!(f, "No archives were found for extraction")
                } else {
                    let paths = missing
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "Missing archives: {paths}")
                }
            }
            Self::PathResolution { message } => write!(f, "Failed to resolve path: {message}"),
            Self::Io { path, source } => {
                write!(f, "I/O error at {}: {source}", path.display())
            }
            Self::Archive { path, message } => {
                write!(f, "Archive error for {}: {message}", path.display())
            }
            Self::DiskSpace {
                destination,
                required_bytes,
                available_bytes,
            } => write!(
                f,
                "Insufficient disk space at {} (required: {} bytes, available: {} bytes)",
                destination.display(),
                required_bytes,
                available_bytes
            ),
            Self::Aborted { message } => write!(f, "Extraction aborted: {message}"),
            Self::Manifest { path, message } => {
                write!(f, "Manifest error at {}: {message}", path.display())
            }
        }
    }
}

impl Error for ExtractionError {
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Borrowed enum matching exposes the stored source error by reference"
    )]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Provides the list of required archives for the given game version.
#[must_use]
pub const fn required_archives(version: GameVersion) -> &'static [ArchiveDescriptor] {
    match version {
        GameVersion::CTP1 => &CTP1_ARCHIVES,
        GameVersion::CTP2 => &CTP2_ARCHIVES,
    }
}

/// Required CTP1 archives in extraction order.
const CTP1_ARCHIVES: [ArchiveDescriptor; 11] = [
    ArchiveDescriptor {
        relative_path: "ctp_data/default/graphics/patterns/pat555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/default/graphics/patterns/pat565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/default/graphics/pictures/pic555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/default/graphics/pictures/pic565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/default/sound/sound.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/english/gamedata/gl/gl.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/english/graphics/patterns/pat555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/english/graphics/patterns/pat565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/english/graphics/pictures/pic555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/english/graphics/pictures/pic565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp_data/english/sound/sound.zfs",
    },
];

/// Required CTP2 archives in extraction order.
const CTP2_ARCHIVES: [ArchiveDescriptor; 10] = [
    ArchiveDescriptor {
        relative_path: "ctp2_data/default/graphics/patterns/pat555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/default/graphics/patterns/pat565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/default/graphics/pictures/pic555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/default/graphics/pictures/pic565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/default/sound/sound.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/english/graphics/patterns/pat555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/english/graphics/patterns/pat565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/english/graphics/pictures/pic555.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/english/graphics/pictures/pic565.zfs",
    },
    ArchiveDescriptor {
        relative_path: "ctp2_data/english/sound/sound.zfs",
    },
];

/// Sanitizes a display name component for safe use in an asset directory name.
fn sanitize_component(input: &str) -> String {
    let mut sanitized = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_') {
            sanitized.push(ch);
        } else if ch.is_whitespace() || ch.is_ascii_punctuation() {
            sanitized.push('-');
        } else {
            sanitized.push('_');
        }
    }

    let mut collapsed = String::with_capacity(sanitized.len());
    let mut previous_separator = false;
    for ch in sanitized.chars() {
        let is_separator = matches!(ch, '-' | '_');
        if is_separator && previous_separator {
            continue;
        }
        collapsed.push(ch);
        previous_separator = is_separator;
    }

    let trimmed = collapsed.trim_matches(|c| c == '-' || c == '_').to_owned();
    if trimmed.is_empty() {
        "installation".to_owned()
    } else {
        trimmed
    }
}

/// Determines the default destination directory for extracted assets.
///
/// # Errors
///
/// Returns [`ExtractionError::PathResolution`] when the operating system data
/// directory cannot be determined.
pub fn default_destination_root(
    installation: &GameInstallation,
) -> Result<PathBuf, ExtractionError> {
    let base = dirs::data_dir().ok_or_else(|| ExtractionError::PathResolution {
        message: "Unable to determine system data directory".to_owned(),
    })?;

    let version_component = match installation.version {
        GameVersion::CTP1 => "ctp1",
        GameVersion::CTP2 => "ctp2",
    };

    let mut hasher = DefaultHasher::new();
    installation.path.hash(&mut hasher);
    let fingerprint = hasher.finish();

    let sanitized_name = sanitize_component(&installation.display_name);

    Ok(base
        .join("ctp-client")
        .join("assets")
        .join(version_component)
        .join(format!("{sanitized_name}-{fingerprint:016x}")))
}

/// Builds a human-readable confirmation message for archive extraction.
#[must_use]
pub fn build_confirmation_prompt(
    destination: &Path,
    archive_count: usize,
    installation_name: &str,
) -> String {
    format!(
        "{installation_name} requires {archive_count} data archives to be copied before play.\n\nThe archives will be extracted to:\n{}\n\nOriginal installation files remain untouched.",
        destination.display()
    )
}

/// Logs a progress event and instructs extraction to continue.
#[must_use]
pub fn log_extraction_progress(event: ExtractionProgressEvent) -> ProgressControl {
    match event.stage {
        ExtractionProgressStage::Validating { archive_count } => {
            log::info!("Validating {archive_count} required data archives");
        }
        ExtractionProgressStage::UpToDate => {
            log::info!("Extracted game data is already up to date");
        }
        ExtractionProgressStage::DiskSpaceCheck {
            required_bytes,
            available_bytes,
            destination,
        } => {
            log::info!(
                "Disk space check for {}: {} bytes required, {} bytes available",
                destination.display(),
                required_bytes,
                available_bytes
            );
        }
        ExtractionProgressStage::ArchiveStarted {
            archive_index,
            archive_total,
            archive_path,
            entry_count,
            expected_bytes,
        } => {
            log::info!(
                "Extracting archive {}/{}: {} ({} entries, {} bytes)",
                archive_index.saturating_add(1),
                archive_total,
                archive_path.display(),
                entry_count,
                expected_bytes
            );
        }
        ExtractionProgressStage::EntryProcessed {
            archive_index,
            entry_index,
            entry_total,
            entry_name,
            entry_size,
            written_bytes_total,
            expected_total_bytes,
        } => {
            log::debug!(
                "Archive {} entry {}/{} processed: {} ({} bytes, {} / {} total bytes)",
                archive_index.saturating_add(1),
                entry_index.saturating_add(1),
                entry_total,
                entry_name,
                entry_size,
                written_bytes_total,
                expected_total_bytes
            );
        }
        ExtractionProgressStage::Completed => {
            log::info!("Game data extraction completed");
        }
    }

    ProgressControl::Continue
}

/// Formats a user-facing error title and description for extraction failures.
#[must_use]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Borrowed enum matching keeps non-Copy error fields available for descriptions"
)]
pub fn describe_extraction_error(error: &ExtractionError) -> (&'static str, String) {
    match error {
        ExtractionError::MissingArchives { missing } => {
            let mut body = String::from("The following archives could not be found:\n");
            for path in missing {
                body.push_str("• ");
                body.push_str(&path.display().to_string());
                body.push('\n');
            }
            body.push_str("\nVerify the installation path or reinstall the game.");
            ("Game data missing", body)
        }
        ExtractionError::PathResolution { message } => {
            ("Unable to prepare destination", message.clone())
        }
        ExtractionError::Io { path, source } => (
            "File system error",
            format!("Failed to access {}: {source}", path.display()),
        ),
        ExtractionError::Archive { path, message } => (
            "Archive extraction failed",
            format!("{message} ({})", path.display()),
        ),
        ExtractionError::DiskSpace {
            destination,
            required_bytes,
            available_bytes,
        } => (
            "Not enough disk space",
            format!(
                "Extraction requires {} bytes at {} but only {} bytes are available.",
                required_bytes,
                destination.display(),
                available_bytes
            ),
        ),
        ExtractionError::Aborted { message } => ("Extraction canceled", message.clone()),
        ExtractionError::Manifest { path, message } => (
            "Manifest persistence failed",
            format!("{} (path: {})", message, path.display()),
        ),
    }
}

/// Resolves an archive entry path beneath the target directory and rejects traversal.
fn resolve_entry_path(
    destination_dir: &Path,
    entry_name: &str,
    source: &Path,
) -> Result<PathBuf, ExtractionError> {
    let relative_path = Path::new(entry_name);

    if relative_path.is_absolute() {
        return Err(ExtractionError::Archive {
            path: source.to_path_buf(),
            message: format!("Archive entry '{entry_name}' has absolute path"),
        });
    }

    let mut target = destination_dir.to_path_buf();
    for component in relative_path.components() {
        match component {
            Component::Normal(segment) => target.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ExtractionError::Archive {
                    path: source.to_path_buf(),
                    message: format!(
                        "Archive entry '{entry_name}' has unsupported path components"
                    ),
                });
            }
        }
    }

    Ok(target)
}

/// Plans and executes extraction for an installation.
pub struct ArchiveExtractor<'installation> {
    /// Installation whose archives will be validated and extracted.
    installation: &'installation GameInstallation,
}

/// Mutable state shared while one archive is extracted.
struct ArchiveExtractionContext<'context, 'progress> {
    /// Archive plan being processed.
    plan: &'context ArchivePlan,
    /// Root directory receiving extracted assets.
    destination_root: &'context Path,
    /// Zero-based archive index for progress reporting.
    archive_index: usize,
    /// Total number of archives in this extraction run.
    archive_total: usize,
    /// Total expected bytes across all archives.
    expected_total_bytes: u64,
    /// Cumulative bytes processed across the run.
    cumulative_written: &'context mut u64,
    /// Optional progress callback supplied by the caller.
    progress:
        &'context mut Option<&'progress mut dyn FnMut(ExtractionProgressEvent) -> ProgressControl>,
}

/// Mutable state for processing one archive entry.
struct EntryExtractionContext<'entry, 'context, 'progress> {
    /// Reader positioned over the archive file.
    reader: &'entry mut BufReader<File>,
    /// Directory where this archive's entries are written.
    target_dir: &'entry Path,
    /// Archive entry currently being processed.
    entry: &'entry ZfsEntry,
    /// Zero-based entry index for progress reporting.
    entry_index: usize,
    /// Total number of entries in the archive.
    entry_total: usize,
    /// Shared archive extraction state.
    archive: &'entry mut ArchiveExtractionContext<'context, 'progress>,
    /// Summary accumulated for the archive.
    summary: &'entry mut ArchiveExtractionSummary,
}

impl<'installation> ArchiveExtractor<'installation> {
    /// Creates a new extractor for the given installation.
    #[must_use]
    pub const fn new(installation: &'installation GameInstallation) -> Self {
        Self { installation }
    }

    /// Validates required archives and returns their absolute paths.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError::MissingArchives`] when any required archive is
    /// absent, or [`ExtractionError::PathResolution`] if a descriptor cannot be
    /// mapped to a destination directory.
    pub fn validate(&self) -> Result<Vec<ArchivePlan>, ExtractionError> {
        let mut missing = Vec::new();
        let mut resolved = Vec::new();
        for descriptor in required_archives(self.installation.version) {
            let absolute = self.installation.path.join(descriptor.relative_path);
            if absolute.exists() {
                let relative_parent = Path::new(descriptor.relative_path)
                    .parent()
                    .ok_or_else(|| ExtractionError::PathResolution {
                        message: format!(
                            "Archive path missing parent directory: {}",
                            descriptor.relative_path
                        ),
                    })?
                    .to_path_buf();

                resolved.push(ArchivePlan {
                    descriptor,
                    source: absolute,
                    relative_parent,
                });
            } else {
                missing.push(absolute);
            }
        }

        if missing.is_empty() {
            Ok(resolved)
        } else {
            Err(ExtractionError::MissingArchives { missing })
        }
    }

    /// Extracts archives using default settings, writing assets into `destination_root`.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError`] when validation, disk space checks, archive
    /// reading, file writing, verification, or manifest persistence fails.
    pub fn extract(&self, destination_root: &Path) -> Result<ExtractionOutcome, ExtractionError> {
        self.extract_with_settings(destination_root, ExtractionSettings::default(), None)
    }

    /// Extracts archives using the provided settings and optional progress callback.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractionError`] when validation, disk space checks, archive
    /// reading, file writing, verification, manifest persistence, or caller
    /// cancellation through the progress callback fails.
    pub fn extract_with_settings(
        &self,
        destination_root: &Path,
        settings: ExtractionSettings,
        mut progress: Option<&mut dyn FnMut(ExtractionProgressEvent) -> ProgressControl>,
    ) -> Result<ExtractionOutcome, ExtractionError> {
        if destination_root.as_os_str().is_empty() {
            return Err(ExtractionError::PathResolution {
                message: "Destination root cannot be empty".to_owned(),
            });
        }

        fs::create_dir_all(destination_root).map_err(|source| ExtractionError::Io {
            path: destination_root.to_path_buf(),
            source,
        })?;

        let plans = self.validate()?;
        emit_progress(
            &mut progress,
            ExtractionProgressStage::Validating {
                archive_count: plans.len(),
            },
        )?;

        let manifest = self.build_manifest(&plans)?;
        let manifest_file = manifest_path(destination_root);

        if settings.skip_if_up_to_date {
            if let Some(existing) = load_manifest(&manifest_file)? {
                if existing == manifest {
                    emit_progress(&mut progress, ExtractionProgressStage::UpToDate)?;
                    if settings.verify_integrity {
                        Self::verify_integrity(&plans, destination_root)?;
                    }
                    return Ok(ExtractionOutcome {
                        destination_root: destination_root.to_path_buf(),
                        archives: Vec::new(),
                        already_up_to_date: true,
                        verified: settings.verify_integrity,
                    });
                }
            }
        }

        let required_bytes = Self::estimate_required_bytes(&plans)?;
        let buffer_bytes =
            required_bytes
                .checked_div(10)
                .ok_or_else(|| ExtractionError::PathResolution {
                    message: "Failed to compute extraction disk space buffer".to_owned(),
                })?;
        let buffered_requirement = required_bytes.saturating_add(buffer_bytes);
        let available_bytes = ensure_disk_space(destination_root, buffered_requirement)?;
        emit_progress(
            &mut progress,
            ExtractionProgressStage::DiskSpaceCheck {
                required_bytes: buffered_requirement,
                available_bytes,
                destination: destination_root.to_path_buf(),
            },
        )?;

        let mut outcome = ExtractionOutcome {
            destination_root: destination_root.to_path_buf(),
            archives: Vec::with_capacity(plans.len()),
            already_up_to_date: false,
            verified: false,
        };

        let mut cumulative_written = 0_u64;

        for (index, plan) in plans.iter().enumerate() {
            let mut context = ArchiveExtractionContext {
                plan,
                destination_root,
                archive_index: index,
                archive_total: plans.len(),
                expected_total_bytes: required_bytes,
                cumulative_written: &mut cumulative_written,
                progress: &mut progress,
            };
            let summary = Self::extract_single_archive(&mut context)?;
            outcome.archives.push(summary);
        }

        if settings.verify_integrity {
            Self::verify_integrity(&plans, destination_root)?;
            outcome.verified = true;
        }

        save_manifest(&manifest_file, &manifest)?;
        emit_progress(&mut progress, ExtractionProgressStage::Completed)?;

        Ok(outcome)
    }

    /// Builds an extraction manifest describing the current installation state.
    fn build_manifest(&self, plans: &[ArchivePlan]) -> Result<ExtractionManifest, ExtractionError> {
        let mut entries = Vec::with_capacity(plans.len());
        for plan in plans {
            let (len_bytes, modified_seconds) = archive_metadata_signature(&plan.source)?;
            entries.push(ArchiveManifestEntry {
                relative_path: plan.descriptor.relative_path.to_owned(),
                len_bytes,
                modified_seconds,
            });
        }

        Ok(ExtractionManifest {
            version: MANIFEST_VERSION,
            installation_fingerprint: installation_fingerprint(self.installation),
            archives: entries,
        })
    }

    /// Estimates the number of bytes that will be written during extraction.
    fn estimate_required_bytes(plans: &[ArchivePlan]) -> Result<u64, ExtractionError> {
        let mut total = 0_u64;
        for plan in plans {
            let (_reader, zfs) = Self::open_archive(plan)?;
            for entry in &zfs.files {
                let entry_size = Self::entry_size_to_u64(entry, &plan.source)?;
                total = total.saturating_add(entry_size);
            }
        }

        Ok(total)
    }

    /// Opens and parses an archive from a plan.
    fn open_archive(plan: &ArchivePlan) -> Result<(BufReader<File>, ZfsFile), ExtractionError> {
        let file = File::open(&plan.source).map_err(|source| ExtractionError::Io {
            path: plan.source.clone(),
            source,
        })?;
        let mut reader = BufReader::new(file);
        let zfs = ZfsFile::from_stream(&mut reader).map_err(|error| ExtractionError::Archive {
            path: plan.source.clone(),
            message: error.to_string(),
        })?;

        Ok((reader, zfs))
    }

    /// Converts an archive entry size into the extractor's byte count type.
    fn entry_size_to_u64(entry: &ZfsEntry, source: &Path) -> Result<u64, ExtractionError> {
        u64::try_from(entry.size).map_err(|error| ExtractionError::Archive {
            path: source.to_path_buf(),
            message: format!(
                "Entry '{}' exceeds supported size limits: {error}",
                entry.name
            ),
        })
    }

    /// Computes the expected byte count for every entry in an archive.
    fn expected_archive_bytes(entries: &[ZfsEntry], source: &Path) -> Result<u64, ExtractionError> {
        entries.iter().try_fold(0_u64, |acc, entry| {
            let entry_size = Self::entry_size_to_u64(entry, source)?;
            Ok(acc.saturating_add(entry_size))
        })
    }

    /// Extracts a single archive, emitting progress updates as entries are processed.
    fn extract_single_archive(
        context: &mut ArchiveExtractionContext<'_, '_>,
    ) -> Result<ArchiveExtractionSummary, ExtractionError> {
        let target_dir = context.destination_root.join(&context.plan.relative_parent);
        fs::create_dir_all(&target_dir).map_err(|source| ExtractionError::Io {
            path: target_dir.clone(),
            source,
        })?;

        let (mut reader, zfs) = Self::open_archive(context.plan)?;
        let expected_bytes = Self::expected_archive_bytes(&zfs.files, &context.plan.source)?;

        emit_progress(
            context.progress,
            ExtractionProgressStage::ArchiveStarted {
                archive_index: context.archive_index,
                archive_total: context.archive_total,
                archive_path: context.plan.source.clone(),
                entry_count: zfs.files.len(),
                expected_bytes,
            },
        )?;

        let mut summary = ArchiveExtractionSummary {
            archive: context.plan.source.clone(),
            files_written: 0,
            bytes_written: 0,
            files_skipped: 0,
            bytes_skipped: 0,
        };

        for (entry_index, entry) in zfs.files.iter().enumerate() {
            let mut entry_context = EntryExtractionContext {
                reader: &mut reader,
                target_dir: &target_dir,
                entry,
                entry_index,
                entry_total: zfs.files.len(),
                archive: context,
                summary: &mut summary,
            };
            Self::extract_entry(&mut entry_context)?;
        }

        Ok(summary)
    }

    /// Extracts one archive entry and emits its progress update.
    fn extract_entry(
        context: &mut EntryExtractionContext<'_, '_, '_>,
    ) -> Result<(), ExtractionError> {
        let source = &context.archive.plan.source;
        let data =
            context
                .entry
                .read_data(context.reader)
                .map_err(|error| ExtractionError::Archive {
                    path: source.clone(),
                    message: error.to_string(),
                })?;
        let entry_bytes = Self::entry_size_to_u64(context.entry, source)?;
        let entry_name = context.entry.name.clone();
        let target_path = resolve_entry_path(context.target_dir, &entry_name, source)?;

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|io_error| ExtractionError::Io {
                path: parent.to_path_buf(),
                source: io_error,
            })?;
        }

        let processed_bytes = Self::write_entry_if_needed(
            &target_path,
            &data,
            entry_bytes,
            &entry_name,
            context.summary,
        )?;
        *context.archive.cumulative_written = context
            .archive
            .cumulative_written
            .saturating_add(processed_bytes);

        emit_progress(
            context.archive.progress,
            ExtractionProgressStage::EntryProcessed {
                archive_index: context.archive.archive_index,
                entry_index: context.entry_index,
                entry_total: context.entry_total,
                entry_name,
                entry_size: entry_bytes,
                written_bytes_total: *context.archive.cumulative_written,
                expected_total_bytes: context.archive.expected_total_bytes,
            },
        )
    }

    /// Writes an entry unless an existing file already matches its expected size.
    fn write_entry_if_needed(
        target_path: &Path,
        data: &[u8],
        entry_bytes: u64,
        entry_name: &str,
        summary: &mut ArchiveExtractionSummary,
    ) -> Result<u64, ExtractionError> {
        if target_path.exists() {
            let metadata = fs::metadata(target_path).map_err(|source| ExtractionError::Io {
                path: target_path.to_path_buf(),
                source,
            })?;
            if metadata.len() == entry_bytes {
                summary.files_skipped = summary.files_skipped.saturating_add(1);
                summary.bytes_skipped = summary.bytes_skipped.saturating_add(entry_bytes);
                return Ok(entry_bytes);
            }
        }

        let mut output = File::create(target_path).map_err(|source| ExtractionError::Io {
            path: target_path.to_path_buf(),
            source,
        })?;
        output
            .write_all(data)
            .map_err(|source| ExtractionError::Io {
                path: target_path.to_path_buf(),
                source,
            })?;
        output.flush().map_err(|source| ExtractionError::Io {
            path: target_path.to_path_buf(),
            source,
        })?;

        let actual_size = fs::metadata(target_path)
            .map_err(|source| ExtractionError::Io {
                path: target_path.to_path_buf(),
                source,
            })?
            .len();

        if actual_size != entry_bytes {
            return Err(ExtractionError::Archive {
                path: target_path.to_path_buf(),
                message: format!("Post-write size mismatch for entry '{entry_name}'"),
            });
        }

        summary.files_written = summary.files_written.saturating_add(1);
        summary.bytes_written = summary.bytes_written.saturating_add(actual_size);
        Ok(actual_size)
    }

    /// Verifies that every file described by the archives exists at the destination.
    fn verify_integrity(
        plans: &[ArchivePlan],
        destination_root: &Path,
    ) -> Result<(), ExtractionError> {
        for plan in plans {
            let (_reader, zfs) = Self::open_archive(plan)?;
            let target_dir = destination_root.join(&plan.relative_parent);

            for entry in &zfs.files {
                let target_path = resolve_entry_path(&target_dir, &entry.name, &plan.source)?;
                let metadata =
                    fs::metadata(&target_path).map_err(|source| ExtractionError::Archive {
                        path: target_path.clone(),
                        message: format!("Expected extracted file missing ({source})"),
                    })?;
                let entry_size = Self::entry_size_to_u64(entry, &target_path)?;

                if metadata.len() != entry_size {
                    return Err(ExtractionError::Archive {
                        path: target_path,
                        message: format!("Extracted file size mismatch for entry '{}'", entry.name),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameVersion;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::fs::{self, File};
    use std::io::{Seek, Write};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn create_installation(version: GameVersion) -> (GameInstallation, TempDir) {
        create_installation_with_name(version, "Test")
    }

    fn create_installation_with_name(
        version: GameVersion,
        display_name: &str,
    ) -> (GameInstallation, TempDir) {
        let temp = TempDir::new().expect("temp dir");
        let installation = GameInstallation::new(
            temp.path().to_path_buf(),
            version,
            display_name.to_owned(),
            None,
        );
        (installation, temp)
    }

    fn write_le32(writer: &mut File, value: u32) {
        writer
            .write_u32::<LittleEndian>(value)
            .expect("write little-endian u32");
    }

    fn write_padding(file: &mut File, padding: u64) {
        let padding_len = usize::try_from(padding).expect("padding fits in usize");
        file.write_all(&vec![0_u8; padding_len])
            .expect("write padding");
    }

    fn write_sample_zfs(path: &Path, entry_name: &str, data: &[u8]) {
        const MAX_FILENAME_LEN: u32 = 260;
        const FILETABLE_OFFSET: u32 = 64;

        let mut file = File::create(path).expect("create zfs");

        write_le32(&mut file, unciv::ZFS_SIGNATURE);
        write_le32(&mut file, 1);
        write_le32(&mut file, MAX_FILENAME_LEN);
        write_le32(&mut file, 1);
        write_le32(&mut file, 1);
        write_le32(&mut file, 0);
        write_le32(&mut file, FILETABLE_OFFSET);

        let filetable_offset = u64::from(FILETABLE_OFFSET);
        let current = file.stream_position().expect("read zfs header position");
        if current < filetable_offset {
            let padding = filetable_offset
                .checked_sub(current)
                .expect("current position precedes file table");
            write_padding(&mut file, padding);
        }

        write_le32(&mut file, 0); // next table offset

        let filename_len = usize::try_from(MAX_FILENAME_LEN).expect("filename length fits usize");
        let mut name_block = vec![0_u8; filename_len];
        let name_bytes = entry_name.as_bytes();
        assert!(
            name_bytes.len() < name_block.len(),
            "entry name must fit in sample ZFS header"
        );
        name_block[..name_bytes.len()].copy_from_slice(name_bytes);
        file.write_all(&name_block).expect("write entry name block");

        let data_offset = filetable_offset
            .checked_add(4)
            .and_then(|value| value.checked_add(u64::from(MAX_FILENAME_LEN)))
            .and_then(|value| value.checked_add(20))
            .expect("sample ZFS data offset fits u64");
        write_le32(
            &mut file,
            u32::try_from(data_offset).expect("sample data offset fits u32"),
        );
        write_le32(&mut file, 0); // unk3
        write_le32(
            &mut file,
            u32::try_from(data.len()).expect("sample payload length fits u32"),
        );
        write_le32(&mut file, 0); // timestamp
        write_le32(&mut file, 0); // flags

        let after_entry = file.stream_position().expect("read file table position");
        if after_entry < data_offset {
            let padding = data_offset
                .checked_sub(after_entry)
                .expect("entry metadata precedes payload");
            write_padding(&mut file, padding);
        }

        file.write_all(data).expect("write sample payload");
    }

    fn populate_sample_archives(
        root: &Path,
        version: GameVersion,
    ) -> Vec<(PathBuf, PathBuf, Vec<u8>)> {
        let mut expectations = Vec::new();
        for (index, descriptor) in required_archives(version).iter().enumerate() {
            let archive_path = root.join(descriptor.relative_path);
            if let Some(parent) = archive_path.parent() {
                fs::create_dir_all(parent).expect("create archive parent");
            }

            let entry_path = PathBuf::from(format!("entries/{index}/payload.bin"));
            let payload = format!("payload-{index}").into_bytes();
            write_sample_zfs(
                &archive_path,
                entry_path.to_str().expect("utf8 entry"),
                &payload,
            );

            let relative_parent = Path::new(descriptor.relative_path)
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default();

            expectations.push((relative_parent, entry_path, payload));
        }
        expectations
    }

    #[test]
    fn validate_reports_missing_archives() {
        let (installation, _temp) = create_installation(GameVersion::CTP2);
        let extractor = ArchiveExtractor::new(&installation);

        let result = extractor.validate();
        assert!(matches!(
            result,
            Err(ExtractionError::MissingArchives { .. })
        ));
    }

    #[test]
    fn extraction_writes_expected_files() {
        let (installation, temp) = create_installation(GameVersion::CTP1);

        let expectations = populate_sample_archives(temp.path(), GameVersion::CTP1);
        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");
        let outcome = extractor.extract(dest.path()).expect("extraction succeeds");

        assert_eq!(outcome.archives.len(), expectations.len());
        for (summary, expectation) in outcome.archives.iter().zip(expectations.iter()) {
            assert_eq!(summary.files_written, 1);
            assert_eq!(summary.files_skipped, 0);
            assert_eq!(
                summary.bytes_written,
                u64::try_from(expectation.2.len()).expect("payload length fits u64")
            );
            assert_eq!(summary.bytes_skipped, 0);
        }

        for (relative_parent, entry_path, payload) in expectations {
            let extracted = dest.path().join(&relative_parent).join(&entry_path);
            let data = fs::read(&extracted).expect("read extracted payload");
            assert_eq!(data, payload);
        }
    }

    #[test]
    fn extraction_persists_manifest_for_completed_run() {
        let (installation, temp) = create_installation(GameVersion::CTP1);
        populate_sample_archives(temp.path(), GameVersion::CTP1);
        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");

        extractor.extract(dest.path()).expect("extraction succeeds");

        let manifest_data = fs::read(manifest_path(dest.path())).expect("read manifest");
        let manifest: ExtractionManifest =
            serde_json::from_slice(&manifest_data).expect("parse manifest");

        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert_eq!(
            manifest.archives.len(),
            required_archives(GameVersion::CTP1).len()
        );
        assert_eq!(
            manifest.installation_fingerprint,
            installation_fingerprint(&installation)
        );
    }

    #[test]
    fn extraction_preserves_source_archives() {
        let (installation, temp) = create_installation(GameVersion::CTP2);
        populate_sample_archives(temp.path(), GameVersion::CTP2);
        let snapshot: Vec<(PathBuf, Vec<u8>)> = required_archives(GameVersion::CTP2)
            .iter()
            .map(|descriptor| {
                let archive_path = temp.path().join(descriptor.relative_path);
                (
                    archive_path.clone(),
                    fs::read(&archive_path).expect("read archive"),
                )
            })
            .collect();

        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");

        extractor.extract(dest.path()).expect("extraction succeeds");

        for (path, original) in snapshot {
            let data = fs::read(&path).expect("read source archive");
            assert_eq!(data, original);
        }
    }

    #[test]
    fn extraction_skips_when_manifest_matches() {
        let (installation, temp) = create_installation(GameVersion::CTP1);
        let expectations = populate_sample_archives(temp.path(), GameVersion::CTP1);
        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");

        extractor
            .extract(dest.path())
            .expect("first extraction succeeds");

        let mut stages = Vec::new();
        let mut progress = |event: ExtractionProgressEvent| {
            stages.push(event.stage);
            ProgressControl::Continue
        };

        let outcome = extractor
            .extract_with_settings(
                dest.path(),
                ExtractionSettings::default(),
                Some(&mut progress),
            )
            .expect("second extraction succeeds");

        assert!(outcome.already_up_to_date);
        assert!(outcome.archives.is_empty());
        assert!(outcome.verified);
        assert!(
            stages
                .iter()
                .any(|stage| matches!(stage, &ExtractionProgressStage::UpToDate)),
            "up-to-date progress stage should be emitted"
        );

        // Ensure previously extracted files remain intact
        for (relative_parent, entry_path, payload) in expectations {
            let extracted = dest.path().join(&relative_parent).join(&entry_path);
            assert_eq!(
                fs::read(extracted).expect("read extracted payload"),
                payload
            );
        }
    }

    #[test]
    fn forced_reextract_counts_skipped_files() {
        let (installation, temp) = create_installation(GameVersion::CTP2);
        let expectations = populate_sample_archives(temp.path(), GameVersion::CTP2);
        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");

        extractor
            .extract(dest.path())
            .expect("initial extraction succeeds");

        let settings = ExtractionSettings {
            skip_if_up_to_date: false,
            verify_integrity: false,
        };

        let outcome = extractor
            .extract_with_settings(dest.path(), settings, None)
            .expect("forced re-extraction succeeds");

        assert_eq!(outcome.archives.len(), expectations.len());
        for summary in outcome.archives {
            assert_eq!(summary.files_written, 0);
            assert_eq!(summary.files_skipped, 1);
            assert_eq!(summary.bytes_written, 0);
            assert!(summary.bytes_skipped > 0);
        }
    }

    #[test]
    fn extraction_fails_with_insufficient_disk_space() {
        let (installation, temp) = create_installation(GameVersion::CTP1);
        populate_sample_archives(temp.path(), GameVersion::CTP1);
        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");

        set_disk_space_override(Some(0));
        let result = extractor.extract(dest.path());
        set_disk_space_override(None);

        assert!(matches!(result, Err(ExtractionError::DiskSpace { .. })));
    }

    #[test]
    fn progress_callback_can_abort_extraction() {
        let (installation, temp) = create_installation(GameVersion::CTP1);
        populate_sample_archives(temp.path(), GameVersion::CTP1);
        let extractor = ArchiveExtractor::new(&installation);
        let dest = TempDir::new().expect("destination temp dir");

        let mut progress = |event: ExtractionProgressEvent| match event.stage {
            ExtractionProgressStage::ArchiveStarted { .. } => ProgressControl::Abort {
                message: "user stop".to_owned(),
            },
            _ => ProgressControl::Continue,
        };

        let result = extractor.extract_with_settings(
            dest.path(),
            ExtractionSettings::default(),
            Some(&mut progress),
        );

        let error = result.expect_err("progress callback should abort extraction");
        assert!(
            matches!(
                error,
                ExtractionError::Aborted { message } if message == "user stop"
            ),
            "expected user stop abort error"
        );
    }

    #[test]
    fn default_destination_root_includes_sanitized_name() {
        let (installation, _temp) =
            create_installation_with_name(GameVersion::CTP1, "Call: To Power!");

        let destination = default_destination_root(&installation).expect("path");
        let last_component = destination
            .components()
            .next_back()
            .and_then(|c| c.as_os_str().to_str())
            .expect("last component");

        assert!(
            last_component.starts_with("call-to-power"),
            "sanitized component missing: {last_component}"
        );
        assert!(
            last_component
                .split('-')
                .next_back()
                .expect("fingerprint component")
                .len()
                == 16,
            "fingerprint should be 16 hex chars"
        );
    }
}
