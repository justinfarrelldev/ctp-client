//! Overflow-safe coordinates from actual own-nation history, never foreign rankings.
use super::MenuRect;
use crate::gameplay::server::v2::Statistics;

/// Supported metrics deliberately exclude unavailable foreign comparisons.
pub const METRICS: [&str; 4] = ["score", "population", "cities", "units"];

/// Exact own sample field. Unsupported metrics cannot become a fake zero.
fn value(sample: &Statistics, metric: &str) -> Option<i128> {
    match metric {
        "score" => Some(i128::from(sample.score)),
        "population" => Some(i128::from(sample.population)),
        "cities" => Some(i128::from(sample.city_count)),
        "units" => Some(i128::from(sample.unit_count)),
        _ => None,
    }
}
/// Projects bounded real samples into the original `InfoGraph` rectangle.
#[must_use]
pub fn points(samples: &[Statistics], metric: &str, bounds: MenuRect) -> Vec<(i32, i32)> {
    if bounds.width < 2 || bounds.height < 2 || !METRICS.contains(&metric) {
        return Vec::new();
    }
    let data = samples
        .iter()
        .filter_map(|sample| value(sample, metric).map(|value| (i128::from(sample.turn), value)))
        .collect::<Vec<_>>();
    let Some(min_x) = data.iter().map(|sample| sample.0).min() else {
        return Vec::new();
    };
    let max_x = data.iter().map(|sample| sample.0).max().unwrap_or(min_x);
    let min_y = data.iter().map(|sample| sample.1).min().unwrap_or(0);
    let max_y = data.iter().map(|sample| sample.1).max().unwrap_or(min_y);
    let width = i128::from(bounds.width.saturating_sub(1));
    let height = i128::from(bounds.height.saturating_sub(1));
    data.into_iter()
        .filter_map(|(turn, value)| {
            let x = turn
                .checked_sub(min_x)?
                .checked_mul(width)?
                .checked_div(max_x.saturating_sub(min_x).max(1))?;
            let y = if max_y == min_y {
                height.div_euclid(2)
            } else {
                max_y
                    .checked_sub(value)?
                    .checked_mul(height)?
                    .checked_div(max_y.saturating_sub(min_y).max(1))?
            };
            Some((
                bounds.x.checked_add(i32::try_from(x).ok()?)?,
                bounds.y.checked_add(i32::try_from(y).ok()?)?,
            ))
        })
        .collect()
}
