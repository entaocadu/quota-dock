use chrono::DateTime;

pub fn to_remaining_from_used(used: Option<f64>) -> Option<f64> {
    match used {
        Some(val) if val.is_finite() && val >= 0.0 => {
            let clamped = (100.0 - val).clamp(0.0, 100.0);
            Some((clamped * 100.0).round() / 100.0)
        }
        _ => None,
    }
}

pub fn to_remaining_from_fraction(fraction: Option<f64>) -> Option<f64> {
    match fraction {
        Some(val) if val.is_finite() && val >= 0.0 && val <= 1.5 => {
            let clamped = (val * 100.0).clamp(0.0, 100.0);
            Some((clamped * 100.0).round() / 100.0)
        }
        _ => None,
    }
}

pub fn epoch_seconds_to_iso(secs: i64) -> Option<String> {
    if secs <= 0 {
        return None;
    }
    DateTime::from_timestamp(secs, 0).map(|dt| dt.to_rfc3339())
}

pub fn epoch_ms_to_iso(ms: i64) -> Option<String> {
    if ms <= 0 {
        return None;
    }
    DateTime::from_timestamp_millis(ms).map(|dt| dt.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_remaining_from_used() {
        assert_eq!(to_remaining_from_used(Some(35.0)), Some(65.0));
        assert_eq!(to_remaining_from_used(Some(0.0)), Some(100.0));
        assert_eq!(to_remaining_from_used(Some(100.0)), Some(0.0));
        assert_eq!(to_remaining_from_used(Some(105.0)), Some(0.0));
        assert_eq!(to_remaining_from_used(None), None);
        assert_eq!(to_remaining_from_used(Some(f64::NAN)), None);
        assert_eq!(to_remaining_from_used(Some(f64::INFINITY)), None);
        assert_eq!(to_remaining_from_used(Some(-5.0)), None);
    }

    #[test]
    fn test_to_remaining_from_fraction() {
        assert_eq!(to_remaining_from_fraction(Some(0.42)), Some(42.0));
        assert_eq!(to_remaining_from_fraction(Some(1.0)), Some(100.0));
        assert_eq!(to_remaining_from_fraction(Some(0.0)), Some(0.0));
        assert_eq!(to_remaining_from_fraction(None), None);
        assert_eq!(to_remaining_from_fraction(Some(f64::NAN)), None);
        assert_eq!(to_remaining_from_fraction(Some(f64::INFINITY)), None);
        assert_eq!(to_remaining_from_fraction(Some(-0.1)), None);
        assert_eq!(to_remaining_from_fraction(Some(2.5)), None);
    }

    #[test]
    fn test_epoch_conversions() {
        // 1791770696 is year 2026
        let iso_secs = epoch_seconds_to_iso(1791770696);
        assert!(iso_secs.is_some());
        assert!(iso_secs.unwrap().contains("2026"));

        let iso_ms = epoch_ms_to_iso(1791770696000);
        assert!(iso_ms.is_some());
        assert!(iso_ms.unwrap().contains("2026"));

        assert_eq!(epoch_seconds_to_iso(-10), None);
        assert_eq!(epoch_ms_to_iso(0), None);
    }
}
