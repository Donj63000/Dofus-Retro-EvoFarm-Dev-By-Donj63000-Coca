use super::*;
use crate::models::{ArenaForm, DurationInput, ZoneForm};

struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> Self {
        let base = std::env::temp_dir();
        Self(create_workspace(&base).unwrap())
    }
    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(windows)]
#[test]
fn windows_junction_is_rejected_without_changing_its_target() {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    struct Junction(PathBuf);
    impl Drop for Junction {
        fn drop(&mut self) {
            // Je retire seulement la jonction, avant de nettoyer le dossier de test.
            let _ = fs::remove_dir(&self.0);
        }
    }

    let dir = TestDirectory::new();
    let target = dir.file("cible");
    fs::create_dir(&target).unwrap();
    let original = target.join("data.json");
    fs::write(&original, b"donnees a conserver").unwrap();
    let junction = Junction(dir.file("jonction"));
    // Je passe les chemins comme données, sans les interpoler dans la commande.
    let result = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction -Path $env:EVOFARM_TEST_JUNCTION -Target $env:EVOFARM_TEST_TARGET | Out-Null",
        ])
        .env("EVOFARM_TEST_JUNCTION", &junction.0)
        .env("EVOFARM_TEST_TARGET", &target)
        .creation_flags(0x0800_0000)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(is_link_metadata(
        &fs::symlink_metadata(&junction.0).unwrap()
    ));
    assert!(open_regular_file(&junction.0).is_err());
    assert!(list_named_saves_in_dir(&junction.0).is_err());
    assert!(save_state_to_path(
        &state_with_zone("Refusee", 1.0),
        &junction.0.join("data.json")
    )
    .is_err());
    assert_eq!(fs::read(&original).unwrap(), b"donnees a conserver");
    assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
}

fn state_with_zone(name: &str, amount: f32) -> PersistedState {
    PersistedState {
        data: AppData {
            zones: vec![ZoneEntry {
                name: name.to_string(),
                session_time_seconds: 3600.0,
                session_total_kamas: amount,
                ..ZoneEntry::default()
            }],
            ..AppData::default()
        },
        ..PersistedState::default()
    }
}

#[test]
fn bounded_serializer_accepts_exact_limit_and_rejects_one_extra_byte() {
    let text = "x".repeat(MAX_STATE_FILE_BYTES as usize - 2);
    let bytes = serialize_state_bounded(&text).unwrap();
    assert_eq!(bytes.len() as u64, MAX_STATE_FILE_BYTES);
    assert!(serialize_state_bounded(&(text + "x")).is_err());
}

#[test]
fn reader_accepts_exact_limit_and_rejects_one_extra_byte() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    let mut bytes = vec![b' '; MAX_STATE_FILE_BYTES as usize];
    bytes[..2].copy_from_slice(b"{}");
    fs::write(&path, &bytes).unwrap();
    assert!(load_state_from_path(&path).is_ok());
    bytes.push(b' ');
    fs::write(&path, bytes).unwrap();
    assert!(load_state_from_path(&path).is_err());
}

#[test]
fn rejected_raw_write_does_not_touch_primary_or_backup() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    let backup = backup_path_for(&path);
    fs::write(&path, b"principal").unwrap();
    fs::write(&backup, b"secours").unwrap();
    assert!(write_atomic(&path, &vec![b' '; MAX_STATE_FILE_BYTES as usize + 1]).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"principal");
    assert_eq!(fs::read(&backup).unwrap(), b"secours");
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 2);
}

#[test]
fn compact_import_that_expands_past_limit_cannot_replace_either_save() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    save_state_to_path(&state_with_zone("A", 10.0), &path).unwrap();
    save_state_to_path(&state_with_zone("B", 20.0), &path).unwrap();
    let before = fs::read(&path).unwrap();
    let before_backup = fs::read(backup_path_for(&path)).unwrap();
    let entry = ArenaEntry {
        name: "A".repeat(MAX_NAME_BYTES),
        round_time_minutes: 1.0,
        seat_price: 1.0,
        seats_sold: 1,
        capture_price: 1.0,
        captures_count: 1,
        ..ArenaEntry::default()
    };
    let candidate = PersistedState {
        data: AppData {
            arenas: vec![entry; MAX_TOTAL_ENTRIES],
            ..AppData::default()
        },
        ..PersistedState::default()
    };
    let compact = serde_json::to_string(&candidate).unwrap();
    assert!((compact.len() as u64) < MAX_STATE_FILE_BYTES);
    let (imported, _) = parse_state(&compact).unwrap();
    assert!(save_state_to_path(&imported, &path).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read(backup_path_for(&path)).unwrap(), before_backup);
}

#[test]
fn oversized_named_save_is_rejected_without_creating_a_file() {
    let dir = TestDirectory::new();
    let mut state = state_with_zone("A", 1.0);
    state.drafts.arena_form = Some(ArenaForm {
        name: "x".repeat(MAX_DRAFT_FIELD_BYTES + 1),
        ..ArenaForm::default()
    });
    assert!(save_named_state_in_dir("Nom", &state, None, &dir.0).is_err());
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}

#[test]
fn total_session_quota_is_shared_by_all_collections() {
    let mut state = state_with_zone("A", 1.0);
    state.data.zones = vec![state.data.zones[0].clone(); MAX_TOTAL_ENTRIES + 1];
    assert!(sanitize_persisted_state(&state).is_err());
    let json = serde_json::to_string(&state).unwrap();
    assert!(parse_state(&json).is_err());
}

#[test]
fn unknown_or_truncated_schema_versions_are_not_normalized() {
    for version in ["0", "3", "4294967298", "-1", "2.5", "null", "\"2\""] {
        assert!(parse_state(&format!(r#"{{"schema_version":{version},"data":{{}}}}"#)).is_err());
    }
    assert!(parse_state(r#"{"schema_version":1}"#).is_ok());
    assert!(parse_state(r#"{"schema_version":2}"#).is_ok());
}

#[test]
fn saved_inline_draft_follows_original_row_through_recalculation_and_sort() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    let mut state = state_with_zone("Lente", 10.0);
    state
        .data
        .zones
        .push(state_with_zone("Rapide", 1000.0).data.zones.remove(0));
    state.drafts.zone_edit = Some(PersistedInlineEdit {
        index: 0,
        form: ZoneForm {
            name: "Brouillon lent".to_string(),
            ..ZoneForm::default()
        },
    });
    save_state_to_path(&state, &path).unwrap();
    let restored = load_state_from_path(&path).unwrap().state;
    let edit = restored.drafts.zone_edit.as_ref().unwrap();
    assert_eq!(edit.index, 1);
    assert_eq!(restored.data.zones[edit.index].name, "Lente");
    save_state_to_path(&restored, &path).unwrap();
    let again = load_state_from_path(&path).unwrap().state;
    assert_eq!(again.drafts.zone_edit.as_ref().unwrap().index, 1);
}

#[test]
fn legacy_cleanup_remaps_draft_index_before_sorting() {
    let mut raw = serde_json::to_value(state_with_zone("Cible", 100.0)).unwrap();
    raw["data"]["zones"].as_array_mut().unwrap().insert(
        0,
        serde_json::json!({
            "name": "Ancienne", "combat_time_seconds": 100, "kamas_per_combat": 1
        }),
    );
    raw["drafts"]["zone_edit"] = serde_json::json!({
        "index": 1, "form": {"name": "À préserver"}
    });
    let (state, removed) = parse_state(&raw.to_string()).unwrap();
    assert_eq!(removed, 1);
    let edit = state.drafts.zone_edit.unwrap();
    assert_eq!(edit.index, 0);
    assert_eq!(state.data.zones[edit.index].name, "Cible");
}

#[test]
fn file_identity_cannot_be_redirected_by_embedded_metadata() {
    let dir = TestDirectory::new();
    let saved = save_named_state_in_dir("A", &state_with_zone("A", 1.0), None, &dir.0).unwrap();
    let mut raw: Value = serde_json::from_str(&fs::read_to_string(&saved.path).unwrap()).unwrap();
    raw["meta"]["save_id"] = Value::String("other-save".to_string());
    fs::write(&saved.path, raw.to_string()).unwrap();
    assert!(load_named_save_in_dir(&saved.meta.save_id, &dir.0).is_err());
    assert!(list_named_saves_in_dir(&dir.0).unwrap().is_empty());
}

#[test]
fn orphaned_named_backup_still_appears_and_can_be_loaded() {
    let dir = TestDirectory::new();
    let saved = save_named_state_in_dir("A", &state_with_zone("A", 1.0), None, &dir.0).unwrap();
    fs::rename(&saved.path, backup_path_for(&saved.path)).unwrap();
    let listed = list_named_saves_in_dir(&dir.0).unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].used_backup);
    assert!(
        load_named_save_in_dir(&saved.meta.save_id, &dir.0)
            .unwrap()
            .used_backup
    );
}

#[test]
fn failed_install_and_failed_rollback_keep_both_recovery_files() {
    let dir = TestDirectory::new();
    let target = dir.file("data.json");
    let pending = dir.file("new.json");
    let recovery = dir.file("old.json");
    fs::write(&target, b"old").unwrap();
    fs::write(&pending, b"new").unwrap();
    let mut calls = 0;
    let error = replace_file_with_rollback_using(&pending, &target, &recovery, |from, to| {
        calls += 1;
        if calls == 1 {
            fs::rename(from, to)
        } else {
            Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "panne injectée",
            ))
        }
    })
    .unwrap_err();
    assert_eq!(calls, 3);
    assert_eq!(fs::read(&recovery).unwrap(), b"old");
    assert_eq!(fs::read(&pending).unwrap(), b"new");
    assert!(!target.exists());
    assert!(error.contains(&recovery.display().to_string()));
}

#[test]
fn failed_install_with_successful_rollback_restores_previous_bytes() {
    let dir = TestDirectory::new();
    let target = dir.file("data.json");
    let pending = dir.file("new.json");
    let recovery = dir.file("old.json");
    fs::write(&target, b"old").unwrap();
    fs::write(&pending, b"new").unwrap();
    let mut calls = 0;
    assert!(
        replace_file_with_rollback_using(&pending, &target, &recovery, |from, to| {
            calls += 1;
            if calls == 2 {
                Err(io::Error::other("panne injectée"))
            } else {
                fs::rename(from, to)
            }
        })
        .is_err()
    );
    assert_eq!(fs::read(&target).unwrap(), b"old");
    assert_eq!(fs::read(&pending).unwrap(), b"new");
}

#[test]
fn preexisting_recovery_file_is_never_removed() {
    let dir = TestDirectory::new();
    let target = dir.file("data.json");
    let pending = dir.file("new.json");
    let recovery = dir.file("old.json");
    fs::write(&target, b"old").unwrap();
    fs::write(&pending, b"new").unwrap();
    fs::write(&recovery, b"precious").unwrap();
    assert!(replace_file_with_rollback(&pending, &target, &recovery).is_err());
    assert_eq!(fs::read(&recovery).unwrap(), b"precious");
    assert_eq!(fs::read(&target).unwrap(), b"old");
}

#[test]
fn corrupt_primary_does_not_overwrite_valid_backup_during_repair() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    save_state_to_path(&state_with_zone("Secours", 10.0), &path).unwrap();
    save_state_to_path(&state_with_zone("Courant", 20.0), &path).unwrap();
    let expected = fs::read(backup_path_for(&path)).unwrap();
    fs::write(&path, b"corrupt").unwrap();
    save_state_to_path(&state_with_zone("Repare", 30.0), &path).unwrap();
    assert_eq!(fs::read(backup_path_for(&path)).unwrap(), expected);
    assert_eq!(
        load_state_from_path(&path).unwrap().state.data.zones[0].name,
        "Repare"
    );
}

#[cfg(unix)]
#[test]
fn symlink_backup_cannot_overwrite_another_file() {
    use std::os::unix::fs::symlink;
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    let victim = dir.file("victim.txt");
    save_state_to_path(&state_with_zone("Avant", 10.0), &path).unwrap();
    let before = fs::read(&path).unwrap();
    fs::write(&victim, b"do not touch").unwrap();
    symlink(&victim, backup_path_for(&path)).unwrap();
    assert!(save_state_to_path(&state_with_zone("Apres", 20.0), &path).is_err());
    assert_eq!(fs::read(&victim).unwrap(), b"do not touch");
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn symbolic_primary_and_broken_symbolic_paths_are_rejected() {
    use std::os::unix::fs::symlink;
    let dir = TestDirectory::new();
    let target = dir.file("real.json");
    let link = dir.file("link.json");
    fs::write(&target, b"{}").unwrap();
    symlink(&target, &link).unwrap();
    assert!(read_state_file_limited(&link).is_err());
    fs::remove_file(&target).unwrap();
    assert!(write_atomic(&link, b"{}").is_err());
    assert!(!target.exists());
}

#[cfg(unix)]
#[test]
fn hardlinked_backup_never_truncates_the_other_link() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    let victim = dir.file("victim.txt");
    save_state_to_path(&state_with_zone("Avant", 10.0), &path).unwrap();
    fs::write(&victim, b"do not touch").unwrap();
    fs::hard_link(&victim, backup_path_for(&path)).unwrap();
    save_state_to_path(&state_with_zone("Apres", 20.0), &path).unwrap();
    assert_eq!(fs::read(&victim).unwrap(), b"do not touch");
}

#[cfg(unix)]
#[test]
fn newly_created_files_and_directories_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TestDirectory::new();
    let path = dir.file("private/data.json");
    save_state_to_path(&state_with_zone("A", 10.0), &path).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
    assert_eq!(
        fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o077,
        0
    );
}

#[test]
fn long_draft_duration_segments_are_rejected_without_parsing_overflow() {
    let mut state = state_with_zone("A", 10.0);
    state.drafts.zone_form = Some(ZoneForm {
        session_time: DurationInput {
            hours: "9".repeat(MAX_DRAFT_FIELD_BYTES + 1),
            ..DurationInput::default()
        },
        ..ZoneForm::default()
    });
    assert!(sanitize_persisted_state(&state).is_err());
}

#[test]
fn oversized_corrupt_primary_can_be_repaired_from_a_valid_backup() {
    let dir = TestDirectory::new();
    let path = dir.file("data.json");
    save_state_to_path(&state_with_zone("Secours", 10.0), &path).unwrap();
    save_state_to_path(&state_with_zone("Courant", 20.0), &path).unwrap();
    let expected = fs::read(backup_path_for(&path)).unwrap();
    fs::write(&path, vec![b'x'; MAX_STATE_FILE_BYTES as usize + 1]).unwrap();
    assert!(load_state_from_path(&path).unwrap().used_backup);
    save_state_to_path(&state_with_zone("Repare", 30.0), &path).unwrap();
    assert_eq!(fs::read(backup_path_for(&path)).unwrap(), expected);
    assert!(!load_state_from_path(&path).unwrap().used_backup);
}
