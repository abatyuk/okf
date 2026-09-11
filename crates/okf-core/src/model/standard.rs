//! Typed scalar semantics from OKF v0.2.

/// A parsed timestamp normalized for chronological comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: u32,
}

/// Parse an OKF ISO 8601 datetime with an explicit `Z` or numeric UTC offset.
pub fn parse_timestamp(raw: &str) -> Option<Timestamp> {
    let (date, rest) = raw.trim().split_once('T')?;
    let mut d = date.split('-');
    let year = d.next()?.parse::<i32>().ok()?;
    let month = d.next()?.parse::<u32>().ok()?;
    let day = d.next()?.parse::<u32>().ok()?;
    if d.next().is_some() || !valid_date(year, month, day) {
        return None;
    }

    let (clock, offset_seconds) = if let Some(clock) = rest.strip_suffix('Z') {
        (clock, 0i64)
    } else {
        let pos = rest
            .char_indices()
            .skip(1)
            .find_map(|(i, c)| matches!(c, '+' | '-').then_some(i))?;
        let (clock, offset) = rest.split_at(pos);
        let sign = if offset.starts_with('+') { 1i64 } else { -1i64 };
        let (oh, om) = offset[1..].split_once(':')?;
        if oh.len() != 2 || om.len() != 2 {
            return None;
        }
        let oh = oh.parse::<i64>().ok()?;
        let om = om.parse::<i64>().ok()?;
        if oh > 23 || om > 59 {
            return None;
        }
        (clock, sign * (oh * 3600 + om * 60))
    };

    let mut t = clock.split(':');
    let hour = t.next()?.parse::<u32>().ok()?;
    let minute = t.next()?.parse::<u32>().ok()?;
    let sec_frac = t.next()?;
    if t.next().is_some() || hour > 23 || minute > 59 {
        return None;
    }
    let (second_raw, fraction) = sec_frac.split_once('.').unwrap_or((sec_frac, ""));
    let second = second_raw.parse::<u32>().ok()?;
    if second > 59 || (!fraction.is_empty() && !fraction.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let nanos = if fraction.is_empty() {
        0
    } else {
        let mut digits = fraction.chars().take(9).collect::<String>();
        while digits.len() < 9 {
            digits.push('0');
        }
        digits.parse::<u32>().ok()?
    };
    let days = days_from_civil(year, month, day)?;
    let local = days
        .checked_mul(86_400)?
        .checked_add(hour as i64 * 3600 + minute as i64 * 60 + second as i64)?;
    Some(Timestamp {
        seconds: local.checked_sub(offset_seconds)?,
        nanos,
    })
}

/// Validate actor forms used by `generated.by` and `verified[].by`.
pub fn valid_actor(raw: &str) -> bool {
    let s = raw.trim();
    if let Some(id) = s
        .strip_prefix("human:")
        .or_else(|| s.strip_prefix("process:"))
    {
        return valid_actor_part(id);
    }
    match s.split_once('/') {
        Some((producer, version)) => {
            valid_actor_part(producer) && valid_actor_part(version) && !version.contains('/')
        }
        None => false,
    }
}

fn valid_actor_part(s: &str) -> bool {
    !s.is_empty() && !s.chars().any(char::is_whitespace)
}

fn valid_date(year: i32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    day <= max
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !valid_date(year, month, day) {
        return None;
    }
    let y = year as i64 - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = month as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_require_offset_and_compare_as_instants() {
        assert!(parse_timestamp("2026-01-01").is_none());
        assert!(parse_timestamp("2026-01-01T00:00:00").is_none());
        assert_eq!(
            parse_timestamp("2026-01-01T00:00:00Z"),
            parse_timestamp("2025-12-31T16:00:00-08:00")
        );
        assert!(parse_timestamp("2026-02-29T00:00:00Z").is_none());
    }

    #[test]
    fn validates_actor_convention() {
        assert!(valid_actor("human:alice"));
        assert!(valid_actor("process:nightly"));
        assert!(valid_actor("agent/v1"));
        assert!(!valid_actor("alice"));
        assert!(!valid_actor("human:"));
    }
}
