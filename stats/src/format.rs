//! Number formatting for the dashboard.
//!
//! Kept separate from the views so the arithmetic can be tested without a
//! browser: `cargo test` runs these on the host, where the wasm render cannot.

/// Compact token counts: 1_234 -> "1.23K", 8_205_016_357 -> "8.21B".
pub fn compact(n: u64) -> String {
    let n = n as f64;
    if n >= 1_000_000_000_000.0 {
        format!("{:.2}T", n / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000.0 {
        format!("{:.2}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.2}K", n / 1_000.0)
    } else {
        format!("{n:.0}")
    }
}

/// Group digits with commas: 93444 -> "93,444".
pub fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// One decimal place percent: 89.75 -> "89.8%".
pub fn pct(v: f64) -> String {
    format!("{v:.1}%")
}

/// A percentage with two decimals.
///
/// Uptime lives inside the last percent, so one decimal would round a 99.85%
/// day up to 99.9% and erase the difference between a normal day and a bad one.
pub fn pct2(v: f64) -> String {
    format!("{v:.2}%")
}

/// Milliseconds as a short duration: 5096 -> "5.1s", 420 -> "420ms".
pub fn uptime(ms: u64) -> String {
    if ms >= 10_000 {
        format!("{:.0}s", ms as f64 / 1000.0)
    } else if ms >= 1_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

/// Minutes between an RFC3339 timestamp and now, floored at zero.
///
/// A browser clock ahead of the server would otherwise produce a negative age
/// and the page would claim the snapshot came from the future.
pub fn age_minutes(generated_at: &str) -> i64 {
    age_minutes_at(generated_at, (js_sys::Date::now() / 1000.0) as i64)
}

/// The clock-free half of [`age_minutes`], so the clamp is testable on the host.
pub fn age_minutes_at(generated_at: &str, now_secs: i64) -> i64 {
    let then = match parse_rfc3339(generated_at) {
        Some(t) => t,
        None => return 0,
    };
    ((now_secs - then) / 60).max(0)
}

/// Render an RFC3339 timestamp as `YYYY-MM-DD HH:MM` in UTC.
pub fn stamp(generated_at: &str) -> String {
    match parse_rfc3339(generated_at) {
        Some(secs) => {
            let d = js_sys::Date::new(&(secs as f64 * 1000.0).into());
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                d.get_utc_full_year(),
                d.get_utc_month() + 1,
                d.get_utc_date(),
                d.get_utc_hours(),
                d.get_utc_minutes(),
            )
        }
        None => generated_at.to_string(),
    }
}

/// Seconds since the Unix epoch for a `YYYY-MM-DDTHH:MM:SSZ` timestamp.
///
/// Hand-rolled because the snapshot always carries exactly this shape and a
/// date library would be a dependency for four fields.
fn parse_rfc3339(s: &str) -> Option<i64> {
    let s = s.trim_end_matches('Z');
    let (date, time) = s.split_once('T')?;
    let mut dp = date.split('-');
    let y: i64 = dp.next()?.parse().ok()?;
    let mo: i64 = dp.next()?.parse().ok()?;
    let d: i64 = dp.next()?.parse().ok()?;
    let mut tp = time.split(':');
    let h: i64 = tp.next()?.parse().ok()?;
    let mi: i64 = tp.next()?.parse().ok()?;
    let se: i64 = tp.next().unwrap_or("0").split('.').next()?.parse().ok()?;
    Some(days_from_civil(y, mo, d) * 86_400 + h * 3_600 + mi * 60 + se)
}

/// Days since 1970-01-01 for a proleptic Gregorian date, from Howard Hinnant's
/// `days_from_civil`.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_scales_at_each_boundary() {
        assert_eq!(compact(0), "0");
        assert_eq!(compact(999), "999");
        assert_eq!(compact(1_000), "1.00K");
        assert_eq!(compact(1_234), "1.23K");
        assert_eq!(compact(1_000_000), "1.00M");
        assert_eq!(compact(8_205_016_357), "8.21B");
        assert_eq!(compact(64_641_258), "64.64M");
    }

    #[test]
    fn thousands_groups_every_three_digits() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(93_444), "93,444");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn pct_keeps_one_decimal() {
        assert_eq!(pct(0.0), "0.0%");
        assert_eq!(pct(89.75), "89.8%");
        assert_eq!(pct(100.0), "100.0%");
    }

    #[test]
    fn uptime_picks_a_readable_unit() {
        assert_eq!(uptime(0), "0ms");
        assert_eq!(uptime(420), "420ms");
        assert_eq!(uptime(1_000), "1.0s");
        assert_eq!(uptime(5_096), "5.1s");
        assert_eq!(uptime(40_000), "40s");
    }

    #[test]
    fn parse_rfc3339_matches_a_known_epoch() {
        // 2026-10-19T23:32:36Z
        assert_eq!(parse_rfc3339("2026-10-19T23:32:36Z"), Some(1_792_452_756));
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_rfc3339("2026-10-19T23:32:36.123Z"),
            Some(1_792_452_756)
        );
        assert_eq!(parse_rfc3339("not a date"), None);
    }

    #[test]
    fn a_timestamp_in_the_future_reads_as_zero_age() {
        // `age_minutes` reads the browser clock, so only the clamp is testable
        // here: a far-future stamp must not produce a negative age.
        let now = 1_792_452_756;
        assert_eq!(age_minutes_at("2999-01-01T00:00:00Z", now), 0);
        assert_eq!(age_minutes_at("2026-10-19T23:22:36Z", now), 10);
    }
}
