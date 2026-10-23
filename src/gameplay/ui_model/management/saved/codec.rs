//! Bounded original saved-build-queue text, never executable metadata or paths.
use super::{Item, Template};

/// One bounded file; larger input is rejected, never partially interpreted.
pub const MAX_BYTES: usize = 0x0001_0000_usize;
/// Complete atomic server queue limit, including any separately retained head.
pub const MAX_ITEMS: usize = 256_usize;

/*
 * Original source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{Save,
 * LoadCustom,DisplayQueueContents}; gs/gameobj/bldque.cpp::{Save,Load}.
 * U/B/W records use database IDs; CAP/INF represent capitalization/infrastructure.
 * Fix original custom Save's swapped specials, last-character removal without
 * a newline and blank-line indexing. Reject malformed whole templates instead
 * of silently skipping items before a destructive replacement. Parsing supplies
 * no costs, effects, buildability or authoritative entry IDs; UTF-8/ASCII only.
 */
/// Serializes all original categories, correcting original Save's swapped CAP/INF.
///
/// # Errors
/// Rejects oversized/malformed templates as a whole; never emits partial records.
pub fn encode(template: &Template) -> Result<String, String> {
    if template.items.len()>MAX_ITEMS { return Err("Saved queue exceeds 256 items".to_owned()); }
    let mut output=String::new();
    for item in &template.items {
        let (kind,id)=item.definition();
        let tag=match kind {"unit"=>"U","building"=>"B","wonder"=>"W","capitalization"=>"CAP","infrastructure"=>"INF",_=>return Err("Unknown template category".to_owned())};
        output.push_str(tag);
        if let Some(identifier)=id {
            if identifier.is_empty() || identifier.len()>256_usize || identifier.chars().any(|character|character.is_control()||character.is_whitespace()) {return Err("Saved queue contains an invalid original content ID".to_owned());}
            output.push(' ');output.push_str(identifier);
        }
        output.push('\n');
        if output.len()>MAX_BYTES {return Err("Saved queue exceeds the safe file-size limit".to_owned());}
    }
    Ok(output)
}

/// Parses original user template bytes already decoded losslessly as UTF-8.
///
/// # Errors
/// Rejects oversized files/IDs/queues, controls and any malformed record.
pub fn parse(input: &str) -> Result<Template, String> {
    if input.len() > MAX_BYTES { return Err("Saved queue exceeds the safe file-size limit".to_owned()); }
    if input.chars().any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t')) {
        return Err("Saved queue contains unsupported control characters".to_owned());
    }
    let text = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut items = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if items.len() >= MAX_ITEMS { return Err("Saved queue exceeds 256 items".to_owned()); }
        let mut fields = line.split_whitespace();
        let Some(tag) = fields.next() else { continue; };
        let invalid = || format!("Saved queue has an invalid record on line {}", index.saturating_add(1_usize));
        let item = match tag {
            "U" | "B" | "W" => {
                let id = fields.next().filter(|id| !id.is_empty() && id.len() <= 256_usize).ok_or_else(invalid)?;
                if tag == "U" { Item::Unit(id.to_owned()) } else if tag == "B" { Item::Building(id.to_owned()) } else { Item::Wonder(id.to_owned()) }
            }
            "CAP" => Item::Capitalization,
            "INF" => Item::Infrastructure,
            _ => return Err(invalid()),
        };
        if fields.next().is_some() { return Err(invalid()); }
        items.push(item);
    }
    Ok(Template { items })
}
