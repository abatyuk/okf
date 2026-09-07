//! Clock effect (for `stale_after` checks and timestamps).
pub trait Clock {
    /// RFC 3339 timestamp of "now".
    fn now_rfc3339(&self) -> String;
}

/// Production clock: UTC "now" formatted as RFC 3339 (`YYYY-MM-DDTHH:MM:SSZ`),
/// computed from the system clock with no external date dependency.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        rfc3339_utc(secs)
    }
}

/// A clock that always returns a fixed timestamp — for deterministic tests.
#[derive(Clone)]
pub struct FixedClock(pub String);

impl Clock for FixedClock {
    fn now_rfc3339(&self) -> String {
        self.0.clone()
    }
}

/// Format seconds-since-Unix-epoch as an RFC 3339 UTC timestamp.
fn rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, mo, d) = civil_from_days(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Days-since-1970-01-01 → (year, month, day). Howard Hinnant's `civil_from_days`.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_zero_formats_as_1970() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn known_timestamp() {
        // 2026-08-01T00:00:00Z == 1785542400 seconds.
        assert_eq!(rfc3339_utc(1_785_542_400), "2026-08-01T00:00:00Z");
    }
}
