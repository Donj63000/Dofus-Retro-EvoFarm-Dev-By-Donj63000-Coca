//! Je partage le même mois fictif entre les captures, l'export et les tests métier.

use crate::models::*;
use crate::{calculations, reports, storage, MyApp};
use chrono::{Duration, NaiveDate, NaiveDateTime};
use std::path::Path;

pub fn reference_time() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 9, 12)
        .unwrap()
        .and_hms_opt(23, 59, 0)
        .unwrap()
}

fn duration(minutes: usize) -> DurationInput {
    DurationInput {
        hours: format!("{:02}", minutes / 60),
        minutes: format!("{:02}", minutes % 60),
        seconds: "00".into(),
    }
}

pub fn month_state() -> PersistedState {
    let start = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
    let days = [
        0, 1, 3, 4, 6, 7, 9, 10, 11, 13, 14, 16, 17, 19, 20, 22, 23, 25, 27, 29,
    ];
    let counts = [2, 4, 3, 2, 4, 3, 3, 2, 4, 3, 2, 4, 3, 3, 2, 4, 3, 2, 4, 3];
    // Je répartis chaque cycle en six zones, quatre donjons, trois captures et deux arènes.
    let activities = [0, 1, 0, 2, 3, 0, 1, 2, 0, 1, 3, 0, 2, 1, 0];
    let classes = [
        DofusClass::Cra,
        DofusClass::Cra,
        DofusClass::Sadida,
        DofusClass::Feca,
        DofusClass::Cra,
        DofusClass::Sadida,
    ];
    let mut data = AppData::default();
    let mut index = 0;
    for (day_index, (&day, &count)) in days.iter().zip(&counts).enumerate() {
        for slot in 0..count {
            let recorded_at = (start + Duration::days(day))
                .and_hms_opt(17, 45, 0)
                .unwrap()
                + Duration::minutes((day_index % 4 * 10 + slot * 95) as i64);
            let date = recorded_at.format("%Y-%m-%d %H:%M").to_string();
            let class = Some(classes[(index + index / 15) % classes.len()]);
            match activities[index % activities.len()] {
                0 => {
                    let n = data.zones.len();
                    let form = ZoneForm {
                        name: [
                            "Plaines de Cania",
                            "Prairies des Blops",
                            "Coin des Scarafeuilles",
                        ][n / 3 % 3]
                            .into(),
                        character_class: class,
                        recorded_at_input: date,
                        session_time: duration(45 + n * 7 % 40),
                        session_total_kamas: (135_000
                            + n * 19_000 % 165_000
                            + (n / 6) * 12_000
                            + if n == 20 { 110_000 } else { 0 })
                        .to_string(),
                    };
                    data.zones.push(crate::build_zone_entry(&form).unwrap());
                }
                1 => {
                    let n = data.dungeons.len();
                    let form = DungeonForm {
                        name: [
                            "Donjon des Blops",
                            "Donjon des Bouftous",
                            "Donjon des Rats Noirs",
                        ][n / 2 % 3]
                            .into(),
                        character_class: class,
                        recorded_at_input: date,
                        run_time: duration(28 + n * 7 % 25),
                        gross_kamas_per_run: (90_000 + n * 23_000 % 160_000).to_string(),
                        key_price: (8_000 + n % 4 * 2_000).to_string(),
                    };
                    data.dungeons
                        .push(crate::build_dungeon_entry(&form).unwrap());
                }
                2 => {
                    let n = data.duo_trios.len();
                    let form = DuoTrioForm {
                        name: if n % 3 == 2 {
                            "Blop Multicolore Royal"
                        } else {
                            "Dragon Cochon"
                        }
                        .into(),
                        character_class: class,
                        recorded_at_input: date,
                        party_mode: if n % 3 == 2 {
                            PartyMode::Trio
                        } else {
                            PartyMode::Duo
                        },
                        run_time: duration(38 + n * 5 % 25),
                        loot_kamas_per_run: (95_000 + n * 17_000 % 100_000).to_string(),
                        capture_stone_price: (12_000 + n % 3 * 1_500).to_string(),
                        key_unit_price: (6_000 + n % 4 * 1_000).to_string(),
                        full_soul_sale_price: (110_000 + n % 5 * 18_000).to_string(),
                    };
                    data.duo_trios
                        .push(crate::build_duo_trio_entry(&form).unwrap());
                }
                3 => {
                    let n = data.arenas.len();
                    let form = ArenaForm {
                        name: if n % 2 == 0 {
                            "Captures Bouftou Royal"
                        } else {
                            "Captures Blop Royal"
                        }
                        .into(),
                        character_class: class,
                        recorded_at_input: date,
                        round_time: duration(15 + n * 3 % 13),
                        seat_price: (24_000 + n % 3 * 3_000).to_string(),
                        seats_sold: if n == 1 || n == 6 { "2" } else { "6" }.into(),
                        capture_price: (90_000 + n % 4 * 5_000).to_string(),
                        captures_count: "1".into(),
                    };
                    data.arenas.push(crate::build_arena_entry(&form).unwrap());
                }
                _ => unreachable!(),
            }
            index += 1;
        }
    }
    // Je reproduis le classement enregistré par l'application après chaque saisie.
    data.zones
        .sort_by(|a, b| b.kamas_per_hour.total_cmp(&a.kamas_per_hour));
    data.dungeons
        .sort_by(|a, b| b.kamas_per_hour.total_cmp(&a.kamas_per_hour));
    data.duo_trios
        .sort_by(|a, b| b.kamas_per_hour.total_cmp(&a.kamas_per_hour));
    data.arenas
        .sort_by(|a, b| b.kamas_per_hour.total_cmp(&a.kamas_per_hour));

    let mut zone = crate::zone_form_from_entry(&data.zones[0]);
    let mut dungeon = crate::dungeon_form_from_entry(&data.dungeons[0]);
    let mut duo = crate::duo_trio_form_from_entry(
        data.duo_trios
            .iter()
            .find(|e| e.party_mode == PartyMode::Duo)
            .unwrap(),
    );
    let mut arena = crate::arena_form_from_entry(&data.arenas[0]);
    // Je laisse quatre exemples prêts à saisir ; ces brouillons ne comptent pas dans les 60 sessions.
    for date in [
        &mut zone.recorded_at_input,
        &mut dungeon.recorded_at_input,
        &mut duo.recorded_at_input,
        &mut arena.recorded_at_input,
    ] {
        *date = "2026-09-12 23:45".into();
    }
    PersistedState {
        data,
        drafts: DraftState {
            zone_form: Some(zone),
            dungeon_form: Some(dungeon),
            duo_trio_form: Some(duo),
            arena_form: Some(arena),
            ..DraftState::default()
        },
        ..PersistedState::default()
    }
}

pub fn weekly_saves() -> Vec<storage::NamedSaveEnvelope> {
    [(8, 20), (8, 27), (9, 3), (9, 12)]
        .into_iter()
        .enumerate()
        .map(|(index, (month, day))| {
            let end = NaiveDate::from_ymd_opt(2026, month, day)
                .unwrap()
                .and_hms_opt(23, 59, 0)
                .unwrap();
            let mut state = month_state();
            state
                .data
                .zones
                .retain(|e| e.recorded_at.is_some_and(|date| date <= end));
            state
                .data
                .dungeons
                .retain(|e| e.recorded_at.is_some_and(|date| date <= end));
            state
                .data
                .duo_trios
                .retain(|e| e.recorded_at.is_some_and(|date| date <= end));
            state
                .data
                .arenas
                .retain(|e| e.recorded_at.is_some_and(|date| date <= end));
            if index < 3 {
                state.drafts = DraftState::default();
            }
            // Je convertis l'heure française d'été en UTC pour les métadonnées des sauvegardes.
            let timestamp = (end - Duration::hours(2)).and_utc();
            storage::NamedSaveEnvelope {
                meta: storage::NamedSaveMeta {
                    save_id: format!("demo-semaine-{}", index + 1),
                    display_name: if index == 3 {
                        "Mon mois de farm — bilan final".into()
                    } else {
                        format!("Mon historique au {}", end.format("%d/%m/%Y"))
                    },
                    created_at: timestamp,
                    updated_at: timestamp,
                },
                state,
            }
        })
        .collect()
}

pub fn save_summaries() -> Vec<storage::NamedSaveSummary> {
    weekly_saves()
        .into_iter()
        .rev()
        .map(|save| {
            let data = &save.state.data;
            storage::NamedSaveSummary {
                path: format!("{}.json", save.meta.save_id).into(),
                data_summary: storage::NamedSaveDataSummary {
                    zones: data.zones.len(),
                    dungeons: data.dungeons.len(),
                    duo_trios: data.duo_trios.len(),
                    arenas: data.arenas.len(),
                    draft_count: save.state.drafts.restored_items_count(),
                },
                meta: save.meta,
                used_backup: false,
            }
        })
        .collect()
}

pub fn summary() -> serde_json::Value {
    let state = month_state();
    let report = reports::build_report_summary(
        &state.data,
        reports::ReportPeriod::Last30Days,
        reports::ReportCategoryFilter::default(),
        reference_time(),
    );
    let sessions = reports::collect_sessions(&state.data);
    let groups = reports::build_activity_recommendations(
        &state.data,
        &reports::ActivitySearchFilters {
            class: Some(DofusClass::Cra),
            available_time_seconds: Some(3600.0),
        },
    );
    serde_json::json!({
        "fictional": true,
        "reference_time": reference_time(),
        "sessions": report.session_count,
        "active_days": sessions.iter().map(|s| s.recorded_at.unwrap().date()).collect::<std::collections::BTreeSet<_>>().len(),
        "duration_seconds": sessions.iter().map(|s| s.duration_seconds).sum::<f32>(),
        "total_value": report.total_earned,
        "average_value": report.average_per_session,
        "categories": report.chart_series.iter().map(|s| serde_json::json!({
            "name": s.kind.label(), "sessions": s.session_count, "total_value": s.total_in_period
        })).collect::<Vec<_>>(),
        "classes": report.class_summaries.iter().map(|s| serde_json::json!({
            "name": s.class.label(), "sessions": s.sessions, "total_value": s.total_value
        })).collect::<Vec<_>>(),
        "recommendations_cra_one_hour": groups.iter().map(|g| serde_json::json!({
            "category": g.kind.label(), "items": g.items.iter().map(|r| serde_json::json!({
                "name": r.name, "sessions": r.session_count, "runs": r.estimated_runs,
                "estimated_value": r.estimated_total_value
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

pub fn export_to(output: &Path) -> Result<(), String> {
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
    storage::save_state_to_path(&month_state(), &output.join("mois-demo.json"))?;
    std::fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&summary()).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let saves = output.join("saves");
    std::fs::create_dir_all(&saves).map_err(|e| e.to_string())?;
    for save in weekly_saves() {
        std::fs::write(
            saves.join(format!("{}.json", save.meta.save_id)),
            serde_json::to_vec_pretty(&save).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[test]
#[ignore = "Exporte le mois fictif dans target/qa/demo-export sans ouvrir le profil utilisateur."]
fn export_demo_month() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/qa/demo-export");
    export_to(&output).unwrap();
    println!(
        "{}\n{}",
        output.display(),
        serde_json::to_string_pretty(&summary()).unwrap()
    );
}

#[test]
fn month_covers_sixty_sessions_twenty_days_and_all_activities() {
    let state = month_state();
    assert_eq!(state.schema_version, 2);
    assert_eq!(
        [
            state.data.zones.len(),
            state.data.dungeons.len(),
            state.data.duo_trios.len(),
            state.data.arenas.len()
        ],
        [24, 16, 12, 8]
    );
    assert_eq!(
        state
            .data
            .duo_trios
            .iter()
            .filter(|e| e.party_mode == PartyMode::Duo)
            .count(),
        8
    );
    assert_eq!(
        state
            .data
            .arenas
            .iter()
            .filter(|e| e.net_profit < 0.0)
            .count(),
        2
    );
    let mut sessions = reports::collect_sessions(&state.data);
    sessions.sort_by_key(|s| s.recorded_at);
    let days = sessions
        .iter()
        .map(|s| s.recorded_at.unwrap().date())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(days.len(), 20);
    assert_eq!(days.first().unwrap().to_string(), "2026-08-14");
    assert_eq!(days.last().unwrap().to_string(), "2026-09-12");
    for session in &sessions {
        assert!(session.duration_seconds > 0.0);
        assert!(session.accounting_value.is_finite() && session.kamas_per_hour.is_finite());
        assert!(
            session.recorded_at.unwrap() + Duration::seconds(session.duration_seconds as i64)
                <= reference_time()
        );
        assert!(session.class.is_some());
    }
    for pair in sessions.windows(2) {
        assert!(
            pair[0].recorded_at.unwrap() + Duration::seconds(pair[0].duration_seconds as i64)
                <= pair[1].recorded_at.unwrap()
        );
    }
    assert_eq!(state.drafts.restored_items_count(), 4);
    assert_eq!(
        serde_json::to_value(&state).unwrap(),
        serde_json::to_value(month_state()).unwrap()
    );
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() <= 0.5, "{actual} != {expected}");
}

#[test]
fn month_values_and_periods_agree_with_independent_accounting() {
    let state = month_state();
    for e in &state.data.zones {
        close(
            e.kamas_per_hour,
            e.session_total_kamas * 3600.0 / e.session_time_seconds,
        );
    }
    for e in &state.data.dungeons {
        close(e.net_kamas_per_run, e.gross_kamas_per_run - e.key_price);
        close(
            e.kamas_per_hour,
            e.net_kamas_per_run * 60.0 / e.run_time_minutes,
        );
    }
    for e in &state.data.duo_trios {
        let keys = if e.party_mode == PartyMode::Duo { 2 } else { 3 };
        assert_eq!(e.keys_count, keys);
        close(e.total_key_cost, e.key_unit_price * keys as f32);
        close(e.total_cost, e.capture_stone_price + e.total_key_cost);
        close(
            e.gross_kamas_per_run,
            e.loot_kamas_per_run + e.full_soul_sale_price,
        );
        close(e.net_kamas_per_run, e.gross_kamas_per_run - e.total_cost);
        close(
            e.kamas_per_hour,
            e.net_kamas_per_run * 3600.0 / e.run_time_seconds,
        );
    }
    for e in &state.data.arenas {
        close(
            e.net_profit,
            e.seat_price * e.seats_sold as f32 - e.capture_price * e.captures_count as f32,
        );
        close(e.kamas_per_hour, e.net_profit * 60.0 / e.round_time_minutes);
    }
    let sessions = reports::collect_sessions(&state.data);
    for (period, days) in [
        (reports::ReportPeriod::Last24Hours, 1),
        (reports::ReportPeriod::Last7Days, 7),
        (reports::ReportPeriod::Last30Days, 30),
    ] {
        let visible = sessions
            .iter()
            .filter(|s| s.recorded_at.unwrap() >= reference_time() - Duration::days(days))
            .collect::<Vec<_>>();
        let report = reports::build_report_summary(
            &state.data,
            period,
            reports::ReportCategoryFilter::default(),
            reference_time(),
        );
        assert_eq!(report.session_count, visible.len());
        close(
            report.total_earned,
            visible.iter().map(|s| s.accounting_value).sum(),
        );
        close(
            report.chart_series.iter().map(|s| s.total_in_period).sum(),
            report.total_earned,
        );
        close(
            report.class_summaries.iter().map(|s| s.total_value).sum(),
            report.total_earned,
        );
    }
    let full = reports::build_report_summary(
        &state.data,
        reports::ReportPeriod::AllTime,
        reports::ReportCategoryFilter::default(),
        reference_time() + Duration::days(365),
    );
    assert_eq!(full.session_count, 60);
}

#[test]
fn cra_recommendations_cover_every_category_with_a_realistic_time_budget() {
    let state = month_state();
    let groups = reports::build_activity_recommendations(
        &state.data,
        &reports::ActivitySearchFilters {
            class: Some(DofusClass::Cra),
            available_time_seconds: Some(3600.0),
        },
    );
    assert_eq!(groups.len(), 4);
    for group in groups {
        assert!(!group.items.is_empty());
        for recommendation in group.items {
            assert_eq!(recommendation.class, Some(DofusClass::Cra));
            assert!(recommendation.average_duration_seconds <= 3600.0);
            let runs = recommendation.estimated_runs.unwrap();
            assert!(runs >= 1);
            assert!(runs as f32 * recommendation.average_duration_seconds <= 3600.0);
            close(
                recommendation.estimated_total_value.unwrap(),
                runs as f32 * recommendation.average_value_per_session,
            );
        }
    }
}

struct TempDemo(std::path::PathBuf);
impl TempDemo {
    fn new() -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!("evofarm-demo-{}-{unique}", std::process::id())))
    }
}
impl Drop for TempDemo {
    fn drop(&mut self) {
        // Je nettoie seulement le dossier temporaire unique créé par ce test.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn exported_month_imports_and_roundtrips_with_drafts_and_backups() {
    let temporary = TempDemo::new();
    export_to(&temporary.0).unwrap();
    let source = temporary.0.join("mois-demo.json");
    let imported = storage::load_state_from_path(&source).unwrap();
    assert_eq!(imported.cleaned_legacy_entries, 0);
    assert!(!imported.used_backup);
    assert_eq!(
        serde_json::to_value(&imported.state).unwrap(),
        serde_json::to_value(month_state()).unwrap()
    );
    let destination = temporary.0.join("profil-isole/data.json");
    let previous = PersistedState::default();
    storage::save_state_to_path(&previous, &destination).unwrap();
    let mut app = MyApp::default();
    app.import_from_path_to_save_path(&source, &destination);
    assert!(matches!(
        app.status.as_ref().unwrap().kind,
        crate::StatusKind::Success
    ));
    assert_eq!(
        serde_json::to_value(app.build_persisted_state()).unwrap(),
        serde_json::to_value(month_state()).unwrap()
    );
    let saved = storage::load_state_from_path(&destination).unwrap();
    assert_eq!(
        serde_json::to_value(saved.state).unwrap(),
        serde_json::to_value(month_state()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(
            storage::load_state_from_path(&destination.with_extension("json.bak"))
                .unwrap()
                .state
        )
        .unwrap(),
        serde_json::to_value(previous).unwrap()
    );
    // Je vérifie aussi les quatre brouillons avec les validations utilisées lors d'une saisie.
    crate::build_zone_entry(&app.zone_form).unwrap();
    crate::build_dungeon_entry(&app.dungeon_form).unwrap();
    crate::build_duo_trio_entry(&app.duo_trio_form).unwrap();
    crate::build_arena_entry(&app.arena_form).unwrap();
}

#[test]
fn weekly_library_contains_the_actual_progressive_history() {
    let temporary = TempDemo::new();
    export_to(&temporary.0).unwrap();
    let library = storage::list_named_saves_in_dir(&temporary.0.join("saves")).unwrap();
    let expected = save_summaries();
    assert_eq!(library.len(), 4);
    for (actual, expected) in library.iter().zip(&expected) {
        assert_eq!(actual.meta, expected.meta);
        assert_eq!(actual.data_summary, expected.data_summary);
        let loaded =
            storage::load_named_save_in_dir(&actual.meta.save_id, &temporary.0.join("saves"))
                .unwrap();
        let cutoff = (actual.meta.updated_at + Duration::hours(2)).naive_utc();
        let report = reports::build_report_summary(
            &month_state().data,
            reports::ReportPeriod::AllTime,
            reports::ReportCategoryFilter::default(),
            cutoff,
        );
        assert_eq!(
            reports::collect_sessions(&loaded.state.data).len(),
            report.session_count
        );
        close(
            reports::collect_sessions(&loaded.state.data)
                .iter()
                .map(|s| s.accounting_value)
                .sum(),
            report.total_earned,
        );
    }
    assert_eq!(library[0].data_summary.draft_count, 4);
    assert_eq!(library[3].data_summary.draft_count, 0);
}

#[test]
fn published_json_matches_the_generated_month() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/demo/mois-demo.json");
    let state = storage::load_state_from_path(&path).unwrap();
    assert_eq!(state.cleaned_legacy_entries, 0);
    assert_eq!(
        serde_json::to_value(state.state).unwrap(),
        serde_json::to_value(month_state()).unwrap()
    );
    assert_eq!(
        calculations::format_recorded_at(Some(reference_time())),
        "2026-09-12 23:59"
    );
}
