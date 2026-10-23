//! Original saved-queue grammar and current pinned-content resolution, fileless.
use super::{Item, Template, codec, prompt, resolve};
use crate::gameplay::server::ServerProjection;
use serde_json::json;

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{Save,LoadCustom,
 * DisplayQueueContents} and gs/gameobj/bldque.cpp::{Save,Load}. U/B/W records
 * carry canonical names; CAP/INF are semantic specials. Do not port Save's
 * reversed specials, final-line truncation, empty-line indexing or silent bad
 * record skipping. Parsing never runs SLIC or supplies production costs/effects.
 */
#[test]
fn build_saved_codec_reads_original_records_crlf_comments_unicode_and_final_line_without_newline() {
    let parsed=codec::parse("# local template\r\nU UNIT_SETTLER\r\nB IMPROVE_GRANARY\r\nW WONDER_PYRAMIDS\r\nCAP\r\nINF").expect("original saved grammar");
    assert_eq!(parsed.items,vec![Item::Unit("UNIT_SETTLER".to_owned()),Item::Building("IMPROVE_GRANARY".to_owned()),Item::Wonder("WONDER_PYRAMIDS".to_owned()),Item::Capitalization,Item::Infrastructure],"no swapped CAP/INF or truncated final token");
    assert_eq!(codec::parse("\u{feff}U UNIT_Δ").expect("UTF-8 canonical ID").items,vec![Item::Unit("UNIT_Δ".to_owned())],"no lossy byte slicing");
}

#[test]
fn build_saved_codec_empty_is_explicit_and_malformed_records_never_become_a_partial_queue() {
    assert_eq!(codec::parse("\r\n # comment\r\n").expect("known empty file").items,vec![],"explicit empty isn't failed read");
    for text in ["U","X UNIT_SETTLER","CAP extra","INF extra","B ID extra","U A\0B","U ","U OK\ninvalid"] {
        assert!(codec::parse(text).is_err(),"reject entire invalid input {text:?}; never silently drop records before destructive load");
    }
}

#[test]
fn build_saved_codec_bounds_bytes_identifiers_and_item_count_without_truncation() {
    let mut maximum=core::iter::repeat_n("U UNIT_SETTLER\n",256_usize).collect::<String>();
    assert_eq!(codec::parse(&maximum).expect("maximum entries").items.len(),256_usize,"server atomic queue bound");
    maximum.push_str("U UNIT_SETTLER\n");
    assert!(codec::parse(&maximum).is_err(),"never truncate oversized template");
    assert!(codec::parse(&"#".repeat(codec::MAX_BYTES.saturating_add(1_usize))).is_err(),"bounded source file");
    assert!(codec::parse(&format!("U {}","X".repeat(257_usize))).is_err(),"bounded canonical identifier");
}

/// Complete pinned definitions, no local cost or prerequisite simulation.
fn viewer() -> ServerProjection {
    serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"rules":{"units":[{"id":"UNIT_SETTLER"}],"production":[{"id":"UNIT_SETTLER","kind":"unit"},{"id":"IMPROVE_GRANARY","kind":"building"},{"id":"WONDER_PYRAMIDS","kind":"wonder"},{"id":"CAPITALIZATION","kind":"capitalization"},{"id":"INFRASTRUCTURE","kind":"infrastructure"}]}})).expect("pinned rule schema")
}

#[test]
fn build_saved_resolve_preserves_exact_pinned_ids_kind_and_duplicate_new_items() {
    let parsed=codec::parse("U UNIT_SETTLER\nU UNIT_SETTLER\nB IMPROVE_GRANARY\nW WONDER_PYRAMIDS\nCAP\nINF").expect("template");
    assert_eq!(resolve(&viewer(),&parsed).expect("current kinds"),vec!["UNIT_SETTLER","UNIT_SETTLER","IMPROVE_GRANARY","WONDER_PYRAMIDS","CAPITALIZATION","INFRASTRUCTURE"],"new item duplicates are deliberate; no cost/effect metadata");
}

#[test]
fn build_saved_resolve_unknown_wrong_kind_ambiguous_special_and_missing_rules_fail_closed() {
    for text in ["U MISSING","B UNIT_SETTLER","U IMPROVE_GRANARY"] {
        assert!(resolve(&viewer(),&codec::parse(text).expect("syntactically valid")).is_err(),"no client substitutions or category guesses");
    }
    let mut ambiguous=viewer();ambiguous.interface.rules.as_mut().expect("rules")["production"].as_array_mut().expect("production").push(json!({"id":"OTHER_CAPITALIZATION","kind":"capitalization"}));
    assert!(resolve(&ambiguous,&codec::parse("CAP").expect("special")).is_err(),"must not pick first semantic definition");
    ambiguous.interface.rules=None;
    assert!(resolve(&ambiguous,&codec::parse("U UNIT_SETTLER").expect("template")).is_err(),"missing definitions aren't permissive");
    assert!(resolve(&ambiguous,&Template {items:Vec::new()}).is_ok_and(|items|items.is_empty()),"explicit empty does not require fabricated definitions");
}

#[test]
fn build_saved_prompt_substitutes_only_exact_localized_string_slots_without_printf_or_slic() {
    assert_eq!(prompt("File %s / city %s / %%",&["日本%s","Rome"]),Some("File 日本%s / city Rome / %".to_owned()),"inserted strings aren't evaluated");
    for format in ["%n","%s %s %s","%s","trailing %"] {assert!(prompt(format,&["a","b"]).is_none(),"unsupported/mismatched format rejected");}
}

#[test]
fn build_saved_replacement_explicit_empty_funded_head_unknown_quote_and_total_limit_are_not_conflated() {
    let rules=viewer().interface.rules.expect("pinned definitions");
    let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c2","owner_id":"human","tile_id":"a","queue":[{"entry_id":"head","id":"UNIT_SETTLER"}],"rush_buy_quote":{"revision":7_i64,"status":"available","cost":30_i64}}],"rules":rules,"legal_actions":[{"route":"edit-production","entity_id":"c2","payload":{"city_id":"c2","entries":[]}}]})).expect("owner/current quote/action schema");
    let mut state=crate::gameplay::model::GameState::from_server_projection(&projection,1_u8,1_u64);
    let view=super::super::ManagerView {kind:super::super::ManagerKind::Build,city_id:Some("c2".to_owned()),selections:alloc::collections::BTreeMap::new(),scroll:alloc::collections::BTreeMap::new()};
    assert_eq!(super::super::build::load_replacement(&state,&view,&[]).expect("explicit empty load").payload["entries"],json!([]),"approved empty file may clear unfinished queue");
    let city=state.viewer.as_mut().expect("viewer").cities.first_mut().expect("one exact city");city.rush_buy_quote.as_mut().expect("quote").status="completed".to_owned();city.rush_buy_quote.as_mut().expect("quote").cost=Some(0_i64);
    assert_eq!(super::super::build::load_replacement(&state,&view,&[]).expect("funded empty load").payload["entries"],json!([{"entry_id":"head"}]),"funded head never cleared by empty template");
    assert!(super::super::build::load_replacement(&state,&view,&vec!["UNIT_SETTLER".to_owned();256_usize]).is_none(),"retained head plus256 cannot be silently truncated to server bound");
    state.viewer.as_mut().expect("viewer").cities.first_mut().expect("city").rush_buy_quote=None;
    assert!(super::super::build::load_replacement(&state,&view,&[]).is_none(),"missing funding never means unfinished");
    state.viewer.as_mut().expect("viewer").cities.first_mut().expect("city").facts["queue"]=json!([]);
    assert!(super::super::build::load_replacement(&state,&view,&["UNIT_SETTLER".to_owned()]).is_some(),"explicit no-head queue needs no fabricated quote");
}

#[test]
fn build_saved_duplicate_filename_sources_remain_distinct_and_visibly_identified_not_first_root_selected() {
    let mut ui=super::super::ManagementUi::default();
    for origin in ["profile","retail"] {
        let id=format!("{origin}:same");ui.saved_queues.files.insert(id.clone(),super::QueueFile::from_bytes(id,"same".to_owned(),b"U UNIT_SETTLER"));
    }
    let view=super::super::ManagerView {kind:super::super::ManagerKind::Build,city_id:None,selections:alloc::collections::BTreeMap::from([(super::MODE.to_owned(),"true".to_owned())]),scroll:alloc::collections::BTreeMap::new()};
    let rows=super::rows(&ui,&view,"LoadBox.QueuesList").expect("source file list");
    assert_eq!(rows,vec![("profile:same".to_owned(),"same [profile]".to_owned()),("retail:same".to_owned(),"same [retail]".to_owned())],"do not collapse opaque IDs or hide duplicate source");
}
