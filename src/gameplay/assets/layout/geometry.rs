//! AUI intrinsic image sizing for controls without explicit LDL dimensions.

use alloc::collections::BTreeMap;
use std::path::Path;

use super::LayoutNode;
use crate::gameplay::GameplayError;
use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::rim::load_rim_rgba_entry;

/// Completes only unspecified image dimensions. Explicit pixel/percent sizes,
/// including zero, remain untouched. `aui_Control::ResizeLayers` derives these
/// extents from native image rectangles; using the parent's height creates huge
/// duplicate text boxes and inconsistent pointer bounds.
pub(super) fn complete(node: &mut LayoutNode, root: &Path) -> Result<(), GameplayError> {
    let mut cache = BTreeMap::new();
    size_node(node, root, &mut cache)
}

/// Cached native image extents, never stored or extracted onto disk.
type Dimensions = BTreeMap<String, (u32, u32)>;

/// Sizes a resolved node and its children with one bounded per-tree image cache.
fn size_node(
    node: &mut LayoutNode,
    root: &Path,
    cache: &mut Dimensions,
) -> Result<(), GameplayError> {
    let name = node.field("image00").or_else(|| node.field("image0"));
    if let Some(name) =
        name.filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("error.tga"))
    {
        let count = node.integer("imagesperlayer").unwrap_or(1_i32);
        let needs_height = node.field("heightpix").is_none() && node.field("heightpct").is_none();
        let needs_width = node.field("widthpix").is_none() && node.field("widthpct").is_none();
        if count != 9_i32 && (needs_height || (needs_width && count == 1_i32)) {
            let key = name.to_ascii_lowercase().replace(".tga", ".rim");
            let (width, height) = if let Some(size) = cache.get(&key) {
                *size
            } else {
                let image = load_rim_rgba_entry(root, PICTURE_ARCHIVE, &key, None)?;
                let size = (image.width, image.height);
                cache.insert(key, size);
                size
            };
            if needs_height {
                node.fields
                    .insert("heightpix".to_owned(), height.to_string());
            }
            if needs_width && count == 1_i32 {
                node.fields.insert("widthpix".to_owned(), width.to_string());
            }
        }
    }
    for child in &mut node.children {
        size_node(child, root, cache)?;
    }
    Ok(())
}
