//! Repaired server append contract; public content existence is not city eligibility.
use super::{ServerCityProjection, ServerProjection, commands};
use alloc::collections::BTreeSet;
use serde_json::Value;

/// Complete atomic-append choices bound to the same current city/rules snapshot.
/// Absence, partial/unavailable, malformed rows and mismatches fail closed as a whole.
#[must_use]
pub(in crate::gameplay) fn items<'view>(viewer:&'view ServerProjection,city:&'view ServerCityProjection)->Option<&'view Vec<Value>> {
    if !viewer.interface.supports("production_options") || city.stale || !viewer.interface.legal_actions.iter().any(|action|action.route=="edit-production"&&action.entity_id==city.id) { return None; }
    let query=&city.facts["production_options"];
    let hash=viewer.interface.rules.as_ref()?["content_hash"].as_str().filter(|hash|!hash.is_empty())?;
    if query["revision"].as_i64()!=Some(viewer.revision)||query["city_id"].as_str()!=Some(city.id.as_str())||query["content_hash"].as_str()!=Some(hash)||query["operation"].as_str()!=Some("append")||query["status"].as_str()!=Some("complete") {return None;}
    let items=query["items"].as_array()?;
    let mut ids=BTreeSet::new();
    for item in items {
        let id=item["id"].as_str().filter(|id|!id.is_empty())?;
        if !ids.insert(id)||!commands::known_content(viewer,id)||!matches!(item["kind"].as_str(),Some("unit"|"building"|"wonder"|"capitalization"|"infrastructure"))||item["cost"].as_i64().is_none_or(|cost|cost<1_i64)||item.get("entry_id").is_some(){return None;}
    }
    Some(items)
}

/// New city candidate at this revision; legacy servers authorize only explicit actions.
#[must_use]
pub(in crate::gameplay) fn permits(viewer:&ServerProjection,city:&ServerCityProjection,id:&str)->bool {
    if viewer.interface.supports("production_options") {
        items(viewer,city).is_some_and(|items|items.iter().any(|item|item["id"].as_str()==Some(id)))
    }else{
        !city.stale&&viewer.interface.legal_actions.iter().any(|action|action.route=="queue-production"&&action.entity_id==city.id&&action.target_id==id&&action.payload["city_id"].as_str()==Some(city.id.as_str())&&action.payload["item"]["id"].as_str()==Some(id))
    }
}
