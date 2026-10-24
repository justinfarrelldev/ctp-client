//! Startup defaults/error reproduction against read-only retail references, never live calls.
use super::{AI_COUNTRIES,rejection::Rejection};
use crate::gameplay::assets::layout::LayoutBook;
use alloc::collections::BTreeSet;
use std::{fs,path::Path};

/// Actual original IDs/localized aliases, not a copied compatibility table.
fn retail_aliases()->BTreeSet<String> {
    let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let source=fs::read_to_string(root.join("ctp2_data/default/gamedata/civilisation.txt")).expect("read-only original civilization identifiers");let book=LayoutBook::load(&root).expect("original localized alias references");let mut aliases=BTreeSet::new();
    for line in source.lines().filter(|line|line.chars().next().is_some_and(|value|value.is_ascii_uppercase())) {
        let id=line.split('#').next().unwrap_or_default().trim();if id.is_empty()||!id.bytes().all(|value|value.is_ascii_uppercase()||value==b'_'){continue;}
        aliases.insert(id.to_ascii_lowercase());for suffix in ["COUNTRY_NAME","SINGULAR","PLURAL"] {let key=format!("{id}_{suffix}");let label=book.label(&key);if label!=key {aliases.insert(label.to_ascii_lowercase());}}
    }
    aliases
}
/// Accurate isolated startup gateway, including bare setup errors and admission retries.
mod gateway;
#[test]
fn launch_recovery_default_ai_countries_are_real_original_retail_catalog_aliases() {
    let aliases=retail_aliases();
    for country in AI_COUNTRIES {assert!(aliases.contains(&country.to_ascii_lowercase()),"bootstrap country {country} is absent in released retail; server start_game rejects the whole setup instead of inventing a civilization");}
}
#[test]
fn launch_recovery_preserves_safe_bare_setup_rejection_and_middleware_rate_status() {
    let rejected=Rejection::decode(400_u16,"selected country is not available in the pinned retail civilization catalog");assert_eq!(rejected.code,"setup_country_unavailable","actual game/start_game handler writes a safe bare validation sentence, not gateway JSON");assert!(rejected.message.contains("civilization"),"diagnostic must preserve known safe cause");
    let busy=Rejection::decode(429_u16,r#"{"status":"Request Failed","body":"The API is at capacity, try again later."}"#);assert_eq!(busy.code,"rate_limited","admission exhaustion is not an opaque gameplay rejection");
    let private=Rejection::decode(400_u16,"SQL private_password=do-not-display");assert!(!private.label().contains("private_password"),"never show arbitrary legacy bodies");
}
