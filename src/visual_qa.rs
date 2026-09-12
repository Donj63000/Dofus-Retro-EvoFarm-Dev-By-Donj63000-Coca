#![cfg(target_os = "windows")]

use crate::models::{DofusClass, DurationInput, Tab};
use crate::{reports, theme, MyApp};
use eframe::egui::{self, ColorImage, Event, ViewportCommand};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use winit::platform::windows::EventLoopBuilderExtWindows;

const REVIEW_TITLE: &str = "EvoFarm — contrôle visuel";
const FRAME_LIMIT: usize = 3_600;
const REVIEW_TIMEOUT: Duration = Duration::from_secs(90);
const TEST_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScenarioDialog {
    NamedSave,
    NamedLoad,
}

impl ScenarioDialog {
    fn label(self) -> &'static str {
        match self {
            Self::NamedSave => "Sauvegarder une session nommée",
            Self::NamedLoad => "Bibliothèque des sauvegardes nommées",
        }
    }
}

#[derive(Clone, Copy)]
struct Scenario {
    tab: Tab,
    name: &'static str,
    logical_size: [u32; 2],
    pixels_per_point: f32,
    bar_chart: bool,
    dialog: Option<ScenarioDialog>,
}

impl Scenario {
    fn pixel_size(self) -> [usize; 2] {
        self.logical_size
            .map(|side| (side as f32 * self.pixels_per_point).round() as usize)
    }

    fn file_name(self) -> String {
        let chart = if self.tab == Tab::Bilans {
            if self.bar_chart {
                "-barres"
            } else {
                "-courbe"
            }
        } else {
            ""
        };
        format!(
            "{}{}-{}x{}-dpi{}.png",
            self.name,
            chart,
            self.logical_size[0],
            self.logical_size[1],
            (self.pixels_per_point * 100.0).round() as u32
        )
    }
}

fn scenarios() -> Vec<Scenario> {
    let tabs = [
        (Tab::Bilans, "bilans"),
        (Tab::RechercheActivite, "recherche"),
        (Tab::Zones, "zones"),
        (Tab::Donjons, "donjons"),
        (Tab::DuoTrio, "duo-trio"),
        (Tab::PlArene, "arene"),
    ];
    let mut result = Vec::new();
    for pixels_per_point in [1.0, 1.25, 1.5] {
        for logical_size in [[920, 680], [1180, 820]] {
            for (tab, name) in tabs {
                result.push(Scenario {
                    tab,
                    name,
                    logical_size,
                    pixels_per_point,
                    bar_chart: logical_size[0] == 920,
                    dialog: None,
                });
            }
        }
    }
    // Je capture aussi les parties situées sous le pli sans modifier le défilement utilisateur.
    for (tab, name) in tabs {
        result.push(Scenario {
            tab,
            name,
            logical_size: [1180, 2200],
            pixels_per_point: 1.0,
            bar_chart: true,
            dialog: None,
        });
    }
    result.push(Scenario {
        tab: Tab::Bilans,
        name: "bilans",
        logical_size: [1180, 2200],
        pixels_per_point: 1.0,
        bar_chart: false,
        dialog: None,
    });
    for (dialog, name) in [
        (ScenarioDialog::NamedSave, "sauvegarder-session"),
        (ScenarioDialog::NamedLoad, "bibliotheque-sauvegardes"),
    ] {
        result.push(Scenario {
            tab: Tab::Zones,
            name,
            logical_size: [920, 680],
            pixels_per_point: 1.25,
            bar_chart: false,
            dialog: Some(dialog),
        });
    }
    result
}

fn configure_scenario(app: &mut MyApp, scenario: Scenario) {
    app.current_tab = scenario.tab;
    app.report_bar_mode = scenario.bar_chart;
    app.show_named_save_dialog = scenario.dialog == Some(ScenarioDialog::NamedSave);
    app.show_named_load_dialog = scenario.dialog == Some(ScenarioDialog::NamedLoad);
    app.show_named_save_confirm = false;
    app.pending_named_save_confirm_action = None;
    app.named_save_rename = None;
    app.named_save_error = None;
    app.named_saves_error = None;
    app.named_saves.clear();

    // Je prépare la bibliothèque directement en mémoire pour ne pas lire le profil utilisateur.
    match scenario.dialog {
        Some(ScenarioDialog::NamedSave) => {
            app.named_save_name_input =
                "Expédition dans le Monde des Douze — sessions de la semaine".into();
        }
        Some(ScenarioDialog::NamedLoad) => {
            let timestamp = chrono::DateTime::parse_from_rfc3339("2026-09-12T12:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc);
            let names = [
                "Donjons et captures de la semaine — aventures en équipe dans le Monde des Douze, du Bouftou Royal au Dragon Cochon",
                "Plaines de Cania — progression de mon Crâ",
            ];
            app.named_saves = names
                .into_iter()
                .enumerate()
                .map(|(index, display_name)| crate::storage::NamedSaveSummary {
                    meta: crate::storage::NamedSaveMeta {
                        save_id: format!("controle-visuel-{index}"),
                        display_name: display_name.into(),
                        created_at: timestamp,
                        updated_at: timestamp - chrono::Duration::hours(index as i64),
                    },
                    path: PathBuf::from(format!("controle-visuel-{index}.json")),
                    used_backup: index == 1,
                    data_summary: crate::storage::NamedSaveDataSummary {
                        zones: 10,
                        dungeons: 12,
                        duo_trios: 8,
                        arenas: 4,
                        draft_count: 3,
                    },
                })
                .collect();
            app.named_save_rename = Some(crate::NamedSaveRenameState {
                save_id: "controle-visuel-0".into(),
                value: "Sessions et captures de septembre — mon équipe".into(),
                error: None,
            });
        }
        None => {}
    }
}

#[derive(Default)]
struct ReviewProgress {
    captures: Vec<serde_json::Value>,
    error: Option<String>,
}

enum CaptureStage {
    Configure,
    Settle { stable_frames: usize, frames: usize },
    AwaitImage { requested: Instant },
    Finished,
}

struct VisualReview {
    app: MyApp,
    scenarios: Vec<Scenario>,
    current: usize,
    stage: CaptureStage,
    output: PathBuf,
    progress: Arc<Mutex<ReviewProgress>>,
    started: Instant,
    frames: usize,
    rejected_images: usize,
}

impl VisualReview {
    fn fail(&mut self, ctx: &egui::Context, message: String) {
        self.progress.lock().unwrap().error.get_or_insert(message);
        self.stage = CaptureStage::Finished;
        ctx.send_viewport_cmd(ViewportCommand::Close);
        ctx.request_repaint();
    }

    fn save_capture(&mut self, image: &ColorImage) -> Result<(), String> {
        let scenario = self.scenarios[self.current];
        let file_name = scenario.file_name();
        let path = self.output.join(&file_name);
        let bytes: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(|pixel| pixel.to_array())
            .collect();
        image::save_buffer_with_format(
            &path,
            &bytes,
            image.size[0] as u32,
            image.size[1] as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .map_err(|error| format!("Impossible d'écrire {} : {error}", path.display()))?;
        self.progress
            .lock()
            .unwrap()
            .captures
            .push(serde_json::json!({
                "file": file_name,
                "tab": scenario.tab.label(),
                "logical_size": scenario.logical_size,
                "pixels_per_point": scenario.pixels_per_point,
                "pixel_size": image.size,
                "bar_chart": scenario.tab == Tab::Bilans && scenario.bar_chart,
                "dialog": scenario.dialog.map(ScenarioDialog::label),
            }));
        println!(
            "Capture {}/{} : {}",
            self.current + 1,
            self.scenarios.len(),
            path.display()
        );
        Ok(())
    }

    fn receive_capture(&mut self, ctx: &egui::Context) -> bool {
        if !matches!(self.stage, CaptureStage::AwaitImage { .. }) {
            return false;
        }
        let screenshot = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                Event::Screenshot { viewport_id, image }
                    if *viewport_id == egui::ViewportId::ROOT =>
                {
                    Some(Arc::clone(image))
                }
                _ => None,
            })
        });
        let Some(image) = screenshot else {
            return false;
        };

        let scenario = self.scenarios[self.current];
        if image.size != scenario.pixel_size() {
            self.rejected_images += 1;
            if self.rejected_images > 2 {
                self.fail(
                    ctx,
                    format!(
                        "Dimensions de capture incorrectes pour {} : {:?}, attendu {:?}.",
                        scenario.file_name(),
                        image.size,
                        scenario.pixel_size()
                    ),
                );
            } else {
                self.stage = CaptureStage::Configure;
            }
            return true;
        }

        if let Err(error) = self.save_capture(&image) {
            self.fail(ctx, error);
            return true;
        }
        self.current += 1;
        self.rejected_images = 0;
        self.stage = if self.current == self.scenarios.len() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            CaptureStage::Finished
        } else {
            CaptureStage::Configure
        };
        true
    }
}

// Je conserve les informations de taille natives, mais aucun clic, raccourci ou fichier déposé.
fn isolate_input(input: &mut egui::RawInput) {
    input
        .events
        .retain(|event| matches!(event, Event::Screenshot { .. }));
    input.modifiers = egui::Modifiers::default();
    input.hovered_files.clear();
    input.dropped_files.clear();
    input.focused = false;
    for viewport in input.viewports.values_mut() {
        viewport.events.clear();
        viewport.focused = Some(false);
    }
}

impl eframe::App for VisualReview {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        isolate_input(input);
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames += 1;
        if matches!(self.stage, CaptureStage::Finished) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        if self.frames > FRAME_LIMIT || self.started.elapsed() > REVIEW_TIMEOUT {
            self.fail(
                ctx,
                "Le contrôle visuel a dépassé son délai maximal.".to_string(),
            );
            return;
        }

        if self.receive_capture(ctx) {
            ctx.request_repaint();
            return;
        }

        let scenario = self.scenarios[self.current];
        let requested_size = egui::vec2(
            scenario.logical_size[0] as f32,
            scenario.logical_size[1] as f32,
        );
        match &mut self.stage {
            CaptureStage::Configure => {
                configure_scenario(&mut self.app, scenario);
                ctx.set_pixels_per_point(scenario.pixels_per_point);
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(requested_size));
                self.stage = CaptureStage::Settle {
                    stable_frames: 0,
                    frames: 0,
                };
            }
            CaptureStage::Settle {
                stable_frames,
                frames,
            } => {
                *frames += 1;
                let actual_size = ctx.screen_rect().size();
                let scale_matches =
                    (ctx.pixels_per_point() - scenario.pixels_per_point).abs() < 0.001;
                let size_matches = (actual_size.x - requested_size.x).abs() < 0.5
                    && (actual_size.y - requested_size.y).abs() < 0.5;
                if scale_matches && size_matches {
                    *stable_frames += 1;
                } else {
                    *stable_frames = 0;
                    // Je redemande la taille après la prise en compte du nouveau facteur DPI.
                    ctx.set_pixels_per_point(scenario.pixels_per_point);
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(requested_size));
                }
                if *frames > 180 {
                    self.fail(
                        ctx,
                        format!(
                            "La fenêtre ne peut pas atteindre {} : taille logique {:?}, DPI {}.",
                            scenario.file_name(),
                            actual_size,
                            ctx.pixels_per_point()
                        ),
                    );
                    return;
                }
                if *stable_frames >= 4 {
                    ctx.send_viewport_cmd(ViewportCommand::Screenshot);
                    self.stage = CaptureStage::AwaitImage {
                        requested: Instant::now(),
                    };
                }
            }
            CaptureStage::AwaitImage { requested } => {
                if requested.elapsed() > Duration::from_secs(5) {
                    self.fail(
                        ctx,
                        format!("Capture non reçue : {}.", scenario.file_name()),
                    );
                    return;
                }
            }
            CaptureStage::Finished => return,
        }

        // Je rends le véritable écran avec des données fictives en mémoire, sans charger le profil.
        self.app.render(ctx);
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn fixture_app(ctx: &egui::Context) -> MyApp {
    // Je prépare seulement des formulaires fictifs ; les builders métier calculent leurs résultats.
    let mut app = MyApp {
        zone_form: serde_json::from_str(
            r#"{"name":"Plaines de Cania","character_class":"cra",
            "session_time":{"hours":"01","minutes":"15","seconds":"00"},
            "session_total_kamas":"325000"}"#,
        )
        .unwrap(),
        dungeon_form: serde_json::from_str(
            r#"{"name":"Donjon des Blops","character_class":"cra",
            "run_time":{"hours":"00","minutes":"35","seconds":"00"},
            "gross_kamas_per_run":"185000","key_price":"12000"}"#,
        )
        .unwrap(),
        duo_trio_form: serde_json::from_str(
            r#"{"name":"Dragon Cochon en duo","character_class":"cra",
            "run_time":{"hours":"00","minutes":"40","seconds":"00"},"party_mode":"duo",
            "loot_kamas_per_run":"125000","capture_stone_price":"15000",
            "key_unit_price":"7500","full_soul_sale_price":"145000"}"#,
        )
        .unwrap(),
        arena_form: serde_json::from_str(
            r#"{"name":"Captures Bouftou Royal","character_class":"cra",
            "round_time":{"hours":"00","minutes":"15","seconds":"00"},"seat_price":"25000",
            "seats_sold":"6","capture_price":"95000","captures_count":"1"}"#,
        )
        .unwrap(),
        activity_search_class: Some(DofusClass::Cra),
        activity_search_time: DurationInput {
            hours: "01".to_string(),
            ..DurationInput::default()
        },
        report_period: reports::ReportPeriod::Last7Days,
        ..MyApp::default()
    };
    let now = reports::local_now();
    for (index, class) in [DofusClass::Cra, DofusClass::Sadida, DofusClass::Feca]
        .into_iter()
        .enumerate()
    {
        let recorded_at = Some(now - chrono::Duration::hours(2 + index as i64 * 12));
        let mut zone = crate::build_zone_entry(&app.zone_form).unwrap();
        zone.character_class = Some(class);
        zone.recorded_at = recorded_at;
        app.data.zones.push(zone);
        let mut dungeon = crate::build_dungeon_entry(&app.dungeon_form).unwrap();
        dungeon.character_class = Some(class);
        dungeon.recorded_at = recorded_at;
        app.data.dungeons.push(dungeon);
        let mut duo_trio = crate::build_duo_trio_entry(&app.duo_trio_form).unwrap();
        duo_trio.character_class = Some(class);
        duo_trio.recorded_at = recorded_at;
        app.data.duo_trios.push(duo_trio);
        let mut arena_form = app.arena_form.clone();
        if index == 2 {
            arena_form.seats_sold = "2".to_string();
        }
        let mut arena = crate::build_arena_entry(&arena_form).unwrap();
        arena.character_class = Some(class);
        arena.recorded_at = recorded_at;
        app.data.arenas.push(arena);
    }
    app.load_visual_assets(ctx);
    assert!(
        app.background_texture.is_some(),
        "Le fond doit être chargé pour le contrôle visuel."
    );
    assert!(
        app.app_icon_texture.is_some(),
        "Le logo doit être chargé pour le contrôle visuel."
    );
    assert_eq!(app.class_textures.len(), DofusClass::all().len());
    app
}

fn run_review(control: Arc<Mutex<Option<egui::Context>>>, output: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(output).map_err(|error| error.to_string())?;
    let progress = Arc::new(Mutex::new(ReviewProgress::default()));
    let progress_for_app = Arc::clone(&progress);
    let app_output = output.to_path_buf();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(REVIEW_TITLE)
            .with_app_id("evofarm-visual-review")
            .with_inner_size([920.0, 680.0])
            .with_position([12.0, 12.0])
            .with_active(false)
            .with_drag_and_drop(false)
            .with_taskbar(false)
            .with_window_level(egui::WindowLevel::AlwaysOnBottom),
        event_loop_builder: Some(Box::new(|builder| {
            builder.with_any_thread(true);
        })),
        renderer: eframe::Renderer::Glow,
        persist_window: false,
        follow_system_theme: false,
        run_and_return: true,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        REVIEW_TITLE,
        options,
        Box::new(move |cc| {
            theme::apply_theme(&cc.egui_ctx);
            *control.lock().unwrap() = Some(cc.egui_ctx.clone());
            Ok(Box::new(VisualReview {
                app: fixture_app(&cc.egui_ctx),
                scenarios: scenarios(),
                current: 0,
                stage: CaptureStage::Configure,
                output: app_output,
                progress: progress_for_app,
                started: Instant::now(),
                frames: 0,
                rejected_images: 0,
            }))
        }),
    )
    .map_err(|error| error.to_string())?;

    let progress = progress.lock().unwrap();
    let manifest = serde_json::json!({
        "application": REVIEW_TITLE,
        "expected_captures": scenarios().len(),
        "captures": progress.captures,
        "error": progress.error,
    });
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if let Some(error) = &progress.error {
        return Err(format!(
            "{error} Manifest : {}",
            output.join("manifest.json").display()
        ));
    }
    if progress.captures.len() != scenarios().len() {
        return Err(format!(
            "Fenêtre fermée avant la fin : {}/{} captures.",
            progress.captures.len(),
            scenarios().len()
        ));
    }
    Ok(progress.captures.len())
}

#[test]
#[ignore = "Ouvre une fenêtre isolée et capture les écrans et dialogues dans target/qa/visual."]
fn native_visual_review() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/qa/visual")
        .join(format!("run-{unique}"));
    let control = Arc::new(Mutex::new(None::<egui::Context>));
    let worker_control = Arc::clone(&control);
    let worker_output = output.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    // Je borne aussi l'attente du test si le pilote graphique cesse de produire des frames.
    let worker = std::thread::Builder::new()
        .name("evofarm-visual-review".to_string())
        .spawn(move || {
            let result = run_review(worker_control, &worker_output);
            let _ = sender.send(result);
        })
        .unwrap();

    let result = receiver.recv_timeout(TEST_TIMEOUT).unwrap_or_else(|error| {
        if let Some(ctx) = control.lock().unwrap().as_ref() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            ctx.request_repaint();
        }
        panic!(
            "Le contrôle visuel n'a pas terminé : {error}. Dossier : {}",
            output.display()
        );
    });
    worker
        .join()
        .expect("Le thread de contrôle visuel doit se terminer proprement.");
    assert_eq!(
        result.unwrap_or_else(|error| panic!("{error}")),
        scenarios().len()
    );
    println!("Contrôle visuel terminé : {}", output.display());
}

#[test]
fn named_dialog_scenarios_are_distinct_and_preserve_data_and_drafts() {
    let scenarios = scenarios();
    let file_names = scenarios
        .iter()
        .map(|scenario| scenario.file_name())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(file_names.len(), scenarios.len());
    assert_eq!(
        scenarios
            .iter()
            .filter(|scenario| scenario.dialog.is_some())
            .count(),
        2
    );
    let mut app = MyApp::default();
    app.zone_form.name = "Brouillon à conserver".into();
    let initial = serde_json::to_value(app.build_persisted_state()).unwrap();
    for scenario in scenarios
        .iter()
        .filter(|scenario| scenario.dialog.is_some())
    {
        configure_scenario(&mut app, *scenario);
        assert_eq!(
            serde_json::to_value(app.build_persisted_state()).unwrap(),
            initial
        );
        match scenario.dialog.unwrap() {
            ScenarioDialog::NamedSave => {
                assert!(app.show_named_save_dialog);
                assert!(!app.show_named_load_dialog);
                assert!(!app.named_save_name_input.is_empty());
            }
            ScenarioDialog::NamedLoad => {
                assert!(app.show_named_load_dialog);
                assert!(!app.show_named_save_dialog);
                assert_eq!(app.named_saves.len(), 2);
                assert!(app.named_save_rename.is_some());
                assert!(app.named_saves.iter().any(|save| save.used_backup));
            }
        }
    }
    configure_scenario(&mut app, scenarios[0]);
    assert!(!app.show_named_save_dialog);
    assert!(!app.show_named_load_dialog);
    assert!(app.named_saves.is_empty());
    assert!(app.named_save_rename.is_none());
    assert_eq!(
        serde_json::to_value(app.build_persisted_state()).unwrap(),
        initial
    );
}

#[test]
fn visual_review_discards_user_inputs_and_preserves_capture_events() {
    let image = Arc::new(ColorImage::new([2, 2], egui::Color32::WHITE));
    let mut input = egui::RawInput {
        events: vec![
            Event::Text("contenu utilisateur".to_string()),
            Event::PointerButton {
                pos: egui::pos2(20.0, 20.0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
            Event::Screenshot {
                viewport_id: egui::ViewportId::ROOT,
                image: Arc::clone(&image),
            },
        ],
        modifiers: egui::Modifiers::CTRL,
        hovered_files: vec![egui::HoveredFile::default()],
        dropped_files: vec![egui::DroppedFile::default()],
        ..egui::RawInput::default()
    };
    isolate_input(&mut input);
    assert_eq!(
        input.events,
        vec![Event::Screenshot {
            viewport_id: egui::ViewportId::ROOT,
            image
        }]
    );
    assert_eq!(input.modifiers, egui::Modifiers::default());
    assert!(input.hovered_files.is_empty());
    assert!(input.dropped_files.is_empty());
    assert!(!input.focused);
}
