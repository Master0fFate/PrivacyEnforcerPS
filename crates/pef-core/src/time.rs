//! Minimal RFC 3339 support for backup timestamps, without a date-time dependency.

use std::time::{SystemTime, UNIX_EPOCH};

/// The current time as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc_now_rfc3339() -> String {
    format_rfc3339(SystemTime::now())
}

/// Formats a point in time as `YYYY-MM-DDTHH:MM:SSZ` (seconds precision, UTC).
pub fn format_rfc3339(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        second_of_day / 3_600,
        (second_of_day % 3_600) / 60,
        second_of_day % 60
    )
}

/// Converts days since 1970-01-01 to a proleptic Gregorian civil date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn digits(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u32> {
    let slice = bytes.get(range)?;
    if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
        return None;
    }
    slice.iter().try_fold(0u32, |acc, b| {
        acc.checked_mul(10)?.checked_add(u32::from(b - b'0'))
    })
}

/// Returns true for a complete RFC 3339 `date-time` such as `2026-01-02T03:04:05Z`,
/// `2026-01-02T03:04:05.1234567+00:00` or `2026-01-02t03:04:05-05:30`.
pub fn is_rfc3339_timestamp(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() < 20 {
        return false;
    }
    let (Some(year), Some(month), Some(day)) = (
        digits(bytes, 0..4),
        digits(bytes, 5..7),
        digits(bytes, 8..10),
    ) else {
        return false;
    };
    if bytes[4] != b'-' || bytes[7] != b'-' || !matches!(bytes[10], b'T' | b't') {
        return false;
    }
    let (Some(hour), Some(minute), Some(second)) = (
        digits(bytes, 11..13),
        digits(bytes, 14..16),
        digits(bytes, 17..19),
    ) else {
        return false;
    };
    if bytes[13] != b':' || bytes[16] != b':' {
        return false;
    }
    if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
        return false;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return false;
    }
    let mut index = 19;
    if bytes[index] == b'.' {
        let start = index + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            return false;
        }
        index = end;
    }
    match bytes.get(index) {
        Some(b'Z' | b'z') => index + 1 == bytes.len(),
        Some(b'+' | b'-') => {
            if bytes.len() != index + 6 || bytes[index + 3] != b':' {
                return false;
            }
            matches!(
                (digits(bytes, index + 1..index + 3), digits(bytes, index + 4..index + 6)),
                (Some(h), Some(m)) if h <= 23 && m <= 59
            )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let t = UNIX_EPOCH + Duration::from_secs(951_782_400); // 2000-02-29
        assert_eq!(format_rfc3339(t), "2000-02-29T00:00:00Z");
        let t = UNIX_EPOCH + Duration::from_secs(1_767_225_599); // 2025-12-31T23:59:59Z
        assert_eq!(format_rfc3339(t), "2025-12-31T23:59:59Z");
        let t = UNIX_EPOCH + Duration::from_secs(1_767_225_600);
        assert_eq!(format_rfc3339(t), "2026-01-01T00:00:00Z");
        assert_eq!(
            format_rfc3339(UNIX_EPOCH + Duration::from_secs(4_102_444_800)),
            "2100-01-01T00:00:00Z"
        );
    }

    #[test]
    fn formatted_output_validates() {
        for secs in [
            0u64,
            86_399,
            86_400,
            951_782_400,
            1_700_000_000,
            4_102_444_800,
        ] {
            let text = format_rfc3339(UNIX_EPOCH + Duration::from_secs(secs));
            assert!(is_rfc3339_timestamp(&text), "{text}");
        }
        assert!(is_rfc3339_timestamp(&utc_now_rfc3339()));
    }

    #[test]
    fn civil_dates_match_a_reference_walk() {
        // Walk day by day from 1970 and compare against an independent month table.
        let (mut y, mut m, mut d) = (1970u32, 1u32, 1u32);
        for day_number in 0..(366 * 60) {
            assert_eq!(
                civil_from_days(day_number),
                (i64::from(y), m, d),
                "day {day_number}"
            );
            d += 1;
            if d > days_in_month(y, m) {
                d = 1;
                m += 1;
                if m > 12 {
                    m = 1;
                    y += 1;
                }
            }
        }
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn accepts_rfc3339_variants() {
        for ok in [
            "2026-01-02T03:04:05Z",
            "2026-01-02t03:04:05z",
            "2026-01-02T03:04:05.1Z",
            "2026-01-02T03:04:05.1234567+00:00",
            "2026-01-02T03:04:05-05:30",
            "2024-02-29T23:59:60+23:59",
            "0001-01-01T00:00:00Z",
        ] {
            assert!(is_rfc3339_timestamp(ok), "{ok}");
        }
    }

    #[test]
    fn rejects_malformed_timestamps() {
        for bad in [
            "",
            "2026-01-02",
            "2026-01-02T03:04:05",
            "2026-01-02 03:04:05Z",
            "2026-13-02T03:04:05Z",
            "2026-00-02T03:04:05Z",
            "2026-02-30T03:04:05Z",
            "2023-02-29T03:04:05Z",
            "2026-01-02T24:04:05Z",
            "2026-01-02T03:60:05Z",
            "2026-01-02T03:04:61Z",
            "2026-01-02T03:04:05.Z",
            "2026-01-02T03:04:05+0000",
            "2026-01-02T03:04:05+24:00",
            "2026-01-02T03:04:05+00:60",
            "2026-01-02T03:04:05ZZ",
            "2026-01-02T03:04:05Z ",
            "２026-01-02T03:04:05Z",
            "2026-01-02T03:04:05+00:00x",
            "now",
        ] {
            assert!(!is_rfc3339_timestamp(bad), "{bad:?} must be rejected");
        }
    }
}
