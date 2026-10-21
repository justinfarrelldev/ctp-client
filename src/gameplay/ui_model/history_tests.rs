//! Only actual own completed-turn samples may become graph points.
use super::{MenuRect, history};
use crate::gameplay::server::v2::Statistics;

#[test]
fn graph_uses_real_turns_and_values_without_fabricated_foreign_or_missing_samples() {
    let bounds = MenuRect {
        x: 10,
        y: 20,
        width: 101,
        height: 51,
    };
    let samples = vec![
        Statistics {
            turn: 2,
            score: 0,
            population: 10,
            ..Statistics::default()
        },
        Statistics {
            turn: 6,
            score: 100,
            population: 5,
            ..Statistics::default()
        },
    ];
    assert_eq!(
        history::points(&samples, "score", bounds),
        vec![(10_i32, 70_i32), (110_i32, 20_i32)],
        "actual endpoint samples"
    );
    assert_eq!(
        history::points(&[], "score", bounds),
        Vec::<(i32, i32)>::new(),
        "legacy empty history has no invented zero graph"
    );
    assert_eq!(
        history::points(&samples, "population", bounds).len(),
        2,
        "same real sampled turns"
    );
    assert!(
        history::points(&samples, "foreign-rank", bounds).is_empty(),
        "unknown/private metric stays unavailable"
    );
}
