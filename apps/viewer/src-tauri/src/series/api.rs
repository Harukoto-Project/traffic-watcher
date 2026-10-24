use monitor_storage::Storage;

use super::model::SeriesResponse;

const BUCKET_STEPS_MS: &[i64] = &[
    1_000, 2_000, 5_000, 10_000, 15_000, 30_000, 60_000, 120_000, 300_000, 600_000, 900_000,
    1_800_000, 3_600_000,
];

pub fn pick_bucket_ms(since_ms: i64, until_ms: i64, max_points: i64) -> i64 {
    let span = (until_ms - since_ms).max(1);
    let max_points = max_points.max(1);
    BUCKET_STEPS_MS
        .iter()
        .copied()
        .find(|step| span / step <= max_points)
        .unwrap_or(BUCKET_STEPS_MS[BUCKET_STEPS_MS.len() - 1])
}

pub fn load(
    storage: &Storage,
    since_ms: i64,
    until_ms: i64,
    max_points: i64,
) -> anyhow::Result<SeriesResponse> {
    let bucket_ms = pick_bucket_ms(since_ms, until_ms, max_points);
    Ok(SeriesResponse {
        bucket_ms,
        ping: storage.ping_series(since_ms, until_ms, bucket_ms)?,
        system: storage.system_series(since_ms, until_ms, bucket_ms)?,
    })
}

#[cfg(test)]
mod tests {
    use super::pick_bucket_ms;

    #[test]
    fn bucket_grows_with_span() {
        assert_eq!(pick_bucket_ms(0, 600_000, 600), 1_000);
        assert_eq!(pick_bucket_ms(0, 3_600_000, 600), 10_000);
        assert_eq!(pick_bucket_ms(0, 86_400_000, 600), 300_000);
        assert_eq!(pick_bucket_ms(0, 365 * 86_400_000, 600), 3_600_000);
    }

    #[test]
    fn degenerate_inputs_are_clamped() {
        assert_eq!(pick_bucket_ms(5, 5, 600), 1_000);
        assert_eq!(pick_bucket_ms(0, 1_000_000, 0), 600_000);
    }
}
