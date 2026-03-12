use crate::models::{ArenaEntry, DungeonEntry, DuoTrioEntry, DurationInput, ZoneEntry};
use chrono::NaiveDateTime;

const MAX_SELECTOR_DURATION_SECONDS: u32 = 24 * 3600 + 59 * 60 + 59;

pub fn parse_f32(value: &str) -> Option<f32> {
    let normalized = normalize_number_input(value);

    if normalized.is_empty() {
        return None;
    }

    let parsed = normalized.parse::<f32>().ok()?;
    parsed.is_finite().then_some(parsed)
}

pub fn parse_non_negative_f32(value: &str, field_label: &str) -> Result<f32, String> {
    let parsed = parse_f32(value).ok_or_else(|| format!("{field_label} invalide."))?;
    validate_non_negative_f32(parsed, field_label)
}

pub fn parse_u32(value: &str) -> Option<u32> {
    let normalized = normalize_unsigned_input(value);

    if normalized.is_empty() {
        return None;
    }

    normalized.parse::<u32>().ok()
}

#[allow(dead_code)]
pub fn parse_hms_to_seconds(value: &str) -> Option<f32> {
    let value = value.trim();
    let mut parts = value.split(':');

    let hours = parts.next()?.trim().parse::<u32>().ok()?;
    let minutes = parts.next()?.trim().parse::<u32>().ok()?;
    let seconds = parts.next()?.trim().parse::<u32>().ok()?;

    if parts.next().is_some() || minutes >= 60 || seconds >= 60 {
        return None;
    }

    Some((hours * 3600 + minutes * 60 + seconds) as f32)
}

pub fn parse_duration_input(input: &DurationInput) -> Result<f32, String> {
    let hours = parse_duration_part(&input.hours, "heures")?;
    let minutes = parse_duration_part(&input.minutes, "minutes")?;
    let seconds = parse_duration_part(&input.seconds, "secondes")?;

    if minutes >= 60 || seconds >= 60 {
        return Err("Les minutes et les secondes doivent rester entre 00 et 59.".to_string());
    }

    let total_seconds = (hours * 3600 + minutes * 60 + seconds) as f32;

    if total_seconds <= 0.0 {
        return Err("La duree doit etre superieure a 00:00:00.".to_string());
    }

    Ok(total_seconds)
}

pub fn duration_input_from_seconds(value: f32) -> DurationInput {
    let total_seconds = (value.max(0.0).round() as u32).min(MAX_SELECTOR_DURATION_SECONDS);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    DurationInput {
        hours: format!("{hours:02}"),
        minutes: format!("{minutes:02}"),
        seconds: format!("{seconds:02}"),
    }
}

pub fn duration_input_from_minutes(value: f32) -> DurationInput {
    duration_input_from_seconds(value * 60.0)
}

pub fn seconds_to_minutes(value: f32) -> f32 {
    value / 60.0
}

pub fn zone_kamas_per_hour(session_total_kamas: f32, session_time_seconds: f32) -> f32 {
    session_total_kamas * (3600.0 / session_time_seconds)
}

pub fn dungeon_kamas_per_hour(net_kamas_per_run: f32, run_time_minutes: f32) -> f32 {
    net_kamas_per_run * (60.0 / run_time_minutes)
}

pub fn duo_trio_kamas_per_hour(net_kamas_per_run: f32, run_time_seconds: f32) -> f32 {
    net_kamas_per_run * (3600.0 / run_time_seconds)
}

pub fn arena_kamas_per_hour(net_profit: f32, round_time_minutes: f32) -> f32 {
    net_profit * (60.0 / round_time_minutes)
}

pub fn parse_recorded_at(value: &str) -> Result<Option<NaiveDateTime>, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Ok(None);
    }

    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M")
        .map(Some)
        .map_err(|_| "Date et heure invalides. Format attendu: YYYY-MM-DD HH:MM.".to_string())
}

pub fn format_kamas(value: f32) -> String {
    let rounded = value.round() as i64;
    format_with_spaces(rounded)
}

pub fn format_number(value: f32) -> String {
    let s = format!("{value:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn format_duration_hms(value: f32) -> String {
    let total_seconds = value.max(0.0).round() as u32;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn format_recorded_at(value: Option<NaiveDateTime>) -> String {
    value
        .map(|recorded_at| recorded_at.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}

pub fn format_with_spaces(value: i64) -> String {
    let negative = value < 0;
    let s = value.abs().to_string();
    let mut out = String::new();

    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }

    let mut result: String = out.chars().rev().collect();
    if negative {
        result.insert(0, '-');
    }
    result
}

pub fn normalize_text_for_matching(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());

    for ch in value.chars() {
        for lowered in ch.to_lowercase() {
            match lowered {
                'a'..='z' | '0'..='9' => normalized.push(lowered),
                'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => normalized.push('a'),
                'æ' => normalized.push_str("ae"),
                'ç' => normalized.push('c'),
                'è' | 'é' | 'ê' | 'ë' => normalized.push('e'),
                'ì' | 'í' | 'î' | 'ï' => normalized.push('i'),
                'ñ' => normalized.push('n'),
                'ò' | 'ó' | 'ô' | 'ö' | 'õ' => normalized.push('o'),
                'œ' => normalized.push_str("oe"),
                'ù' | 'ú' | 'û' | 'ü' => normalized.push('u'),
                'ý' | 'ÿ' => normalized.push('y'),
                '\'' | '’' | '"' | '-' | '_' | '/' | '\\' | '.' | ',' | ':' | ';' | '(' | ')'
                | '[' | ']' | '{' | '}' | '!' | '?' | '+' | '*' | '&' | '#' | '@' | ' ' | '\t'
                | '\n' | '\r' => {}
                other if other.is_alphanumeric() => normalized.push(other),
                _ => {}
            }
        }
    }

    normalized
}

pub fn sanitize_zone_entry(mut entry: ZoneEntry) -> Result<ZoneEntry, String> {
    entry.name = entry.name.trim().to_string();
    validate_non_empty_name(&entry.name, "Le nom de la zone")?;
    entry.session_time_seconds =
        validate_positive_f32(entry.session_time_seconds, "La duree de session")?;
    entry.session_total_kamas =
        validate_non_negative_f32(entry.session_total_kamas, "La valeur totale de session")?;
    entry.kamas_per_hour =
        zone_kamas_per_hour(entry.session_total_kamas, entry.session_time_seconds);
    Ok(entry)
}

pub fn sanitize_dungeon_entry(mut entry: DungeonEntry) -> Result<DungeonEntry, String> {
    entry.name = entry.name.trim().to_string();
    validate_non_empty_name(&entry.name, "Le nom du donjon")?;
    entry.run_time_minutes = validate_positive_f32(entry.run_time_minutes, "Le temps du donjon")?;
    entry.gross_kamas_per_run =
        validate_non_negative_f32(entry.gross_kamas_per_run, "Le gain brut moyen")?;
    entry.key_price = validate_non_negative_f32(entry.key_price, "Le prix de la cle")?;
    entry.net_kamas_per_run = entry.gross_kamas_per_run - entry.key_price;
    entry.kamas_per_hour = dungeon_kamas_per_hour(entry.net_kamas_per_run, entry.run_time_minutes);
    Ok(entry)
}

pub fn sanitize_duo_trio_entry(mut entry: DuoTrioEntry) -> Result<DuoTrioEntry, String> {
    entry.name = entry.name.trim().to_string();
    validate_non_empty_name(&entry.name, "Le nom du run duo/trio")?;
    entry.run_time_seconds = validate_positive_f32(entry.run_time_seconds, "Le temps du run")?;
    entry.loot_kamas_per_run =
        validate_non_negative_f32(entry.loot_kamas_per_run, "Le loot total du run")?;
    entry.capture_stone_price =
        validate_non_negative_f32(entry.capture_stone_price, "Le prix de la pierre de capture")?;
    entry.key_unit_price =
        validate_non_negative_f32(entry.key_unit_price, "Le prix unitaire de la cle")?;
    entry.full_soul_sale_price = validate_non_negative_f32(
        entry.full_soul_sale_price,
        "Le prix de vente de la capture pleine",
    )?;

    entry.keys_count = entry.party_mode.player_count();
    entry.total_key_cost = entry.key_unit_price * entry.keys_count as f32;
    entry.gross_kamas_per_run = entry.loot_kamas_per_run + entry.full_soul_sale_price;
    entry.total_cost = entry.capture_stone_price + entry.total_key_cost;
    entry.net_kamas_per_run = entry.gross_kamas_per_run - entry.total_cost;
    entry.kamas_per_hour = duo_trio_kamas_per_hour(entry.net_kamas_per_run, entry.run_time_seconds);

    Ok(entry)
}

pub fn sanitize_arena_entry(mut entry: ArenaEntry) -> Result<ArenaEntry, String> {
    entry.name = entry.name.trim().to_string();
    validate_non_empty_name(&entry.name, "Le nom de la session PL arene")?;
    entry.round_time_minutes =
        validate_positive_f32(entry.round_time_minutes, "Le temps de la ronde")?;
    entry.seat_price = validate_non_negative_f32(entry.seat_price, "Le prix d'une place")?;
    entry.capture_price = validate_non_negative_f32(entry.capture_price, "Le prix d'une capture")?;

    entry.gross_revenue = entry.seat_price * entry.seats_sold as f32;
    entry.total_capture_cost = entry.capture_price * entry.captures_count as f32;
    entry.net_profit = entry.gross_revenue - entry.total_capture_cost;
    entry.kamas_per_hour = arena_kamas_per_hour(entry.net_profit, entry.round_time_minutes);

    Ok(entry)
}

fn normalize_number_input(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter_map(|ch| match ch {
            '0'..='9' | '-' | '+' | '.' => Some(ch),
            ',' => Some('.'),
            ' ' | '\t' | '\n' | '\r' | '_' | '\'' | '’' | '\u{00A0}' | '\u{202F}' => None,
            other => Some(other),
        })
        .collect()
}

fn normalize_unsigned_input(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter_map(|ch| match ch {
            '0'..='9' => Some(ch),
            ' ' | '\t' | '\n' | '\r' | '_' | '\'' | '’' | '\u{00A0}' | '\u{202F}' => None,
            other => Some(other),
        })
        .collect()
}

fn parse_duration_part(value: &str, label: &str) -> Result<u32, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(format!("Le champ {label} est obligatoire."));
    }

    trimmed
        .parse::<u32>()
        .map_err(|_| format!("Le champ {label} doit contenir uniquement des chiffres."))
}

fn validate_non_empty_name(value: &str, field_label: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field_label} est obligatoire."));
    }

    Ok(())
}

fn validate_positive_f32(value: f32, field_label: &str) -> Result<f32, String> {
    if !value.is_finite() {
        return Err(format!("{field_label} est invalide."));
    }

    if value <= 0.0 {
        return Err(format!("{field_label} doit etre superieur a 0."));
    }

    Ok(value)
}

fn validate_non_negative_f32(value: f32, field_label: &str) -> Result<f32, String> {
    if !value.is_finite() {
        return Err(format!("{field_label} est invalide."));
    }

    if value < 0.0 {
        return Err(format!("{field_label} doit etre superieur ou egal a 0."));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DofusClass, PartyMode};

    #[test]
    fn parse_hms_to_seconds_accepts_valid_time() {
        assert_eq!(parse_hms_to_seconds("00:06:30"), Some(390.0));
        assert_eq!(parse_hms_to_seconds("2:00:05"), Some(7205.0));
    }

    #[test]
    fn parse_hms_to_seconds_rejects_invalid_time() {
        assert_eq!(parse_hms_to_seconds("06:30"), None);
        assert_eq!(parse_hms_to_seconds("00:70:00"), None);
        assert_eq!(parse_hms_to_seconds("00:10:99"), None);
    }

    #[test]
    fn parse_duration_input_accepts_valid_duration() {
        let input = DurationInput {
            hours: "01".to_string(),
            minutes: "30".to_string(),
            seconds: "45".to_string(),
        };

        assert_eq!(parse_duration_input(&input), Ok(5_445.0));
    }

    #[test]
    fn parse_duration_input_rejects_invalid_values() {
        let invalid_minutes = DurationInput {
            hours: "01".to_string(),
            minutes: "60".to_string(),
            seconds: "00".to_string(),
        };
        let zero_duration = DurationInput {
            hours: "00".to_string(),
            minutes: "00".to_string(),
            seconds: "00".to_string(),
        };

        assert_eq!(
            parse_duration_input(&invalid_minutes),
            Err("Les minutes et les secondes doivent rester entre 00 et 59.".to_string())
        );
        assert_eq!(
            parse_duration_input(&zero_duration),
            Err("La duree doit etre superieure a 00:00:00.".to_string())
        );
    }

    #[test]
    fn parse_f32_accepts_grouped_french_numbers() {
        assert_eq!(parse_f32("1 500 000"), Some(1_500_000.0));
        assert_eq!(parse_f32("1\u{202F}500\u{202F}000,50"), Some(1_500_000.5));
        assert_eq!(parse_f32("12_345"), Some(12_345.0));
    }

    #[test]
    fn parse_f32_rejects_nan_and_infinity() {
        assert_eq!(parse_f32("NaN"), None);
        assert_eq!(parse_f32("inf"), None);
        assert_eq!(parse_f32("-inf"), None);
    }

    #[test]
    fn parse_non_negative_f32_rejects_negative_values() {
        assert_eq!(
            parse_non_negative_f32("-1", "Prix"),
            Err("Prix doit etre superieur ou egal a 0.".to_string())
        );
    }

    #[test]
    fn parse_u32_accepts_grouped_digits_and_rejects_invalid_values() {
        assert_eq!(parse_u32("1 500"), Some(1_500));
        assert_eq!(parse_u32("12_345"), Some(12_345));
        assert_eq!(parse_u32("-1"), None);
        assert_eq!(parse_u32("10,5"), None);
    }

    #[test]
    fn normalize_text_for_matching_is_accent_and_separator_insensitive() {
        assert_eq!(
            normalize_text_for_matching("Cimetière des Torturés"),
            "cimetieredestortures"
        );
        assert_eq!(normalize_text_for_matching("Qu'Tan"), "qutan");
        assert_eq!(normalize_text_for_matching("Blop-Royal"), "bloproyal");
        assert_eq!(normalize_text_for_matching("Œuf d'Âne"), "oeufdane");
    }

    #[test]
    fn format_duration_hms_formats_seconds() {
        assert_eq!(format_duration_hms(390.0), "00:06:30");
        assert_eq!(format_duration_hms(7205.0), "02:00:05");
    }

    #[test]
    fn duration_input_roundtrips_seconds_and_minutes() {
        let from_seconds = duration_input_from_seconds(5_445.0);
        let from_minutes = duration_input_from_minutes(90.75);
        let clamped = duration_input_from_seconds(100_000.0);

        assert_eq!(
            from_seconds,
            DurationInput {
                hours: "01".to_string(),
                minutes: "30".to_string(),
                seconds: "45".to_string(),
            }
        );
        assert_eq!(from_minutes, from_seconds);
        assert_eq!(
            clamped,
            DurationInput {
                hours: "24".to_string(),
                minutes: "59".to_string(),
                seconds: "59".to_string(),
            }
        );
        assert_eq!(seconds_to_minutes(5_445.0), 90.75);
    }

    #[test]
    fn zone_kamas_per_hour_uses_session_values() {
        assert_eq!(zone_kamas_per_hour(240_000.0, 7_200.0), 120_000.0);
    }

    #[test]
    fn duo_trio_kamas_per_hour_uses_run_seconds() {
        assert_eq!(duo_trio_kamas_per_hour(180_000.0, 5_400.0), 120_000.0);
    }

    #[test]
    fn parse_recorded_at_accepts_empty_or_valid_datetime() {
        assert_eq!(parse_recorded_at(""), Ok(None));
        assert_eq!(
            parse_recorded_at("2026-03-08 14:45"),
            Ok(Some(
                NaiveDateTime::parse_from_str("2026-03-08 14:45", "%Y-%m-%d %H:%M").unwrap()
            ))
        );
    }

    #[test]
    fn parse_recorded_at_rejects_invalid_datetime() {
        assert_eq!(
            parse_recorded_at("08/03/2026 14:45"),
            Err("Date et heure invalides. Format attendu: YYYY-MM-DD HH:MM.".to_string())
        );
    }

    #[test]
    fn format_with_spaces_keeps_negative_sign() {
        assert_eq!(format_with_spaces(-1_234_567), "-1 234 567");
    }

    #[test]
    fn sanitize_zone_entry_trims_name_and_recomputes_rate() {
        let entry = sanitize_zone_entry(ZoneEntry {
            name: " Plaine ".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at: None,
            session_time_seconds: 1_800.0,
            session_total_kamas: 120_000.0,
            kamas_per_hour: 1.0,
        })
        .unwrap();

        assert_eq!(entry.name, "Plaine");
        assert_eq!(entry.kamas_per_hour, 240_000.0);
    }

    #[test]
    fn sanitize_dungeon_entry_recalculates_derived_values() {
        let entry = sanitize_dungeon_entry(DungeonEntry {
            name: "Blop".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at: None,
            run_time_minutes: 20.0,
            gross_kamas_per_run: 120_000.0,
            key_price: 15_000.0,
            net_kamas_per_run: 999_999.0,
            kamas_per_hour: 1.0,
        })
        .unwrap();

        assert_eq!(entry.net_kamas_per_run, 105_000.0);
        assert_eq!(entry.kamas_per_hour, 315_000.0);
    }

    #[test]
    fn sanitize_duo_trio_entry_recomputes_party_costs() {
        let entry = sanitize_duo_trio_entry(DuoTrioEntry {
            name: "Trio Illy".to_string(),
            character_class: Some(DofusClass::Enutrof),
            recorded_at: None,
            party_mode: PartyMode::Trio,
            run_time_seconds: 4_800.0,
            loot_kamas_per_run: 320_000.0,
            capture_stone_price: 45_000.0,
            key_unit_price: 12_000.0,
            keys_count: 99,
            total_key_cost: 999_999.0,
            full_soul_sale_price: 180_000.0,
            gross_kamas_per_run: 1.0,
            total_cost: 1.0,
            net_kamas_per_run: 1.0,
            kamas_per_hour: 1.0,
        })
        .unwrap();

        assert_eq!(entry.keys_count, 3);
        assert_eq!(entry.total_key_cost, 36_000.0);
        assert_eq!(entry.gross_kamas_per_run, 500_000.0);
        assert_eq!(entry.total_cost, 81_000.0);
        assert_eq!(entry.net_kamas_per_run, 419_000.0);
        assert_eq!(entry.kamas_per_hour, 314_250.0);
    }

    #[test]
    fn sanitize_arena_entry_recomputes_negative_profit() {
        let entry = sanitize_arena_entry(ArenaEntry {
            name: " Bworker ".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at: None,
            round_time_minutes: 30.0,
            seat_price: 50_000.0,
            seats_sold: 7,
            capture_price: 100_000.0,
            captures_count: 9,
            gross_revenue: 1.0,
            total_capture_cost: 1.0,
            net_profit: 1.0,
            kamas_per_hour: 1.0,
        })
        .unwrap();

        assert_eq!(entry.name, "Bworker");
        assert_eq!(entry.gross_revenue, 350_000.0);
        assert_eq!(entry.total_capture_cost, 900_000.0);
        assert_eq!(entry.net_profit, -550_000.0);
        assert_eq!(entry.kamas_per_hour, -1_100_000.0);
    }
}
