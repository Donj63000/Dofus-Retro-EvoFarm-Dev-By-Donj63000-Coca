use crate::calculations::{
    sanitize_arena_entry, sanitize_dungeon_entry, sanitize_duo_trio_entry, sanitize_zone_entry,
};
use crate::models::{AppData, ArenaEntry, DungeonEntry, DuoTrioEntry, ZoneEntry};
use serde_json::{Map, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LoadDataResult {
    pub data: AppData,
    pub cleaned_legacy_entries: usize,
}

pub fn data_file_path() -> PathBuf {
    let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("dofus_rentabilite");
    dir.push("data.json");
    dir
}

pub fn load_data() -> Result<LoadDataResult, String> {
    let path = data_file_path();
    load_data_from_path(&path)
}

pub fn load_data_from_path(path: &Path) -> Result<LoadDataResult, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Impossible de lire {} : {error}", path.display()))?;
    parse_data(&content)
        .map_err(|error| format!("Impossible de charger {} : {error}", path.display()))
}

pub fn save_data(data: &AppData) -> Result<PathBuf, String> {
    let path = data_file_path();
    save_data_to_path(data, &path)
}

pub fn save_data_to_path(data: &AppData, path: &Path) -> Result<PathBuf, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let sanitized = sanitize_app_data(data)?;
    let json = serde_json::to_vec_pretty(&sanitized).map_err(|error| error.to_string())?;
    write_atomic(path, &json)?;
    Ok(path.to_path_buf())
}

fn parse_data(content: &str) -> Result<LoadDataResult, String> {
    let raw: Value = serde_json::from_str(content).map_err(|error| error.to_string())?;

    let (zones, cleaned_zones) = parse_collection(&raw, "zones", is_legacy_zone_entry, |item| {
        let entry = serde_json::from_value::<ZoneEntry>(item).map_err(|error| error.to_string())?;
        sanitize_zone_entry(entry)
    })?;
    let (dungeons, cleaned_dungeons) = parse_collection(
        &raw,
        "dungeons",
        |_| false,
        |item| {
            let entry =
                serde_json::from_value::<DungeonEntry>(item).map_err(|error| error.to_string())?;
            sanitize_dungeon_entry(entry)
        },
    )?;
    let (duo_trios, cleaned_duo_trios) = parse_collection(
        &raw,
        "duo_trios",
        |_| false,
        |item| {
            let entry =
                serde_json::from_value::<DuoTrioEntry>(item).map_err(|error| error.to_string())?;
            sanitize_duo_trio_entry(entry)
        },
    )?;
    let (arenas, cleaned_arenas) = parse_collection(
        &raw,
        "arenas",
        |_| false,
        |item| {
            let entry =
                serde_json::from_value::<ArenaEntry>(item).map_err(|error| error.to_string())?;
            sanitize_arena_entry(entry)
        },
    )?;

    let mut data = AppData {
        zones,
        dungeons,
        duo_trios,
        arenas,
    };
    sort_loaded_data(&mut data);

    Ok(LoadDataResult {
        data,
        cleaned_legacy_entries: cleaned_zones
            + cleaned_dungeons
            + cleaned_duo_trios
            + cleaned_arenas,
    })
}

fn parse_collection<T>(
    raw: &Value,
    key: &str,
    is_legacy_entry: impl Fn(&Map<String, Value>) -> bool,
    parse_entry: impl Fn(Value) -> Result<T, String>,
) -> Result<(Vec<T>, usize), String> {
    let Some(value) = raw.get(key) else {
        return Ok((Vec::new(), 0));
    };

    if value.is_null() {
        return Ok((Vec::new(), 0));
    }

    let items = value
        .as_array()
        .ok_or_else(|| format!("Le champ {key} doit etre une liste."))?;

    let mut entries = Vec::new();
    let mut cleaned = 0usize;

    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| format!("Une entree de {key} est invalide."))?;

        if object.contains_key("recorded_on") || is_legacy_entry(object) {
            cleaned += 1;
            continue;
        }

        let entry = parse_entry(item.clone())
            .map_err(|error| format!("Entree invalide dans {key}: {error}"))?;
        entries.push(entry);
    }

    Ok((entries, cleaned))
}

fn sanitize_app_data(data: &AppData) -> Result<AppData, String> {
    let mut sanitized = AppData {
        zones: data
            .zones
            .iter()
            .cloned()
            .map(sanitize_zone_entry)
            .collect::<Result<Vec<_>, _>>()?,
        dungeons: data
            .dungeons
            .iter()
            .cloned()
            .map(sanitize_dungeon_entry)
            .collect::<Result<Vec<_>, _>>()?,
        duo_trios: data
            .duo_trios
            .iter()
            .cloned()
            .map(sanitize_duo_trio_entry)
            .collect::<Result<Vec<_>, _>>()?,
        arenas: data
            .arenas
            .iter()
            .cloned()
            .map(sanitize_arena_entry)
            .collect::<Result<Vec<_>, _>>()?,
    };

    sort_loaded_data(&mut sanitized);
    Ok(sanitized)
}

fn sort_loaded_data(data: &mut AppData) {
    data.zones
        .sort_by(|left, right| right.kamas_per_hour.total_cmp(&left.kamas_per_hour));
    data.dungeons
        .sort_by(|left, right| right.kamas_per_hour.total_cmp(&left.kamas_per_hour));
    data.duo_trios
        .sort_by(|left, right| right.kamas_per_hour.total_cmp(&left.kamas_per_hour));
    data.arenas
        .sort_by(|left, right| right.kamas_per_hour.total_cmp(&left.kamas_per_hour));
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp_path = temp_path_for(path);
    let backup_path = backup_path_for(path);

    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| {
                format!(
                    "Impossible de creer le fichier temporaire {} : {error}",
                    temp_path.display()
                )
            })?;

        file.write_all(bytes).map_err(|error| {
            format!(
                "Impossible d'ecrire le fichier temporaire {} : {error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "Impossible de synchroniser le fichier temporaire {} : {error}",
                temp_path.display()
            )
        })?;

        if path.exists() {
            fs::copy(path, &backup_path).map_err(|error| {
                format!(
                    "Impossible de creer la sauvegarde {} : {error}",
                    backup_path.display()
                )
            })?;
        }

        replace_file(&temp_path, path)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

#[cfg(windows)]
fn replace_file(temp_path: &Path, path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("Impossible de remplacer {} : {error}", path.display()))?;
    }

    fs::rename(temp_path, path).map_err(|error| {
        format!(
            "Impossible de finaliser la sauvegarde vers {} : {error}",
            path.display()
        )
    })
}

#[cfg(not(windows))]
fn replace_file(temp_path: &Path, path: &Path) -> Result<(), String> {
    fs::rename(temp_path, path).map_err(|error| {
        format!(
            "Impossible de finaliser la sauvegarde vers {} : {error}",
            path.display()
        )
    })
}

fn backup_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("data.json");
    path.with_file_name(format!("{file_name}.bak"))
}

fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("data.json");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    path.with_file_name(format!("{file_name}.tmp-{}-{unique}", std::process::id()))
}

fn is_legacy_zone_entry(value: &Map<String, Value>) -> bool {
    value.contains_key("combat_time_seconds") || value.contains_key("kamas_per_combat")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DofusClass, PartyMode};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("evofarm-{name}-{unique}.json"))
    }

    #[test]
    fn parse_data_reads_modern_zone_schema() {
        let content = r#"{
            "zones": [
                {
                    "name": "Plaines",
                    "session_time_seconds": 5400.0,
                    "session_total_kamas": 250000.0,
                    "kamas_per_hour": 166666.67
                }
            ],
            "dungeons": [],
            "arenas": []
        }"#;

        let result = parse_data(content).unwrap();

        assert_eq!(result.cleaned_legacy_entries, 0);
        assert_eq!(result.data.zones.len(), 1);
        assert_eq!(result.data.zones[0].character_class, None);
        assert_eq!(result.data.zones[0].session_time_seconds, 5400.0);
        assert_eq!(result.data.zones[0].session_total_kamas, 250000.0);
        assert!((result.data.zones[0].kamas_per_hour - 166_666.67).abs() < 0.01);
        assert!(result.data.duo_trios.is_empty());
    }

    #[test]
    fn parse_data_drops_legacy_entries_but_keeps_modern_ones() {
        let content = r#"{
            "zones": [
                {
                    "name": "Ancienne zone",
                    "combat_time_seconds": 390.0,
                    "kamas_per_combat": 2500.0,
                    "kamas_per_hour": 23076.92
                },
                {
                    "name": "Zone moderne",
                    "session_time_seconds": 3600.0,
                    "session_total_kamas": 200000.0,
                    "kamas_per_hour": 200000.0
                }
            ],
            "dungeons": [
                {
                    "name": "Blop legacy",
                    "recorded_on": "2026-03-08",
                    "run_time_minutes": 20.0,
                    "gross_kamas_per_run": 120000.0,
                    "key_price": 15000.0,
                    "net_kamas_per_run": 105000.0,
                    "kamas_per_hour": 315000.0
                },
                {
                    "name": "Blop moderne",
                    "run_time_minutes": 20.0,
                    "gross_kamas_per_run": 120000.0,
                    "key_price": 15000.0,
                    "net_kamas_per_run": 105000.0,
                    "kamas_per_hour": 315000.0
                }
            ],
            "arenas": [
                {
                    "name": "Session test",
                    "recorded_on": null,
                    "round_time_minutes": 30.0,
                    "seat_price": 50000.0,
                    "seats_sold": 7,
                    "capture_price": 120000.0,
                    "captures_count": 10,
                    "gross_revenue": 350000.0,
                    "total_capture_cost": 1200000.0,
                    "net_profit": -850000.0,
                    "kamas_per_hour": -1700000.0
                }
            ]
        }"#;

        let result = parse_data(content).unwrap();

        assert_eq!(result.cleaned_legacy_entries, 3);
        assert_eq!(result.data.zones.len(), 1);
        assert_eq!(result.data.zones[0].name, "Zone moderne");
        assert_eq!(result.data.dungeons.len(), 1);
        assert_eq!(result.data.dungeons[0].name, "Blop moderne");
        assert!(result.data.duo_trios.is_empty());
        assert!(result.data.arenas.is_empty());
    }

    #[test]
    fn parse_data_recalculates_derived_values_from_imported_json() {
        let content = r#"{
            "zones": [],
            "dungeons": [
                {
                    "name": "Blop moderne",
                    "run_time_minutes": 20.0,
                    "gross_kamas_per_run": 120000.0,
                    "key_price": 15000.0,
                    "net_kamas_per_run": 1.0,
                    "kamas_per_hour": 1.0
                }
            ],
            "duo_trios": [
                {
                    "name": "Trio Illy",
                    "party_mode": "trio",
                    "run_time_seconds": 4800.0,
                    "loot_kamas_per_run": 320000.0,
                    "capture_stone_price": 45000.0,
                    "key_unit_price": 12000.0,
                    "keys_count": 99,
                    "total_key_cost": 999999.0,
                    "full_soul_sale_price": 180000.0,
                    "gross_kamas_per_run": 1.0,
                    "total_cost": 1.0,
                    "net_kamas_per_run": 1.0,
                    "kamas_per_hour": 1.0
                }
            ],
            "arenas": []
        }"#;

        let result = parse_data(content).unwrap();

        assert_eq!(result.data.dungeons[0].net_kamas_per_run, 105_000.0);
        assert_eq!(result.data.dungeons[0].kamas_per_hour, 315_000.0);
        assert_eq!(result.data.duo_trios[0].keys_count, 3);
        assert_eq!(result.data.duo_trios[0].total_key_cost, 36_000.0);
        assert_eq!(result.data.duo_trios[0].net_kamas_per_run, 419_000.0);
        assert_eq!(result.data.duo_trios[0].kamas_per_hour, 314_250.0);
    }

    #[test]
    fn parse_data_roundtrips_duo_trio_schema() {
        let data = AppData {
            zones: Vec::new(),
            dungeons: Vec::new(),
            duo_trios: vec![DuoTrioEntry {
                name: "Trio Illy".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                party_mode: PartyMode::Trio,
                run_time_seconds: 4_800.0,
                loot_kamas_per_run: 320_000.0,
                capture_stone_price: 45_000.0,
                key_unit_price: 12_000.0,
                keys_count: 3,
                total_key_cost: 36_000.0,
                full_soul_sale_price: 180_000.0,
                gross_kamas_per_run: 500_000.0,
                total_cost: 81_000.0,
                net_kamas_per_run: 419_000.0,
                kamas_per_hour: 314_250.0,
            }],
            arenas: Vec::new(),
        };
        let json = serde_json::to_string_pretty(&data).unwrap();

        let result = parse_data(&json).unwrap();

        assert_eq!(result.data.duo_trios.len(), 1);
        assert_eq!(result.data.duo_trios[0].name, "Trio Illy");
        assert_eq!(
            result.data.duo_trios[0].character_class,
            Some(DofusClass::Cra)
        );
        assert_eq!(result.data.duo_trios[0].party_mode, PartyMode::Trio);
        assert_eq!(result.data.duo_trios[0].keys_count, 3);
        assert_eq!(result.data.duo_trios[0].net_kamas_per_run, 419_000.0);
    }

    #[test]
    fn parse_data_accepts_legacy_capitalized_party_mode() {
        let content = r#"{
            "zones": [],
            "dungeons": [],
            "duo_trios": [
                {
                    "name": "Ben",
                    "party_mode": "Duo",
                    "run_time_seconds": 3600.0,
                    "loot_kamas_per_run": 200000.0,
                    "capture_stone_price": 20000.0,
                    "key_unit_price": 15000.0,
                    "keys_count": 99,
                    "total_key_cost": 999999.0,
                    "full_soul_sale_price": 100000.0,
                    "gross_kamas_per_run": 1.0,
                    "total_cost": 1.0,
                    "net_kamas_per_run": 1.0,
                    "kamas_per_hour": 1.0
                }
            ],
            "arenas": []
        }"#;

        let result = parse_data(content).unwrap();

        assert_eq!(result.data.duo_trios.len(), 1);
        assert_eq!(result.data.duo_trios[0].party_mode, PartyMode::Duo);
        assert_eq!(result.data.duo_trios[0].keys_count, 2);
        assert_eq!(result.data.duo_trios[0].total_key_cost, 30_000.0);
        assert_eq!(result.data.duo_trios[0].net_kamas_per_run, 250_000.0);
    }

    #[test]
    fn parse_data_sorts_entries_by_sanitized_kamas_per_hour() {
        let content = r#"{
            "zones": [],
            "dungeons": [
                {
                    "name": "Lent",
                    "run_time_minutes": 20.0,
                    "gross_kamas_per_run": 120000.0,
                    "key_price": 20000.0,
                    "net_kamas_per_run": 1.0,
                    "kamas_per_hour": 1.0
                },
                {
                    "name": "Rapide",
                    "run_time_minutes": 10.0,
                    "gross_kamas_per_run": 80000.0,
                    "key_price": 10000.0,
                    "net_kamas_per_run": 1.0,
                    "kamas_per_hour": 1.0
                }
            ],
            "duo_trios": [],
            "arenas": []
        }"#;

        let result = parse_data(content).unwrap();

        assert_eq!(result.data.dungeons[0].name, "Rapide");
        assert_eq!(result.data.dungeons[0].kamas_per_hour, 420_000.0);
        assert_eq!(result.data.dungeons[1].name, "Lent");
        assert_eq!(result.data.dungeons[1].kamas_per_hour, 300_000.0);
    }

    #[test]
    fn load_data_from_path_reads_explicit_file() {
        let path = temp_file_path("load-ok");
        let content = r#"{
            "zones": [{"name":"Plaine","session_time_seconds":3600.0,"session_total_kamas":100000.0,"kamas_per_hour":100000.0}],
            "dungeons": [],
            "duo_trios": [],
            "arenas": []
        }"#;

        fs::write(&path, content).unwrap();

        let result = load_data_from_path(&path).unwrap();

        assert_eq!(result.data.zones.len(), 1);
        assert_eq!(result.cleaned_legacy_entries, 0);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_data_from_path_reports_missing_file_cleanly() {
        let path = temp_file_path("missing");
        let error = load_data_from_path(&path).err().unwrap();

        assert!(error.contains("Impossible de lire"));
        assert!(error.contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn load_data_from_path_reports_invalid_json_cleanly() {
        let path = temp_file_path("invalid");
        fs::write(&path, "{ invalid json }").unwrap();

        let error = load_data_from_path(&path).err().unwrap();

        assert!(error.contains("Impossible de charger"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn parse_data_rejects_non_list_collection() {
        let error = parse_data(r#"{"zones": {}}"#).err().unwrap();

        assert!(error.contains("Le champ zones doit etre une liste."));
    }

    #[test]
    fn save_data_to_path_creates_backup_when_overwriting() {
        let path = temp_file_path("backup");
        fs::write(&path, r#"{"marker":"ancienne-version"}"#).unwrap();

        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Plaine".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 3_600.0,
                session_total_kamas: 100_000.0,
                kamas_per_hour: 1.0,
            }],
            ..AppData::default()
        };

        save_data_to_path(&data, &path).unwrap();

        let backup_path = backup_path_for(&path);
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        let reloaded = load_data_from_path(&path).unwrap();

        assert!(backup_content.contains("ancienne-version"));
        assert_eq!(reloaded.data.zones[0].kamas_per_hour, 100_000.0);

        fs::remove_file(path).unwrap();
        fs::remove_file(backup_path).unwrap();
    }

    #[test]
    fn save_data_to_path_sanitizes_and_sorts_entries_before_writing() {
        let path = temp_file_path("sanitize");
        let data = AppData {
            dungeons: vec![
                DungeonEntry {
                    name: " Lent ".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    run_time_minutes: 20.0,
                    gross_kamas_per_run: 120_000.0,
                    key_price: 20_000.0,
                    net_kamas_per_run: 1.0,
                    kamas_per_hour: 1.0,
                },
                DungeonEntry {
                    name: "Rapide".to_string(),
                    character_class: Some(DofusClass::Feca),
                    recorded_at: None,
                    run_time_minutes: 10.0,
                    gross_kamas_per_run: 80_000.0,
                    key_price: 10_000.0,
                    net_kamas_per_run: 1.0,
                    kamas_per_hour: 1.0,
                },
            ],
            ..AppData::default()
        };

        save_data_to_path(&data, &path).unwrap();
        let reloaded = load_data_from_path(&path).unwrap();

        assert_eq!(reloaded.data.dungeons[0].name, "Rapide");
        assert_eq!(reloaded.data.dungeons[0].net_kamas_per_run, 70_000.0);
        assert_eq!(reloaded.data.dungeons[0].kamas_per_hour, 420_000.0);
        assert_eq!(reloaded.data.dungeons[1].name, "Lent");
        assert_eq!(reloaded.data.dungeons[1].net_kamas_per_run, 100_000.0);
        assert_eq!(reloaded.data.dungeons[1].kamas_per_hour, 300_000.0);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn save_data_to_path_rejects_invalid_entries() {
        let path = temp_file_path("invalid-save");
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Plaine".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 0.0,
                session_total_kamas: 100_000.0,
                kamas_per_hour: 1.0,
            }],
            ..AppData::default()
        };

        let error = save_data_to_path(&data, &path).unwrap_err();

        assert!(error.contains("La duree de session doit etre superieur a 0."));
        assert!(!path.exists());
    }
}
