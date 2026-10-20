//! Only source-verified runtime fields/effects may resolve original expressions.

use super::variables::resolve;
use crate::gameplay::model::GameState;
use serde_json::json;

/// Numbers are synthetic; no retail rule values or assets are embedded.
fn fixture() -> GameState {
    let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    state.viewer=Some(serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"rules":{
        "units":[{"id":"UNIT_TEST","attack":13_i64,"production_cost":71_i64,"movement":2_i64,"max_fuel":2400_i64}],
        "production":[{"id":"IMPROVE_TEST","cost":123_i64,"upkeep":0_i64,"effects":[{"kind":"yield_food_percent","value":23_i64},{"kind":"happiness","value":-2_i64},{"kind":"happiness","value":5_i64},{"kind":"crime_percent","value":-17_i64}]},
            {"id":"WONDER_TEST","effects":[{"kind":"crime_percent","value":-31_i64},{"kind":"increase_brokerages","value":19_i64}]}],
        "governments":[{"id":"GOVERNMENT_TEST","too_many_cities_threshold":9_i64,"max_science_percent":75_i64}]
    }})).expect("viewer metadata"));
    state
}

#[test]
fn mapped_scalar_and_percent_effects_use_current_server_semantics_not_retail_values() {
    let state = fixture();
    for (id, expression, expected) in [
        ("UNIT_TEST", "UnitDB(UnitRecord[0]).Attack / 100", "13"),
        ("UNIT_TEST", "UnitDB(UnitRecord[0]).ShieldCost", "71"),
        ("IMPROVE_TEST", "BuildingDB(Building[0]).FoodPercent", "23"),
        ("IMPROVE_TEST", "BuildingDB(Building[0]).HappyInc", "3"),
        ("IMPROVE_TEST", "BuildingDB(Building[0]).LowerCrime", "-17"),
        ("IMPROVE_TEST", "BuildingDB(Building[0]).Upkeep", "0"),
        ("WONDER_TEST", "WonderDB(Wonder[0]). DecCrimePercent", "31"),
        (
            "WONDER_TEST",
            "WonderDB(Wonder[0]).IncreaseBrokerages",
            "19",
        ),
        (
            "GOVERNMENT_TEST",
            "GovernmentDB(Government[0]).MaxScienceRate",
            "75",
        ),
        (
            "GOVERNMENT_TEST",
            "GovernmentDB(Government[0]).TooManyCitiesThreshold",
            "9",
        ),
    ] {
        assert_eq!(
            resolve(&state, id, expression),
            expected,
            "verified expression {expression}"
        );
    }
}

#[test]
fn unknown_scales_ranks_missing_effects_and_overflow_never_invent_values() {
    let mut state = fixture();
    for (id, expression) in [
        ("UNIT_TEST", "UnitDB(UnitRecord[0]).MaxMovePoints / 10000"),
        ("UNIT_TEST", "UnitDB(UnitRecord[0]).MaxFuel / 100"),
        (
            "GOVERNMENT_TEST",
            "GovernmentDB(Government[0]).MilitaryRank",
        ),
        ("IMPROVE_TEST", "BuildingDB(Building[0]).SciencePercent"),
        ("IMPROVE_TEST", "arbitrary_script('/tmp/anything')"),
        ("MISSING", "BuildingDB(Building[0]).ProductionCost"),
    ] {
        assert_eq!(
            resolve(&state, id, expression),
            "—",
            "unsupported/ambiguous expression {expression}"
        );
    }
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .rules
        .as_mut()
        .expect("rules")["production"][0_usize]["effects"] =
        json!([{"kind":"happiness","value":i64::MAX},{"kind":"happiness","value":1_i64}]);
    assert_eq!(
        resolve(&state, "IMPROVE_TEST", "BuildingDB(Building[0]).HappyInc"),
        "—",
        "overflow is not a fabricated saturated value"
    );
}
