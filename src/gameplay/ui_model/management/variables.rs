//! Verified original-expression names mapped to current public runtime semantics.
//!
//! SlicDBConduit multiplies float getters by 100, while the server publishes
//! normalized attack/defense/percent effects. These mappings do not evaluate SLIC
//! or reconstruct source rules. Movement/fuel scales and legacy government ranks
//! are deliberately unavailable without a sufficient authoritative contract.

use crate::gameplay::model::GameState;
use serde_json::Value;

/// Explicit scalar/effect mapping, not a general expression language.
#[derive(Debug, Clone, Copy)]
enum Binding {
    /// Exact runtime category and field.
    Field(&'static str, &'static str),
    /// Sum of a public effect kind, optionally negated for an inverse source label.
    Effect(&'static str, bool),
}

/// Resolves only verified mappings; zero is displayed only when actually present.
pub(super) fn resolve(state: &GameState, id: &str, expression: &str) -> String {
    let normalized = expression.replace(' ', "");
    let value = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.rules.as_ref())
        .and_then(|rules| match binding(&normalized)? {
            Binding::Field(category, field) => definition(rules, category, id)?[field].as_i64(),
            Binding::Effect(kind, negate) => {
                let value = effect_sum(definition(rules, "production", id)?, kind)?;
                if negate {
                    value.checked_neg()
                } else {
                    Some(value)
                }
            }
        });
    value.map_or_else(|| "—".to_owned(), |number| number.to_string())
}

/// Exact database namespaces prevent unrelated definitions from filling a field.
fn binding(expression: &str) -> Option<Binding> {
    if expression == "AdvanceDB(Advance[0]).Cost" {
        return Some(Binding::Field("advances", "cost"));
    }
    if let Some(property) = expression.strip_prefix("UnitDB(UnitRecord[0]).") {
        return unit(property).map(|field| Binding::Field("units", field));
    }
    if let Some(property) = expression.strip_prefix("GovernmentDB(Government[0]).") {
        let field = match property {
            "MaxScienceRate" => "max_science_percent",
            "TooManyCitiesThreshold" => "too_many_cities_threshold",
            _ => return None,
        };
        return Some(Binding::Field("governments", field));
    }
    if let Some(property) = expression.strip_prefix("BuildingDB(Building[0]).") {
        return match property {
            "ProductionCost" => Some(Binding::Field("production", "cost")),
            "Upkeep" => Some(Binding::Field("production", "upkeep")),
            _ => building(property).map(|kind| Binding::Effect(kind, false)),
        };
    }
    if let Some(property) = expression.strip_prefix("WonderDB(Wonder[0]).") {
        return match property {
            "ProductionCost" => Some(Binding::Field("production", "cost")),
            "DecCrimePercent" => Some(Binding::Effect("crime_percent", true)),
            _ => wonder(property).map(|kind| Binding::Effect(kind, false)),
        };
    }
    None
}

/// Raw runtime unit fields already undo the original SLIC float-getter scaling.
fn unit(property: &str) -> Option<&'static str> {
    match property {
        "Attack/100" => Some("attack"),
        "Defense/100" => Some("defense"),
        "Armor/100" => Some("armor"),
        "ZBRangeAttack" => Some("ranged"),
        "Firepower" => Some("firepower"),
        "VisionRange" => Some("visibility_range"),
        "ShieldCost" => Some("production_cost"),
        _ => None,
    }
}

/// retailBuildingRuntimeEffects exposes actual percent points, including signs.
fn building(property: &str) -> Option<&'static str> {
    match property {
        "CommercePercent" => Some("yield_gold_percent"),
        "FoodPercent" => Some("yield_food_percent"),
        "ProductionPercent" => Some("yield_production_percent"),
        "SciencePercent" => Some("research_percent"),
        "GoldPerCitizen" => Some("yield_gold_per_population"),
        "HappyInc" => Some("happiness"),
        "LowerCrime" => Some("crime_percent"),
        "LowerPeaceMovement" => Some("lower_peace_movement_percent"),
        "PopulationPollutionPercent" => Some("pollution_population_percent"),
        "ProductionPollutionPercent" => Some("pollution_production_percent"),
        "DefendersPercent/100" => Some("combat_defense"),
        "OffenseBonusAir/100" => Some("combat_attack_air"),
        "OffenseBonusLand/100" => Some("combat_attack_land"),
        "OffenseBonusWater/100" => Some("combat_attack_sea"),
        _ => None,
    }
}

/// Explicit canonical wonder effect kinds, not inferred source numeric constants.
fn wonder(property: &str) -> Option<&'static str> {
    match property {
        "GoldPerInternationalTradeRoute" => Some("gold_per_international_trade_route"),
        "GoldPerWaterTradeRoute" => Some("gold_per_water_trade_route"),
        "GoldPerTelevision" => Some("gold_per_television"),
        "IncHappinessEmpire" => Some("happiness"),
        "IncKnowledgePercent" => Some("research_percent"),
        "IncreaseBrokerages" => Some("increase_brokerages"),
        "IncreaseProduction" => Some("yield_production_percent"),
        "ReduceReadinessCost" => Some("reduce_readiness_cost"),
        _ => None,
    }
}

/// A missing record is unknown, not a zero-valued guessed default.
fn definition<'rules>(rules: &'rules Value, category: &str, id: &str) -> Option<&'rules Value> {
    rules[category]
        .as_array()?
        .iter()
        .find(|definition| definition["id"].as_str() == Some(id))
}

/// Matching effects sum with checked arithmetic; absence or malformed values fail closed.
fn effect_sum(definition: &Value, kind: &str) -> Option<i64> {
    let mut found = false;
    let mut sum = 0_i64;
    for effect in definition["effects"].as_array()? {
        if effect["kind"].as_str() == Some(kind) {
            sum = sum.checked_add(effect["value"].as_i64()?)?;
            found = true;
        }
    }
    found.then_some(sum)
}
