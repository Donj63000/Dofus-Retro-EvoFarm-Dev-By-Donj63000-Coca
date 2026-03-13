use crate::calculations::{
    sanitize_arena_entry, sanitize_dungeon_entry, sanitize_duo_trio_entry, sanitize_zone_entry,
};
use crate::models::{
    AppData, ArenaEntry, DraftState, DungeonEntry, DuoTrioEntry, PersistedState, ZoneEntry,
    PERSISTED_STATE_VERSION,
};
use serde_json::{Map, Value};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LoadDataResult {
    pub data: AppData,
    pub cleaned_legacy_entries: usize,
}

pub struct LoadStateResult {
    pub state: PersistedState,
    pub cleaned_legacy_entries: usize,
    pub source_path: PathBuf,
    pub used_backup: bool,
}

pub struct SaveStateResult {
    pub path: PathBuf,
    pub backup_path: Option<PathBuf>,
}

pub fn data_file_path() -> PathBuf {
    preferred_data_file_path()
}

pub fn data_backup_path() -> PathBuf {
    backup_path_for(&data_file_path())
}

pub fn has_local_state() -> bool {
    let preferred_path = preferred_data_file_path();
    let legacy_path = legacy_data_file_path();

    preferred_path.exists()
        || data_backup_path().exists()
        || legacy_path.exists()
        || backup_path_for(&legacy_path).exists()
}

#[allow(dead_code)]
pub fn load_data() -> Result<LoadDataResult, String> {
    let result = load_state()?;
    Ok(LoadDataResult {
        data: result.state.data,
        cleaned_legacy_entries: result.cleaned_legacy_entries,
    })
}

pub fn load_data_from_path(path: &Path) -> Result<LoadDataResult, String> {
    let result = load_state_from_path(path)?;
    Ok(LoadDataResult {
        data: result.state.data,
        cleaned_legacy_entries: result.cleaned_legacy_entries,
    })
}

#[allow(dead_code)]
pub fn save_data(data: &AppData) -> Result<PathBuf, String> {
    let result = save_state(&PersistedState {
        data: data.clone(),
        ..PersistedState::default()
    })?;
    Ok(result.path)
}

pub fn save_data_to_path(data: &AppData, path: &Path) -> Result<PathBuf, String> {
    let result = save_state_to_path(
        &PersistedState {
            data: data.clone(),
            ..PersistedState::default()
        },
        path,
    )?;
    Ok(result.path)
}

pub fn load_state() -> Result<LoadStateResult, String> {
    let preferred_path = preferred_data_file_path();
    let preferred_backup = backup_path_for(&preferred_path);

    if preferred_path.exists() || preferred_backup.exists() {
        return load_state_from_path(&preferred_path);
    }

    let legacy_path = legacy_data_file_path();
    let legacy_backup = backup_path_for(&legacy_path);

    if legacy_path.exists() || legacy_backup.exists() {
        return load_state_from_path(&legacy_path);
    }

    load_state_from_path(&preferred_path)
}

pub fn load_state_from_path(path: &Path) -> Result<LoadStateResult, String> {
    match read_and_parse_state(path) {
        Ok((state, cleaned_legacy_entries)) => Ok(LoadStateResult {
            state,
            cleaned_legacy_entries,
            source_path: path.to_path_buf(),
            used_backup: false,
        }),
        Err(primary_error) => {
            let backup_path = backup_path_for(path);

            if !backup_path.exists() {
                return Err(primary_error);
            }

            match read_and_parse_state(&backup_path) {
                Ok((state, cleaned_legacy_entries)) => Ok(LoadStateResult {
                    state,
                    cleaned_legacy_entries,
                    source_path: backup_path,
                    used_backup: true,
                }),
                Err(backup_error) => Err(format!(
                    "{primary_error} Sauvegarde de secours invalide : {backup_error}"
                )),
            }
        }
    }
}

pub fn save_state(state: &PersistedState) -> Result<SaveStateResult, String> {
    let path = data_file_path();
    save_state_to_path(state, &path)
}

pub fn delete_state() -> Result<(), String> {
    let preferred_path = preferred_data_file_path();
    delete_state_at_path(&preferred_path)?;

    let legacy_path = legacy_data_file_path();
    if legacy_path != preferred_path {
        delete_state_at_path(&legacy_path)?;
    }

    Ok(())
}

pub fn delete_state_at_path(path: &Path) -> Result<(), String> {
    remove_file_if_exists(path)?;
    remove_file_if_exists(&backup_path_for(path))?;
    Ok(())
}

pub fn save_state_to_path(state: &PersistedState, path: &Path) -> Result<SaveStateResult, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Impossible de créer le dossier {} : {error}",
                parent.display()
            )
        })?;
    }

    let sanitized = sanitize_persisted_state(state)?;
    let json = serde_json::to_vec_pretty(&sanitized)
        .map_err(|error| format!("Impossible de sérialiser la sauvegarde : {error}"))?;
    let backup_path = write_atomic(path, &json)?;

    Ok(SaveStateResult {
        path: path.to_path_buf(),
        backup_path,
    })
}

fn parse_data(content: &str) -> Result<LoadDataResult, String> {
    let (state, cleaned_legacy_entries) = parse_state(content)?;
    Ok(LoadDataResult {
        data: state.data,
        cleaned_legacy_entries,
    })
}

fn preferred_data_file_path() -> PathBuf {
    app_data_file_path("MarkarthFarm")
}

fn legacy_data_file_path() -> PathBuf {
    app_data_file_path("dofus_rentabilite")
}

fn app_data_file_path(app_directory: &str) -> PathBuf {
    let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push(app_directory);
    dir.push("data.json");
    dir
}

fn read_and_parse_state(path: &Path) -> Result<(PersistedState, usize), String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Impossible de lire {} : {error}", path.display()))?;

    parse_state(&content)
        .map_err(|error| format!("Impossible de charger {} : {error}", path.display()))
}

fn parse_state(content: &str) -> Result<(PersistedState, usize), String> {
    let raw: Value = serde_json::from_str(content).map_err(|error| error.to_string())?;

    if !raw.is_object() {
        return Err("La racine du fichier doit être un objet JSON.".to_string());
    }

    if is_state_envelope(&raw) {
        parse_state_envelope(&raw)
    } else {
        let (data, cleaned_legacy_entries) = parse_app_data_value(&raw)?;
        Ok((
            PersistedState {
                schema_version: PERSISTED_STATE_VERSION,
                data,
                drafts: DraftState::default(),
            },
            cleaned_legacy_entries,
        ))
    }
}

fn is_state_envelope(raw: &Value) -> bool {
    raw.get("schema_version").is_some() || raw.get("data").is_some() || raw.get("drafts").is_some()
}

fn parse_state_envelope(raw: &Value) -> Result<(PersistedState, usize), String> {
    let data_value = raw
        .get("data")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));

    let drafts = match raw.get("drafts") {
        Some(value) if !value.is_null() => serde_json::from_value::<DraftState>(value.clone())
            .map_err(|error| format!("La section drafts est invalide : {error}"))?,
        _ => DraftState::default(),
    };

    let (data, cleaned_legacy_entries) = parse_app_data_value(&data_value)?;

    let mut state = PersistedState {
        schema_version: raw
            .get("schema_version")
            .and_then(Value::as_u64)
            .map(|value| value as u32)
            .unwrap_or(PERSISTED_STATE_VERSION),
        data,
        drafts,
    };

    sanitize_loaded_state(&mut state);

    Ok((state, cleaned_legacy_entries))
}

fn parse_app_data_value(raw: &Value) -> Result<(AppData, usize), String> {
    let object = raw
        .as_object()
        .ok_or_else(|| "La section data doit être un objet JSON.".to_string())?;

    let has_known_key = ["zones", "dungeons", "duo_trios", "arenas"]
        .iter()
        .any(|key| object.contains_key(*key));

    if !has_known_key && !object.is_empty() {
        return Err(
            "Aucune collection reconnue trouvée. Champs attendus : zones, dungeons, duo_trios, arenas."
                .to_string(),
        );
    }

    let (zones, cleaned_zones) = parse_collection(raw, "zones", is_legacy_zone_entry, |item| {
        let entry = serde_json::from_value::<ZoneEntry>(item).map_err(|error| error.to_string())?;
        sanitize_zone_entry(entry)
    })?;

    let (dungeons, cleaned_dungeons) = parse_collection(
        raw,
        "dungeons",
        |_| false,
        |item| {
            let entry =
                serde_json::from_value::<DungeonEntry>(item).map_err(|error| error.to_string())?;
            sanitize_dungeon_entry(entry)
        },
    )?;

    let (duo_trios, cleaned_duo_trios) = parse_collection(
        raw,
        "duo_trios",
        |_| false,
        |item| {
            let entry =
                serde_json::from_value::<DuoTrioEntry>(item).map_err(|error| error.to_string())?;
            sanitize_duo_trio_entry(entry)
        },
    )?;

    let (arenas, cleaned_arenas) = parse_collection(
        raw,
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

    Ok((
        data,
        cleaned_zones + cleaned_dungeons + cleaned_duo_trios + cleaned_arenas,
    ))
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
        .ok_or_else(|| format!("Le champ {key} doit être une liste."))?;

    let mut entries = Vec::new();
    let mut cleaned = 0usize;

    for item in items {
        let object = item
            .as_object()
            .ok_or_else(|| format!("Une entrée de {key} est invalide."))?;

        if object.contains_key("recorded_on") || is_legacy_entry(object) {
            cleaned += 1;
            continue;
        }

        let entry = parse_entry(item.clone())
            .map_err(|error| format!("Entrée invalide dans {key} : {error}"))?;
        entries.push(entry);
    }

    Ok((entries, cleaned))
}

fn sanitize_persisted_state(state: &PersistedState) -> Result<PersistedState, String> {
    let mut sanitized = PersistedState {
        schema_version: PERSISTED_STATE_VERSION,
        data: sanitize_app_data(&state.data)?,
        drafts: state.drafts.clone(),
    };

    sanitize_loaded_state(&mut sanitized);
    Ok(sanitized)
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

fn sanitize_loaded_state(state: &mut PersistedState) {
    state.schema_version = PERSISTED_STATE_VERSION;
    sort_loaded_data(&mut state.data);

    if state
        .drafts
        .zone_edit
        .as_ref()
        .is_some_and(|edit| edit.index >= state.data.zones.len())
    {
        state.drafts.zone_edit = None;
    }

    if state
        .drafts
        .dungeon_edit
        .as_ref()
        .is_some_and(|edit| edit.index >= state.data.dungeons.len())
    {
        state.drafts.dungeon_edit = None;
    }

    if state
        .drafts
        .duo_trio_edit
        .as_ref()
        .is_some_and(|edit| edit.index >= state.data.duo_trios.len())
    {
        state.drafts.duo_trio_edit = None;
    }

    if state
        .drafts
        .arena_edit
        .as_ref()
        .is_some_and(|edit| edit.index >= state.data.arenas.len())
    {
        state.drafts.arena_edit = None;
    }
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

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Impossible de supprimer {} : {error}",
            path.display()
        )),
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<Option<PathBuf>, String> {
    let temp_path = temp_path_for(path);
    let swap_path = swap_path_for(path);
    let backup_path = backup_path_for(path);

    let result = (|| -> Result<Option<PathBuf>, String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| {
                format!(
                    "Impossible de créer le fichier temporaire {} : {error}",
                    temp_path.display()
                )
            })?;

        file.write_all(bytes).map_err(|error| {
            format!(
                "Impossible d'écrire le fichier temporaire {} : {error}",
                temp_path.display()
            )
        })?;

        file.sync_all().map_err(|error| {
            format!(
                "Impossible de synchroniser le fichier temporaire {} : {error}",
                temp_path.display()
            )
        })?;

        let backup_created = if path.exists() {
            fs::copy(path, &backup_path).map_err(|error| {
                format!(
                    "Impossible de créer la sauvegarde de secours {} : {error}",
                    backup_path.display()
                )
            })?;
            Some(backup_path.clone())
        } else {
            None
        };

        replace_file_with_rollback(&temp_path, path, &swap_path)?;
        Ok(backup_created)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
        let _ = fs::remove_file(&swap_path);
    }

    result
}

fn replace_file_with_rollback(
    temp_path: &Path,
    path: &Path,
    swap_path: &Path,
) -> Result<(), String> {
    let had_existing = path.exists();

    if had_existing {
        if swap_path.exists() {
            fs::remove_file(swap_path).map_err(|error| {
                format!(
                    "Impossible de nettoyer le fichier temporaire de remplacement {} : {error}",
                    swap_path.display()
                )
            })?;
        }

        fs::rename(path, swap_path).map_err(|error| {
            format!(
                "Impossible de préparer le remplacement de {} : {error}",
                path.display()
            )
        })?;
    }

    match fs::rename(temp_path, path) {
        Ok(()) => {
            if had_existing && swap_path.exists() {
                let _ = fs::remove_file(swap_path);
            }
            Ok(())
        }
        Err(error) => {
            let rollback_error = if had_existing && swap_path.exists() && !path.exists() {
                fs::rename(swap_path, path).err()
            } else {
                None
            };

            match rollback_error {
                Some(rollback_error) => Err(format!(
                    "Impossible de finaliser la sauvegarde vers {} : {error}. La restauration automatique a aussi échoué : {rollback_error}",
                    path.display()
                )),
                None => Err(format!(
                    "Impossible de finaliser la sauvegarde vers {} : {error}",
                    path.display()
                )),
            }
        }
    }
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

fn swap_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("data.json");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    path.with_file_name(format!("{file_name}.swap-{}-{unique}", std::process::id()))
}

fn is_legacy_zone_entry(value: &Map<String, Value>) -> bool {
    value.contains_key("combat_time_seconds") || value.contains_key("kamas_per_combat")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DofusClass, DraftState, DurationInput, PartyMode, PersistedInlineEdit, PersistedState,
        ZoneForm,
    };

    fn temp_file_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("markarthfarm-{name}-{unique}.json"))
    }

    fn cleanup_temp_state(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(backup_path_for(path));
    }

    fn sample_zone_draft() -> ZoneForm {
        ZoneForm {
            name: "Brouillon Craqueleurs".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            session_time: DurationInput {
                hours: "01".to_string(),
                minutes: "10".to_string(),
                seconds: "00".to_string(),
            },
            session_total_kamas: "250000".to_string(),
        }
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

        cleanup_temp_state(&path);
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

        cleanup_temp_state(&path);
    }

    #[test]
    fn parse_data_rejects_non_list_collection() {
        let error = parse_data(r#"{"zones": {}}"#).err().unwrap();

        assert!(error.contains("Le champ zones doit être une liste."));
    }

    #[test]
    fn parse_state_rejects_non_object_roots() {
        let error = parse_state(r#"[]"#).err().unwrap();
        assert_eq!(error, "La racine du fichier doit être un objet JSON.");
    }

    #[test]
    fn parse_state_rejects_unknown_root_shape() {
        let error = parse_state(r#"{"foo":"bar"}"#).err().unwrap();

        assert!(error.contains("Aucune collection reconnue trouvée"));
    }

    #[test]
    fn parse_state_envelope_defaults_missing_sections() {
        let (state, cleaned_legacy_entries) = parse_state(r#"{"schema_version":1}"#).unwrap();

        assert_eq!(cleaned_legacy_entries, 0);
        assert_eq!(state.schema_version, PERSISTED_STATE_VERSION);
        assert!(state.data.zones.is_empty());
        assert!(!state.drafts.has_any_draft());
    }

    #[test]
    fn parse_state_envelope_rejects_invalid_drafts_section() {
        let error = parse_state(r#"{"data":{"zones":[]},"drafts":{"zone_form":42}}"#)
            .err()
            .unwrap();

        assert!(error.contains("La section drafts est invalide"));
    }

    #[test]
    fn parse_state_envelope_counts_legacy_entries_in_data_section() {
        let content = r#"{
            "schema_version": 2,
            "data": {
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
                ]
            }
        }"#;

        let (state, cleaned_legacy_entries) = parse_state(content).unwrap();

        assert_eq!(cleaned_legacy_entries, 1);
        assert_eq!(state.data.zones.len(), 1);
        assert_eq!(state.data.zones[0].name, "Zone moderne");
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

        cleanup_temp_state(&path);
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

        cleanup_temp_state(&path);
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

    #[test]
    fn save_state_to_path_on_first_write_does_not_create_backup() {
        let path = temp_file_path("state-first-write");

        let result = save_state_to_path(&PersistedState::default(), &path).unwrap();

        assert_eq!(result.path, path);
        assert!(result.backup_path.is_none());
        assert!(path.exists());
        assert!(!backup_path_for(&path).exists());

        cleanup_temp_state(&path);
    }

    #[test]
    fn save_and_reload_state_roundtrips_form_draft() {
        let path = temp_file_path("state-roundtrip");

        let state = PersistedState {
            data: AppData::default(),
            drafts: DraftState {
                zone_form: Some(sample_zone_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        save_state_to_path(&state, &path).unwrap();
        let reloaded = load_state_from_path(&path).unwrap();

        assert_eq!(
            reloaded.state.drafts.zone_form.as_ref().unwrap().name,
            "Brouillon Craqueleurs"
        );

        cleanup_temp_state(&path);
    }

    #[test]
    fn load_state_falls_back_to_backup_when_main_file_is_invalid() {
        let path = temp_file_path("main-invalid");
        let backup = backup_path_for(&path);

        fs::write(&path, "{ invalid json }").unwrap();

        let state = PersistedState {
            data: AppData {
                zones: vec![ZoneEntry {
                    name: "Plaine".to_string(),
                    character_class: None,
                    recorded_at: None,
                    session_time_seconds: 3600.0,
                    session_total_kamas: 100000.0,
                    kamas_per_hour: 100000.0,
                }],
                ..AppData::default()
            },
            ..PersistedState::default()
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        fs::write(&backup, json).unwrap();

        let result = load_state_from_path(&path).unwrap();

        assert!(result.used_backup);
        assert_eq!(result.state.data.zones.len(), 1);

        cleanup_temp_state(&path);
    }

    #[test]
    fn load_state_from_path_accepts_legacy_app_data_shape() {
        let path = temp_file_path("legacy-shape");
        let content = r#"{
            "zones": [
                {
                    "name": "Plaine",
                    "session_time_seconds": 3600.0,
                    "session_total_kamas": 100000.0,
                    "kamas_per_hour": 100000.0
                }
            ],
            "dungeons": [],
            "duo_trios": [],
            "arenas": []
        }"#;

        fs::write(&path, content).unwrap();

        let result = load_state_from_path(&path).unwrap();

        assert_eq!(result.state.schema_version, PERSISTED_STATE_VERSION);
        assert_eq!(result.state.data.zones.len(), 1);
        assert!(!result.used_backup);

        cleanup_temp_state(&path);
    }

    #[test]
    fn load_data_from_path_reads_state_envelope_and_ignores_drafts() {
        let path = temp_file_path("load-envelope");
        let state = PersistedState {
            data: AppData {
                zones: vec![ZoneEntry {
                    name: "Plaine".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 3_600.0,
                    session_total_kamas: 100_000.0,
                    kamas_per_hour: 100_000.0,
                }],
                ..AppData::default()
            },
            drafts: DraftState {
                zone_form: Some(sample_zone_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        save_state_to_path(&state, &path).unwrap();
        let result = load_data_from_path(&path).unwrap();

        assert_eq!(result.data.zones.len(), 1);
        assert_eq!(result.cleaned_legacy_entries, 0);

        cleanup_temp_state(&path);
    }

    #[test]
    fn load_state_from_path_reports_invalid_primary_and_backup() {
        let path = temp_file_path("invalid-with-invalid-backup");
        let backup_path = backup_path_for(&path);

        fs::write(&path, "{ invalid json }").unwrap();
        fs::write(&backup_path, "[]").unwrap();

        let error = load_state_from_path(&path).err().unwrap();

        assert!(error.contains("Impossible de charger"));
        assert!(error.contains("Sauvegarde de secours invalide"));

        cleanup_temp_state(&path);
    }

    #[test]
    fn delete_state_at_path_removes_primary_and_backup_files() {
        let path = temp_file_path("delete-state");
        let backup_path = backup_path_for(&path);

        fs::write(&path, "{}").unwrap();
        fs::write(&backup_path, "{}").unwrap();

        delete_state_at_path(&path).unwrap();

        assert!(!path.exists());
        assert!(!backup_path.exists());
    }

    #[test]
    fn delete_state_at_path_is_a_no_op_when_files_are_missing() {
        let path = temp_file_path("delete-missing");

        delete_state_at_path(&path).unwrap();

        assert!(!path.exists());
        assert!(!backup_path_for(&path).exists());
    }

    #[test]
    fn delete_state_at_path_reports_error_for_directory_target() {
        let path = temp_file_path("delete-directory-target");
        fs::create_dir_all(&path).unwrap();

        let error = delete_state_at_path(&path).err().unwrap();

        assert!(error.contains("Impossible de supprimer"));

        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn sanitize_loaded_state_drops_out_of_bounds_inline_edit() {
        let path = temp_file_path("edit-bounds");
        let state = PersistedState {
            data: AppData::default(),
            drafts: DraftState {
                zone_edit: Some(PersistedInlineEdit {
                    index: 2,
                    form: ZoneForm::default(),
                }),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        save_state_to_path(&state, &path).unwrap();
        let result = load_state_from_path(&path).unwrap();

        assert!(result.state.drafts.zone_edit.is_none());

        cleanup_temp_state(&path);
    }

    #[test]
    fn save_state_to_path_normalizes_schema_and_drops_out_of_bounds_edits() {
        let path = temp_file_path("state-normalized");
        let state = PersistedState {
            schema_version: 1,
            drafts: DraftState {
                zone_edit: Some(PersistedInlineEdit {
                    index: 99,
                    form: sample_zone_draft(),
                }),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        save_state_to_path(&state, &path).unwrap();
        let result = load_state_from_path(&path).unwrap();

        assert_eq!(result.state.schema_version, PERSISTED_STATE_VERSION);
        assert!(result.state.drafts.zone_edit.is_none());

        cleanup_temp_state(&path);
    }
}
