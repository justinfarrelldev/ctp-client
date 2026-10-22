//! Read-only installed picture indexes for optional source-revision artwork.
use crate::gameplay::{GameplayError, asset_error, assets::layout::LayoutBook};
use alloc::collections::BTreeSet;
use std::{fs::File, io::BufReader};
use unciv::ZfsFile;
/// Inspected during manager construction, never while rendering a frame.
/// Missing images differ from corrupt/unreadable archives; no assets are copied.
pub(super) fn pictures(book: &LayoutBook) -> Result<Option<BTreeSet<String>>, GameplayError> {
    let Some(root) = book.installation_root() else {
        return Ok(None);
    };
    let mut names = BTreeSet::new();
    for archive in [
        crate::gameplay::assets::PICTURE_ARCHIVE,
        "ctp2_data/default/graphics/pictures/pic555.zfs",
    ] {
        let path = root.join(archive);
        if !path
            .try_exists()
            .map_err(|error| asset_error(error.to_string()))?
        {
            continue;
        }
        let file = File::open(&path)
            .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
        let mut reader = BufReader::new(file);
        let index = ZfsFile::from_stream(&mut reader)
            .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
        names.extend(
            index
                .files
                .into_iter()
                .map(|entry| entry.name.to_ascii_lowercase()),
        );
    }
    Ok(Some(names))
}
