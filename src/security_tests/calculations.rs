use super::*;

fn valid_zone() -> ZoneEntry {
    ZoneEntry {
        name: "Zone de contrôle".to_string(),
        session_time_seconds: 60.0,
        session_total_kamas: 100.0,
        ..ZoneEntry::default()
    }
}

#[test]
fn finite_but_extreme_amounts_are_rejected() {
    for amount in [3.0e38, f32::MAX, f32::INFINITY, f32::NAN, -1.0] {
        let mut entry = valid_zone();
        entry.session_total_kamas = amount;
        assert!(sanitize_zone_entry(entry).is_err());
    }
}

#[test]
fn zero_subnormal_and_extreme_durations_are_rejected() {
    for seconds in [0.0, f32::MIN_POSITIVE, 0.5, 1.0e30, f32::INFINITY, f32::NAN] {
        let mut entry = valid_zone();
        entry.session_time_seconds = seconds;
        assert!(sanitize_zone_entry(entry).is_err());
    }
}

#[test]
fn duration_integer_overflow_is_an_error_not_a_panic() {
    assert!(parse_hms_to_seconds("4294967295:59:59").is_none());
    let duration = DurationInput {
        hours: u32::MAX.to_string(),
        minutes: "59".to_string(),
        seconds: "59".to_string(),
    };
    assert!(parse_duration_input(&duration).is_err());
    assert_eq!(
        parse_hms_to_seconds("24:59:59"),
        Some(MAX_DURATION_SECONDS as f32)
    );
    assert!(parse_hms_to_seconds("25:00:00").is_none());
}

#[test]
fn date_limits_and_session_end_are_checked() {
    assert!(validate_session_interval(Some(NaiveDateTime::MAX), 1.0).is_err());
    assert!(validate_session_interval(Some(NaiveDateTime::MIN), 1.0).is_err());
    let last = NaiveDateTime::parse_from_str("9999-12-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();
    assert!(validate_session_interval(Some(last), 1.0).is_err());
    assert!(parse_recorded_at("1969-01-01 00:00").is_err());
    assert!(parse_recorded_at("2026-09-13 14:30").is_ok());
}

#[test]
fn utf8_names_and_control_characters_are_bounded() {
    let mut entry = valid_zone();
    entry.name = "é".repeat(MAX_NAME_BYTES / 2 + 1);
    assert!(sanitize_zone_entry(entry).is_err());
    let mut entry = valid_zone();
    entry.name = "Zone\nmasquée".to_string();
    assert!(sanitize_zone_entry(entry).is_err());
}

#[test]
fn arena_quantities_cannot_amplify_amounts_without_bound() {
    let entry = ArenaEntry {
        name: "Arène".to_string(),
        round_time_minutes: 1.0,
        seats_sold: MAX_QUANTITY + 1,
        ..ArenaEntry::default()
    };
    assert!(sanitize_arena_entry(entry).is_err());
}

#[test]
fn maximum_allowed_arena_calculations_are_finite_and_json_round_trip() {
    let entry = sanitize_arena_entry(ArenaEntry {
        name: "Arène maximale".to_string(),
        round_time_minutes: 1.0 / 60.0,
        seat_price: MAX_INPUT_KAMAS,
        seats_sold: MAX_QUANTITY,
        capture_price: MAX_INPUT_KAMAS,
        captures_count: MAX_QUANTITY,
        ..ArenaEntry::default()
    })
    .unwrap();
    assert!(entry.gross_revenue.is_finite());
    assert!(entry.total_capture_cost.is_finite());
    assert!(entry.kamas_per_hour.is_finite());
    let json = serde_json::to_string(&entry).unwrap();
    let restored: ArenaEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(entry, restored);
}

#[test]
fn stale_nonfinite_derived_fields_are_recomputed_before_serialization() {
    let mut entry = valid_zone();
    entry.kamas_per_hour = f32::INFINITY;
    let sanitized = sanitize_zone_entry(entry).unwrap();
    assert_eq!(sanitized.kamas_per_hour, 6000.0);
    let json = serde_json::to_string(&sanitized).unwrap();
    assert!(serde_json::from_str::<ZoneEntry>(&json).is_ok());
}

#[test]
fn minimum_signed_integer_can_be_formatted_without_overflow() {
    assert_eq!(format_with_spaces(i64::MIN), "-9 223 372 036 854 775 808");
}

#[test]
fn oversized_number_text_is_rejected_before_normalization() {
    let text = "1".repeat(MAX_NUMBER_INPUT_BYTES + 1);
    assert_eq!(parse_f32(&text), None);
    assert_eq!(parse_u32(&text), None);
}

#[test]
fn large_finite_values_are_not_saturated_to_i64_in_the_display() {
    let value = 1.0e25_f32;
    let digits = format_kamas(value).replace(' ', "");
    assert_eq!(digits, format!("{value:.0}"));
    assert_eq!(format_kamas(f32::INFINITY), "—");
    assert_eq!(format_kamas(f32::NAN), "—");
}
