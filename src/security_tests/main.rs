use super::*;

#[test]
fn failed_commit_restores_data_and_input_without_reporting_success() {
    // Une destination qui est un dossier échoue sur toutes les plateformes,
    // y compris lorsque la suite s'exécute avec des privilèges administrateur.
    let path = std::env::temp_dir();
    let mut app = MyApp {
        current_tab: Tab::Donjons,
        zone_form: ZoneForm {
            name: "Saisie à conserver".to_string(),
            ..ZoneForm::default()
        },
        ..MyApp::default()
    };
    let previous = app.build_persisted_state();
    app.data.zones.push(ZoneEntry {
        name: "Non enregistré".to_string(),
        session_time_seconds: 60.0,
        session_total_kamas: 10.0,
        ..ZoneEntry::default()
    });
    app.zone_form = ZoneForm::default();
    assert!(!app.commit_current_state_or_restore_to_path(previous, "Ajout", &path));
    assert!(app.data.zones.is_empty());
    assert_eq!(app.zone_form.name, "Saisie à conserver");
    assert_eq!(app.current_tab, Tab::Donjons);
    assert!(matches!(
        app.status.as_ref().map(|status| status.kind),
        Some(StatusKind::Error)
    ));
}

#[test]
fn quota_rejection_restores_inline_draft_and_prior_dataset() {
    let entry = ZoneEntry {
        name: "Valide".to_string(),
        session_time_seconds: 60.0,
        session_total_kamas: 10.0,
        ..ZoneEntry::default()
    };
    let mut app = MyApp::default();
    app.data.zones.push(entry.clone());
    app.zone_edit = Some(InlineEditState {
        index: 0,
        form: ZoneForm {
            name: "Edition à conserver".to_string(),
            ..ZoneForm::default()
        },
        error: None,
    });
    let previous = app.build_persisted_state();
    app.data.zones = vec![entry; crate::limits::MAX_TOTAL_ENTRIES + 1];
    app.zone_edit = None;
    assert!(!app.commit_current_state_or_restore_to_path(previous, "Ajout", &std::env::temp_dir()));
    assert_eq!(app.data.zones.len(), 1);
    assert_eq!(
        app.zone_edit.as_ref().unwrap().form.name,
        "Edition à conserver"
    );
}
