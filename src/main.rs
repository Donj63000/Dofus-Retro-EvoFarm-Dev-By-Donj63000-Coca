#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod calculations;
mod icon_asset;
mod models;
mod reports;
mod storage;
mod theme;
mod ui;

#[cfg(test)]
mod demo;

#[cfg(all(test, target_os = "windows"))]
mod visual_qa;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use models::{
    AppData, ArenaEntry, ArenaForm, DofusClass, DraftState, DungeonEntry, DungeonForm,
    DuoTrioEntry, DuoTrioForm, DurationInput, PersistedInlineEdit, PersistedState, Tab, ZoneEntry,
    ZoneForm,
};
use reports::{ReportCategoryFilter, ReportPeriod};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "EvoFarm";
const APP_ID: &str = "fr.donj63000.evofarm";
const APP_ICON_BYTES: &[u8] = include_bytes!("../logo.png");
const APP_BACKGROUND_BYTES: &[u8] = include_bytes!("../fond.png");
const APP_ICON_SIZE: u32 = 256;
const CLASS_CRA_BYTES: &[u8] = include_bytes!("../classes/cra.png");
const CLASS_ECAFLIP_BYTES: &[u8] = include_bytes!("../classes/ecaflip.png");
const CLASS_ENIRIPSA_BYTES: &[u8] = include_bytes!("../classes/eniripsa.png");
const CLASS_ENUTROF_BYTES: &[u8] = include_bytes!("../classes/enutrof.png");
const CLASS_FECA_BYTES: &[u8] = include_bytes!("../classes/feca.png");
const CLASS_IOP_BYTES: &[u8] = include_bytes!("../classes/iop.png");
const CLASS_OSAMODAS_BYTES: &[u8] = include_bytes!("../classes/osamodas.png");
const CLASS_PANDAWA_BYTES: &[u8] = include_bytes!("../classes/pandawa.png");
const CLASS_SACRIEUR_BYTES: &[u8] = include_bytes!("../classes/sacrieur.png");
const CLASS_SADIDA_BYTES: &[u8] = include_bytes!("../classes/sadida.png");
const CLASS_SRAM_BYTES: &[u8] = include_bytes!("../classes/sram.png");
const CLASS_XELOR_BYTES: &[u8] = include_bytes!("../classes/xelor.png");

fn app_viewport() -> egui::ViewportBuilder {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 820.0])
        .with_min_inner_size([920.0, 680.0])
        .with_title(APP_NAME)
        // Je relie la fenêtre au lanceur Linux et au bundle macOS avec le même identifiant.
        .with_app_id(APP_ID);

    if let Ok(icon) = load_app_icon_data() {
        viewport = viewport.with_icon(icon);
    }

    viewport
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: app_viewport(),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            theme::apply_theme(&cc.egui_ctx);
            let mut app = MyApp::load();
            app.load_visual_assets(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

#[derive(Clone, Copy)]
pub enum StatusKind {
    Success,
    Error,
    Info,
}

pub struct StatusBanner {
    pub kind: StatusKind,
    pub message: String,
}

pub struct InlineEditState<T> {
    pub index: usize,
    pub form: T,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadAction {
    ReloadLocal,
    ImportExternal(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedSaveConfirmAction {
    Load {
        save_id: String,
        display_name: String,
    },
    Overwrite {
        save_id: String,
        display_name: String,
    },
    Delete {
        save_id: String,
        display_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedSaveRenameState {
    pub save_id: String,
    pub value: String,
    pub error: Option<String>,
}

pub struct MyApp {
    // Je fige uniquement les contrôles de démonstration, sans changer l'horloge du logiciel livré.
    #[cfg(test)]
    pub review_now: Option<chrono::NaiveDateTime>,
    #[cfg(test)]
    pub review_scroll_offset: Option<f32>,
    pub data: AppData,
    pub current_tab: Tab,
    pub zone_form: ZoneForm,
    pub dungeon_form: DungeonForm,
    pub duo_trio_form: DuoTrioForm,
    pub arena_form: ArenaForm,
    pub status: Option<StatusBanner>,
    pub background_texture: Option<TextureHandle>,
    pub app_icon_texture: Option<TextureHandle>,
    pub class_textures: HashMap<DofusClass, TextureHandle>,
    pub zone_form_error: Option<String>,
    pub dungeon_form_error: Option<String>,
    pub duo_trio_form_error: Option<String>,
    pub arena_form_error: Option<String>,
    pub report_period: ReportPeriod,
    pub report_categories: ReportCategoryFilter,
    pub report_bar_mode: bool,
    pub activity_search_class: Option<DofusClass>,
    pub activity_search_time: DurationInput,
    pub activity_search_time_touched: bool,
    pub activity_search_time_error: Option<String>,
    pub zone_search: String,
    pub dungeon_search: String,
    pub duo_trio_search: String,
    pub arena_search: String,
    pub zone_edit: Option<InlineEditState<ZoneForm>>,
    pub dungeon_edit: Option<InlineEditState<DungeonForm>>,
    pub duo_trio_edit: Option<InlineEditState<DuoTrioForm>>,
    pub arena_edit: Option<InlineEditState<ArenaForm>>,
    pub zone_delete_confirm: Option<usize>,
    pub dungeon_delete_confirm: Option<usize>,
    pub duo_trio_delete_confirm: Option<usize>,
    pub arena_delete_confirm: Option<usize>,
    pub show_clear_state_confirm: bool,
    pub show_delete_local_save_confirm: bool,
    pub show_load_confirm: bool,
    pub pending_load_action: Option<LoadAction>,
    pub show_named_save_dialog: bool,
    pub show_named_load_dialog: bool,
    pub show_named_save_confirm: bool,
    pub pending_named_save_confirm_action: Option<NamedSaveConfirmAction>,
    pub named_save_name_input: String,
    pub named_save_error: Option<String>,
    pub named_saves: Vec<storage::NamedSaveSummary>,
    pub named_saves_error: Option<String>,
    pub named_save_rename: Option<NamedSaveRenameState>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            #[cfg(test)]
            review_now: None,
            #[cfg(test)]
            review_scroll_offset: None,
            data: AppData::default(),
            current_tab: Tab::Zones,
            zone_form: ZoneForm::default(),
            dungeon_form: DungeonForm::default(),
            duo_trio_form: DuoTrioForm::default(),
            arena_form: ArenaForm::default(),
            status: None,
            background_texture: None,
            app_icon_texture: None,
            class_textures: HashMap::new(),
            zone_form_error: None,
            dungeon_form_error: None,
            duo_trio_form_error: None,
            arena_form_error: None,
            report_period: ReportPeriod::AllTime,
            report_categories: ReportCategoryFilter::default(),
            report_bar_mode: false,
            activity_search_class: None,
            activity_search_time: DurationInput::default(),
            activity_search_time_touched: false,
            activity_search_time_error: None,
            zone_search: String::new(),
            dungeon_search: String::new(),
            duo_trio_search: String::new(),
            arena_search: String::new(),
            zone_edit: None,
            dungeon_edit: None,
            duo_trio_edit: None,
            arena_edit: None,
            zone_delete_confirm: None,
            dungeon_delete_confirm: None,
            duo_trio_delete_confirm: None,
            arena_delete_confirm: None,
            show_clear_state_confirm: false,
            show_delete_local_save_confirm: false,
            show_load_confirm: false,
            pending_load_action: None,
            show_named_save_dialog: false,
            show_named_load_dialog: false,
            show_named_save_confirm: false,
            pending_named_save_confirm_action: None,
            named_save_name_input: String::new(),
            named_save_error: None,
            named_saves: Vec::new(),
            named_saves_error: None,
            named_save_rename: None,
        }
    }
}

impl MyApp {
    pub fn load() -> Self {
        let mut app = Self::default();

        if !storage::has_local_state() {
            return app;
        }

        match storage::load_state() {
            Ok(result) => {
                let should_notify = result.used_backup
                    || result.cleaned_legacy_entries > 0
                    || result.state.drafts.has_any_draft();

                let mut message = format!(
                    "État local restauré. {}",
                    persisted_state_summary(&result.state)
                );

                if result.used_backup {
                    message.push_str(&format!(
                        " Le fichier principal était indisponible ou invalide ; la sauvegarde de secours a été utilisée : {}",
                        result.source_path.display()
                    ));
                }

                if result.cleaned_legacy_entries > 0 {
                    message.push_str(&format!(
                        " {} entrée(s) legacy ont été ignorées au chargement.",
                        result.cleaned_legacy_entries
                    ));
                }

                app.apply_persisted_state(result.state);

                if should_notify {
                    app.set_status(StatusKind::Info, message);
                }
            }
            Err(error) => {
                app.set_status(
                    StatusKind::Error,
                    format!("Impossible de charger l'état local enregistré : {error}"),
                );
            }
        }

        app
    }

    pub fn load_visual_assets(&mut self, ctx: &egui::Context) {
        if self.background_texture.is_none() {
            if let Ok(texture) = load_background_texture(ctx) {
                self.background_texture = Some(texture);
            }
        }

        if self.app_icon_texture.is_none() {
            if let Ok(texture) = load_app_icon_texture(ctx) {
                self.app_icon_texture = Some(texture);
            }
        }

        if self.class_textures.is_empty() {
            self.class_textures = load_class_textures(ctx);
        }
    }

    pub fn save(&mut self) {
        self.persist_with_status("État local sauvegardé.");
    }

    pub fn open_clear_state_dialog(&mut self) {
        self.show_clear_state_confirm = true;
    }

    pub fn close_clear_state_dialog(&mut self) {
        self.show_clear_state_confirm = false;
    }

    pub fn open_delete_local_save_dialog(&mut self) {
        self.show_delete_local_save_confirm = true;
    }

    pub fn close_delete_local_save_dialog(&mut self) {
        self.show_delete_local_save_confirm = false;
    }

    pub fn request_reload_local(&mut self) {
        self.pending_load_action = Some(LoadAction::ReloadLocal);
        self.show_load_confirm = true;
    }

    pub fn request_import_external(&mut self, path: PathBuf) {
        self.pending_load_action = Some(LoadAction::ImportExternal(path));
        self.show_load_confirm = true;
    }

    pub fn cancel_load_confirmation(&mut self) {
        self.show_load_confirm = false;
        self.pending_load_action = None;
    }

    pub fn pending_load_confirmation(&self) -> Option<(String, String, String)> {
        match self.pending_load_action.as_ref()? {
            LoadAction::ReloadLocal => Some((
                "Recharger la sauvegarde locale".to_string(),
                "Cette action remplace l'état actuellement affiché (données et brouillons) par la dernière sauvegarde locale disponible."
                    .to_string(),
                "Charger".to_string(),
            )),
            LoadAction::ImportExternal(path) => Some((
                "Importer un fichier JSON".to_string(),
                format!(
                    "Cette action remplace l'état actuellement affiché (données et brouillons) avec le contenu de {} puis met à jour la sauvegarde locale de l'application.",
                    path.display()
                ),
                "Importer".to_string(),
            )),
        }
    }

    pub fn confirm_pending_load(&mut self) {
        self.show_load_confirm = false;

        if let Some(action) = self.pending_load_action.take() {
            self.load_from_action(action);
        }
    }

    pub fn open_named_save_dialog(&mut self) {
        self.show_named_save_dialog = true;
        self.show_named_load_dialog = false;
        self.show_named_save_confirm = false;
        self.pending_named_save_confirm_action = None;
        self.named_save_name_input.clear();
        self.named_save_error = None;
        self.named_save_rename = None;
    }

    pub fn close_named_save_dialog(&mut self) {
        self.show_named_save_dialog = false;
        self.show_named_save_confirm = false;
        self.pending_named_save_confirm_action = None;
        self.named_save_error = None;
        self.named_save_name_input.clear();
    }

    pub fn open_named_load_dialog(&mut self) {
        let save_directory = storage::named_saves_dir_path();
        let _ = self.refresh_named_saves_from_dir(&save_directory);
        self.show_named_load_dialog = true;
        self.show_named_save_dialog = false;
        self.show_named_save_confirm = false;
        self.pending_named_save_confirm_action = None;
        self.named_save_error = None;
        self.named_save_rename = None;
    }

    pub fn close_named_load_dialog(&mut self) {
        self.show_named_load_dialog = false;
        self.named_saves_error = None;
        self.named_save_rename = None;
    }

    pub fn refresh_named_saves(&mut self) -> Result<(), String> {
        let save_directory = storage::named_saves_dir_path();
        self.refresh_named_saves_from_dir(&save_directory)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn refresh_named_saves_from_dir(&mut self, save_directory: &Path) -> Result<(), String> {
        match storage::list_named_saves_in_dir(save_directory) {
            Ok(named_saves) => {
                self.named_saves = named_saves;
                self.named_saves_error = None;
                Ok(())
            }
            Err(error) => {
                self.named_saves.clear();
                self.named_saves_error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn request_named_save(&mut self) {
        let save_directory = storage::named_saves_dir_path();
        self.request_named_save_in_dir(&save_directory);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn request_named_save_in_dir(&mut self, save_directory: &Path) {
        let display_name = match normalized_named_save_input(&self.named_save_name_input) {
            Some(display_name) => display_name,
            None => {
                self.named_save_error =
                    Some("Le nom de la sauvegarde est obligatoire.".to_string());
                return;
            }
        };

        self.named_save_error = None;

        match storage::find_named_save_by_name_in_dir(&display_name, save_directory) {
            Ok(Some(existing)) => {
                self.pending_named_save_confirm_action = Some(NamedSaveConfirmAction::Overwrite {
                    save_id: existing.meta.save_id,
                    display_name: existing.meta.display_name,
                });
                self.show_named_save_confirm = true;
            }
            Ok(None) => self.save_named_state_to_dir(save_directory, None),
            Err(error) => {
                self.named_save_error = Some(error);
            }
        }
    }

    pub fn start_named_save_load(&mut self, save_id: &str) {
        if let Some(save) = self
            .named_saves
            .iter()
            .find(|save| save.meta.save_id == save_id)
            .cloned()
        {
            self.pending_named_save_confirm_action = Some(NamedSaveConfirmAction::Load {
                save_id: save.meta.save_id,
                display_name: save.meta.display_name,
            });
            self.show_named_save_confirm = true;
        }
    }

    pub fn start_named_save_delete(&mut self, save_id: &str) {
        if let Some(save) = self
            .named_saves
            .iter()
            .find(|save| save.meta.save_id == save_id)
            .cloned()
        {
            self.pending_named_save_confirm_action = Some(NamedSaveConfirmAction::Delete {
                save_id: save.meta.save_id,
                display_name: save.meta.display_name,
            });
            self.show_named_save_confirm = true;
        }
    }

    pub fn start_named_save_rename(&mut self, save_id: &str) {
        if let Some(save) = self
            .named_saves
            .iter()
            .find(|save| save.meta.save_id == save_id)
            .cloned()
        {
            self.named_save_rename = Some(NamedSaveRenameState {
                save_id: save.meta.save_id,
                value: save.meta.display_name,
                error: None,
            });
        }
    }

    pub fn cancel_named_save_rename(&mut self) {
        self.named_save_rename = None;
    }

    pub fn save_named_save_rename(&mut self) {
        let save_directory = storage::named_saves_dir_path();

        let Some(rename) = self.named_save_rename.as_ref() else {
            return;
        };

        let save_id = rename.save_id.clone();
        let new_name = rename.value.clone();
        self.save_named_save_rename_in_dir(&save_directory, &save_id, &new_name);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn save_named_save_rename_in_dir(
        &mut self,
        save_directory: &Path,
        save_id: &str,
        new_name: &str,
    ) {
        match storage::rename_named_save_in_dir(save_id, new_name, save_directory) {
            Ok(meta) => {
                self.named_save_rename = None;
                let _ = self.refresh_named_saves_from_dir(save_directory);
                self.set_status(
                    StatusKind::Success,
                    format!("Sauvegarde renommee : {}.", meta.display_name),
                );
            }
            Err(error) => {
                if let Some(rename) = self.named_save_rename.as_mut() {
                    rename.error = Some(error);
                }
            }
        }
    }

    pub fn cancel_named_save_confirmation(&mut self) {
        self.show_named_save_confirm = false;
        self.pending_named_save_confirm_action = None;
    }

    pub fn pending_named_save_confirmation(&self) -> Option<(String, String, String)> {
        match self.pending_named_save_confirm_action.as_ref()? {
            NamedSaveConfirmAction::Load { display_name, .. } => Some((
                "Charger une sauvegarde".to_string(),
                format!(
                    "Cette action remplace l'etat actuellement affiche par la sauvegarde nommee {}.",
                    display_name
                ),
                "Charger".to_string(),
            )),
            NamedSaveConfirmAction::Overwrite { display_name, .. } => Some((
                "Remplacer une sauvegarde".to_string(),
                format!(
                    "Une sauvegarde nommee {} existe deja. Cette action la remplace avec l'etat actuel.",
                    display_name
                ),
                "Remplacer".to_string(),
            )),
            NamedSaveConfirmAction::Delete { display_name, .. } => Some((
                "Supprimer une sauvegarde".to_string(),
                format!(
                    "Cette action supprime definitivement la sauvegarde nommee {} et son fichier .bak lorsqu'il existe.",
                    display_name
                ),
                "Supprimer".to_string(),
            )),
        }
    }

    pub fn confirm_pending_named_save_action(&mut self) {
        self.show_named_save_confirm = false;

        if let Some(action) = self.pending_named_save_confirm_action.take() {
            let save_directory = storage::named_saves_dir_path();

            match action {
                NamedSaveConfirmAction::Load { save_id, .. } => {
                    self.load_named_save_from_dir(&save_directory, &save_id);
                }
                NamedSaveConfirmAction::Overwrite { save_id, .. } => {
                    self.save_named_state_to_dir(&save_directory, Some(&save_id));
                }
                NamedSaveConfirmAction::Delete { save_id, .. } => {
                    self.delete_named_save_in_dir(&save_directory, &save_id);
                }
            }
        }
    }

    pub fn submit_zone_form(&mut self) {
        match build_zone_entry(&self.zone_form) {
            Ok(entry) => {
                push_and_sort_by(&mut self.data.zones, entry, |zone| zone.kamas_per_hour);
                self.zone_form = ZoneForm::default();
                self.zone_form_error = None;
                self.clear_zone_transient_state();
                self.persist_with_status("Zone ajoutée.");
            }
            Err(error) => self.zone_form_error = Some(error),
        }
    }

    pub fn submit_dungeon_form(&mut self) {
        match build_dungeon_entry(&self.dungeon_form) {
            Ok(entry) => {
                push_and_sort_by(&mut self.data.dungeons, entry, |dungeon| {
                    dungeon.kamas_per_hour
                });
                self.dungeon_form = DungeonForm::default();
                self.dungeon_form_error = None;
                self.clear_dungeon_transient_state();
                self.persist_with_status("Donjon ajouté.");
            }
            Err(error) => self.dungeon_form_error = Some(error),
        }
    }

    pub fn submit_duo_trio_form(&mut self) {
        match build_duo_trio_entry(&self.duo_trio_form) {
            Ok(entry) => {
                push_and_sort_by(&mut self.data.duo_trios, entry, |run| run.kamas_per_hour);
                self.duo_trio_form = DuoTrioForm::default();
                self.duo_trio_form_error = None;
                self.clear_duo_trio_transient_state();
                self.persist_with_status("Run duo/trio ajouté.");
            }
            Err(error) => self.duo_trio_form_error = Some(error),
        }
    }

    pub fn submit_arena_form(&mut self) {
        match build_arena_entry(&self.arena_form) {
            Ok(entry) => {
                push_and_sort_by(&mut self.data.arenas, entry, |arena| arena.kamas_per_hour);
                self.arena_form = ArenaForm::default();
                self.arena_form_error = None;
                self.clear_arena_transient_state();
                self.persist_with_status("PL arène ajouté.");
            }
            Err(error) => self.arena_form_error = Some(error),
        }
    }

    pub fn start_zone_edit(&mut self, index: usize) {
        if let Some(entry) = self.data.zones.get(index) {
            self.zone_edit = Some(InlineEditState {
                index,
                form: zone_form_from_entry(entry),
                error: None,
            });
            self.zone_delete_confirm = None;
        }
    }

    pub fn start_dungeon_edit(&mut self, index: usize) {
        if let Some(entry) = self.data.dungeons.get(index) {
            self.dungeon_edit = Some(InlineEditState {
                index,
                form: dungeon_form_from_entry(entry),
                error: None,
            });
            self.dungeon_delete_confirm = None;
        }
    }

    pub fn start_duo_trio_edit(&mut self, index: usize) {
        if let Some(entry) = self.data.duo_trios.get(index) {
            self.duo_trio_edit = Some(InlineEditState {
                index,
                form: duo_trio_form_from_entry(entry),
                error: None,
            });
            self.duo_trio_delete_confirm = None;
        }
    }

    pub fn start_arena_edit(&mut self, index: usize) {
        if let Some(entry) = self.data.arenas.get(index) {
            self.arena_edit = Some(InlineEditState {
                index,
                form: arena_form_from_entry(entry),
                error: None,
            });
            self.arena_delete_confirm = None;
        }
    }

    pub fn save_zone_edit(&mut self) {
        let Some(edit) = self.zone_edit.as_ref() else {
            return;
        };

        let index = edit.index;
        let form = edit.form.clone();

        match build_zone_entry(&form) {
            Ok(entry) => {
                replace_and_sort_by(&mut self.data.zones, index, entry, |zone| {
                    zone.kamas_per_hour
                });
                self.clear_zone_transient_state();
                self.persist_with_status("Zone mise à jour.");
            }
            Err(error) => {
                if let Some(edit) = self.zone_edit.as_mut() {
                    edit.error = Some(error);
                }
            }
        }
    }

    pub fn save_dungeon_edit(&mut self) {
        let Some(edit) = self.dungeon_edit.as_ref() else {
            return;
        };

        let index = edit.index;
        let form = edit.form.clone();

        match build_dungeon_entry(&form) {
            Ok(entry) => {
                replace_and_sort_by(&mut self.data.dungeons, index, entry, |dungeon| {
                    dungeon.kamas_per_hour
                });
                self.clear_dungeon_transient_state();
                self.persist_with_status("Donjon mis à jour.");
            }
            Err(error) => {
                if let Some(edit) = self.dungeon_edit.as_mut() {
                    edit.error = Some(error);
                }
            }
        }
    }

    pub fn save_duo_trio_edit(&mut self) {
        let Some(edit) = self.duo_trio_edit.as_ref() else {
            return;
        };

        let index = edit.index;
        let form = edit.form.clone();

        match build_duo_trio_entry(&form) {
            Ok(entry) => {
                replace_and_sort_by(&mut self.data.duo_trios, index, entry, |run| {
                    run.kamas_per_hour
                });
                self.clear_duo_trio_transient_state();
                self.persist_with_status("Run duo/trio mis à jour.");
            }
            Err(error) => {
                if let Some(edit) = self.duo_trio_edit.as_mut() {
                    edit.error = Some(error);
                }
            }
        }
    }

    pub fn save_arena_edit(&mut self) {
        let Some(edit) = self.arena_edit.as_ref() else {
            return;
        };

        let index = edit.index;
        let form = edit.form.clone();

        match build_arena_entry(&form) {
            Ok(entry) => {
                replace_and_sort_by(&mut self.data.arenas, index, entry, |arena| {
                    arena.kamas_per_hour
                });
                self.clear_arena_transient_state();
                self.persist_with_status("PL arène mis à jour.");
            }
            Err(error) => {
                if let Some(edit) = self.arena_edit.as_mut() {
                    edit.error = Some(error);
                }
            }
        }
    }

    pub fn cancel_zone_edit(&mut self) {
        self.zone_edit = None;
    }

    pub fn cancel_dungeon_edit(&mut self) {
        self.dungeon_edit = None;
    }

    pub fn cancel_duo_trio_edit(&mut self) {
        self.duo_trio_edit = None;
    }

    pub fn cancel_arena_edit(&mut self) {
        self.arena_edit = None;
    }

    pub fn confirm_zone_delete(&mut self, index: usize) {
        if index < self.data.zones.len() {
            self.data.zones.remove(index);
            self.clear_zone_transient_state();
            self.persist_with_status("Zone supprimée.");
        }
    }

    pub fn confirm_dungeon_delete(&mut self, index: usize) {
        if index < self.data.dungeons.len() {
            self.data.dungeons.remove(index);
            self.clear_dungeon_transient_state();
            self.persist_with_status("Donjon supprimé.");
        }
    }

    pub fn confirm_duo_trio_delete(&mut self, index: usize) {
        if index < self.data.duo_trios.len() {
            self.data.duo_trios.remove(index);
            self.clear_duo_trio_transient_state();
            self.persist_with_status("Run duo/trio supprimé.");
        }
    }

    pub fn confirm_arena_delete(&mut self, index: usize) {
        if index < self.data.arenas.len() {
            self.data.arenas.remove(index);
            self.clear_arena_transient_state();
            self.persist_with_status("PL arène supprimé.");
        }
    }

    pub fn clear_current_state(&mut self) {
        self.apply_persisted_state(PersistedState::default());
        self.show_clear_state_confirm = false;
        self.show_delete_local_save_confirm = false;
        self.set_status(
            StatusKind::Info,
            "État courant effacé. La sauvegarde locale n'a pas été modifiée.",
        );
    }

    pub fn delete_local_save(&mut self) {
        let had_local_state = storage::has_local_state();
        self.show_delete_local_save_confirm = false;

        match storage::delete_state() {
            Ok(()) if had_local_state => self.set_status(
                StatusKind::Success,
                "Sauvegardes locales supprimées. Les fichiers principaux et de secours des versions reconnues ont été retirés lorsqu'ils existaient.",
            ),
            Ok(()) => self.set_status(StatusKind::Info, "Aucune sauvegarde locale à supprimer."),
            Err(error) => self.set_status(
                StatusKind::Error,
                format!("Impossible de supprimer la sauvegarde locale : {error}"),
            ),
        }
    }

    fn load_from_action(&mut self, action: LoadAction) {
        match action {
            LoadAction::ReloadLocal => self.reload_local_data(),
            LoadAction::ImportExternal(path) => {
                let local_path = storage::data_file_path();
                self.import_from_path_to_save_path(&path, &local_path);
            }
        }
    }

    fn build_draft_state(&self) -> DraftState {
        DraftState {
            zone_form: self
                .zone_form
                .has_user_input()
                .then(|| self.zone_form.clone()),
            dungeon_form: self
                .dungeon_form
                .has_user_input()
                .then(|| self.dungeon_form.clone()),
            duo_trio_form: self
                .duo_trio_form
                .has_user_input()
                .then(|| self.duo_trio_form.clone()),
            arena_form: self
                .arena_form
                .has_user_input()
                .then(|| self.arena_form.clone()),
            zone_edit: self.zone_edit.as_ref().map(|edit| PersistedInlineEdit {
                index: edit.index,
                form: edit.form.clone(),
            }),
            dungeon_edit: self.dungeon_edit.as_ref().map(|edit| PersistedInlineEdit {
                index: edit.index,
                form: edit.form.clone(),
            }),
            duo_trio_edit: self.duo_trio_edit.as_ref().map(|edit| PersistedInlineEdit {
                index: edit.index,
                form: edit.form.clone(),
            }),
            arena_edit: self.arena_edit.as_ref().map(|edit| PersistedInlineEdit {
                index: edit.index,
                form: edit.form.clone(),
            }),
        }
    }

    fn build_persisted_state(&self) -> PersistedState {
        PersistedState {
            data: self.data.clone(),
            drafts: self.build_draft_state(),
            ..PersistedState::default()
        }
    }

    fn apply_persisted_state(&mut self, state: PersistedState) {
        let PersistedState { data, drafts, .. } = state;
        let DraftState {
            zone_form,
            dungeon_form,
            duo_trio_form,
            arena_form,
            zone_edit,
            dungeon_edit,
            duo_trio_edit,
            arena_edit,
        } = drafts;

        self.data = data;
        self.zone_form = zone_form.unwrap_or_default();
        self.dungeon_form = dungeon_form.unwrap_or_default();
        self.duo_trio_form = duo_trio_form.unwrap_or_default();
        self.arena_form = arena_form.unwrap_or_default();

        self.zone_edit = zone_edit.map(|edit| InlineEditState {
            index: edit.index,
            form: edit.form,
            error: None,
        });
        self.dungeon_edit = dungeon_edit.map(|edit| InlineEditState {
            index: edit.index,
            form: edit.form,
            error: None,
        });
        self.duo_trio_edit = duo_trio_edit.map(|edit| InlineEditState {
            index: edit.index,
            form: edit.form,
            error: None,
        });
        self.arena_edit = arena_edit.map(|edit| InlineEditState {
            index: edit.index,
            form: edit.form,
            error: None,
        });

        self.reset_non_persisted_ui_state();
    }

    fn apply_persisted_state_and_focus(&mut self, state: PersistedState) {
        self.apply_persisted_state(state);
        self.focus_first_restored_tab();
    }

    fn focus_first_restored_tab(&mut self) {
        let current_is_data_tab = matches!(
            self.current_tab,
            Tab::Zones | Tab::Donjons | Tab::DuoTrio | Tab::PlArene
        );

        if current_is_data_tab && self.tab_has_restored_content(self.current_tab) {
            return;
        }

        for tab in [Tab::Zones, Tab::Donjons, Tab::DuoTrio, Tab::PlArene] {
            if self.tab_has_restored_content(tab) {
                self.current_tab = tab;
                return;
            }
        }
    }

    fn tab_has_restored_content(&self, tab: Tab) -> bool {
        match tab {
            Tab::Zones => {
                !self.data.zones.is_empty()
                    || self.zone_form.has_user_input()
                    || self.zone_edit.is_some()
            }
            Tab::Donjons => {
                !self.data.dungeons.is_empty()
                    || self.dungeon_form.has_user_input()
                    || self.dungeon_edit.is_some()
            }
            Tab::DuoTrio => {
                !self.data.duo_trios.is_empty()
                    || self.duo_trio_form.has_user_input()
                    || self.duo_trio_edit.is_some()
            }
            Tab::PlArene => {
                !self.data.arenas.is_empty()
                    || self.arena_form.has_user_input()
                    || self.arena_edit.is_some()
            }
            Tab::Bilans | Tab::RechercheActivite => false,
        }
    }

    fn reload_local_data(&mut self) {
        match storage::load_state() {
            Ok(result) => self.apply_reload_state_result(result),
            Err(error) => self.set_status(
                StatusKind::Error,
                format!("Impossible de recharger la sauvegarde locale : {error}"),
            ),
        }
    }

    #[cfg(test)]
    fn reload_local_data_from_path(&mut self, path: &Path) {
        match storage::load_state_from_path(path) {
            Ok(result) => self.apply_reload_state_result(result),
            Err(error) => self.set_status(
                StatusKind::Error,
                format!("Impossible de recharger la sauvegarde locale : {error}"),
            ),
        }
    }

    fn apply_reload_state_result(&mut self, result: storage::LoadStateResult) {
        let mut status_kind = StatusKind::Success;
        let mut message = format!(
            "Sauvegarde locale rechargée. {} Fichier : {}",
            persisted_state_summary(&result.state),
            result.source_path.display()
        );

        if result.used_backup {
            status_kind = StatusKind::Info;
            message.push_str(" Le fichier principal était indisponible ou invalide ; la sauvegarde de secours .bak a été utilisée.");
        }

        if result.cleaned_legacy_entries > 0 {
            status_kind = StatusKind::Info;
            message.push_str(&format!(
                " {} entrée(s) legacy ont été ignorées au rechargement.",
                result.cleaned_legacy_entries
            ));
        }

        self.apply_persisted_state_and_focus(result.state);
        self.set_status(status_kind, message);
    }

    fn import_from_path_to_save_path(&mut self, source_path: &Path, save_path: &Path) {
        match storage::load_state_from_path(source_path) {
            Ok(result) => match storage::save_state_to_path(&result.state, save_path) {
                Ok(saved) => {
                    let mut message = format!(
                        "Import réussi. {} Source : {}. Sauvegarde locale mise à jour : {}",
                        persisted_state_summary(&result.state),
                        result.source_path.display(),
                        saved.path.display()
                    );

                    if result.used_backup {
                        message.push_str(" La version .bak du fichier importé a été utilisée.");
                    }

                    if result.cleaned_legacy_entries > 0 {
                        message.push_str(&format!(
                            " {} entrée(s) legacy ont été retirées pendant l'import.",
                            result.cleaned_legacy_entries
                        ));
                    }

                    if saved.backup_path.is_some() {
                        message.push_str(
                            " Une sauvegarde de secours de l'ancien fichier local a été créée.",
                        );
                    }

                    self.apply_persisted_state_and_focus(result.state);
                    self.set_status(StatusKind::Success, message);
                }
                Err(error) => self.set_status(
                    StatusKind::Error,
                    format!(
                        "Import impossible : la sauvegarde locale n'a pas été mise à jour ({error})."
                    ),
                ),
            },
            Err(error) => self.set_status(
                StatusKind::Error,
                format!("Impossible d'importer {} : {error}", source_path.display()),
            ),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn save_named_state_to_dir(&mut self, save_directory: &Path, overwrite_save_id: Option<&str>) {
        let state = self.build_persisted_state();
        let Some(display_name) = normalized_named_save_input(&self.named_save_name_input) else {
            self.named_save_error = Some("Le nom de la sauvegarde est obligatoire.".to_string());
            return;
        };

        match storage::save_named_state_in_dir(
            &display_name,
            &state,
            overwrite_save_id,
            save_directory,
        ) {
            Ok(result) => {
                self.named_save_error = None;
                let _ = self.refresh_named_saves_from_dir(save_directory);
                self.show_named_save_dialog = false;
                self.named_save_name_input.clear();

                let mut message = if result.replaced_existing {
                    format!(
                        "Sauvegarde nommee remplacee : {}. {} Fichier : {}",
                        result.meta.display_name,
                        persisted_state_summary(&state),
                        result.path.display()
                    )
                } else {
                    format!(
                        "Sauvegarde nommee creee : {}. {} Fichier : {}",
                        result.meta.display_name,
                        persisted_state_summary(&state),
                        result.path.display()
                    )
                };

                if result.backup_path.is_some() {
                    message.push_str(
                        " Une sauvegarde de secours de la version precedente a ete creee.",
                    );
                }

                self.set_status(StatusKind::Success, message);
            }
            Err(error) => {
                self.named_save_error = Some(error);
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn load_named_save_from_dir(&mut self, save_directory: &Path, save_id: &str) {
        match storage::load_named_save_in_dir(save_id, save_directory) {
            Ok(result) => {
                let mut status_kind = StatusKind::Success;
                let mut message = format!(
                    "Sauvegarde nommee chargee : {}. {} Fichier : {}",
                    result.meta.display_name,
                    persisted_state_summary(&result.state),
                    result.source_path.display()
                );

                if result.used_backup {
                    status_kind = StatusKind::Info;
                    message.push_str(
                        " Le fichier principal etait indisponible ou invalide ; la sauvegarde de secours .bak a ete utilisee.",
                    );
                }

                if result.cleaned_legacy_entries > 0 {
                    status_kind = StatusKind::Info;
                    message.push_str(&format!(
                        " {} entree(s) legacy ont ete ignorees au chargement.",
                        result.cleaned_legacy_entries
                    ));
                }

                self.apply_persisted_state_and_focus(result.state);
                self.set_status(status_kind, message);
            }
            Err(error) => self.set_status(
                StatusKind::Error,
                format!("Impossible de charger la sauvegarde nommee : {error}"),
            ),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn delete_named_save_in_dir(&mut self, save_directory: &Path, save_id: &str) {
        match storage::delete_named_save_in_dir(save_id, save_directory) {
            Ok(()) => {
                if self
                    .named_save_rename
                    .as_ref()
                    .is_some_and(|rename| rename.save_id == save_id)
                {
                    self.named_save_rename = None;
                }

                let _ = self.refresh_named_saves_from_dir(save_directory);
                self.set_status(StatusKind::Success, "Sauvegarde nommee supprimee.");
            }
            Err(error) => self.set_status(
                StatusKind::Error,
                format!("Impossible de supprimer la sauvegarde nommee : {error}"),
            ),
        }
    }

    pub fn set_status(&mut self, kind: StatusKind, message: impl Into<String>) {
        self.status = Some(StatusBanner {
            kind,
            message: message.into(),
        });
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    fn persist_with_status(&mut self, success_message: &str) {
        let path = storage::data_file_path();
        self.persist_with_status_to_path(success_message, &path);
    }

    fn persist_with_status_to_path(&mut self, success_message: &str, path: &Path) {
        let state = self.build_persisted_state();

        match storage::save_state_to_path(&state, path) {
            Ok(result) => self.set_status(
                StatusKind::Success,
                format!(
                    "{success_message} {} Fichier : {}",
                    persisted_state_summary(&state),
                    result.path.display()
                ),
            ),
            Err(error) => {
                self.set_status(StatusKind::Error, format!("Erreur de sauvegarde : {error}"))
            }
        }
    }

    fn clear_zone_transient_state(&mut self) {
        self.zone_edit = None;
        self.zone_delete_confirm = None;
    }

    fn clear_dungeon_transient_state(&mut self) {
        self.dungeon_edit = None;
        self.dungeon_delete_confirm = None;
    }

    fn clear_duo_trio_transient_state(&mut self) {
        self.duo_trio_edit = None;
        self.duo_trio_delete_confirm = None;
    }

    fn clear_arena_transient_state(&mut self) {
        self.arena_edit = None;
        self.arena_delete_confirm = None;
    }

    fn reset_non_persisted_ui_state(&mut self) {
        self.zone_form_error = None;
        self.dungeon_form_error = None;
        self.duo_trio_form_error = None;
        self.arena_form_error = None;
        self.activity_search_time_touched = false;
        self.activity_search_time_error = None;
        self.zone_search.clear();
        self.dungeon_search.clear();
        self.duo_trio_search.clear();
        self.arena_search.clear();
        self.zone_delete_confirm = None;
        self.dungeon_delete_confirm = None;
        self.duo_trio_delete_confirm = None;
        self.arena_delete_confirm = None;
        self.show_clear_state_confirm = false;
        self.show_delete_local_save_confirm = false;
        self.show_load_confirm = false;
        self.pending_load_action = None;
        self.show_named_save_dialog = false;
        self.show_named_load_dialog = false;
        self.show_named_save_confirm = false;
        self.pending_named_save_confirm_action = None;
        self.named_save_name_input.clear();
        self.named_save_error = None;
        self.named_saves_error = None;
        self.named_save_rename = None;
    }
}

fn persisted_state_summary(state: &PersistedState) -> String {
    let draft_count = state.drafts.restored_items_count();

    let has_data = !state.data.zones.is_empty()
        || !state.data.dungeons.is_empty()
        || !state.data.duo_trios.is_empty()
        || !state.data.arenas.is_empty();

    if !has_data && draft_count == 0 {
        return "La sauvegarde est vide.".to_string();
    }

    let mut message = format!(
        "Résumé : {} zone(s), {} donjon(s), {} run(s) duo/trio, {} session(s) PL arène",
        state.data.zones.len(),
        state.data.dungeons.len(),
        state.data.duo_trios.len(),
        state.data.arenas.len(),
    );

    if draft_count > 0 {
        message.push_str(&format!(", {draft_count} brouillon(s)"));
    }

    message.push('.');
    message
}

fn named_save_summary(summary: &storage::NamedSaveSummary) -> String {
    let has_data = summary.data_summary.zones > 0
        || summary.data_summary.dungeons > 0
        || summary.data_summary.duo_trios > 0
        || summary.data_summary.arenas > 0;

    if !has_data && summary.data_summary.draft_count == 0 {
        return "La sauvegarde est vide.".to_string();
    }

    let mut message = format!(
        "Resume : {} zone(s), {} donjon(s), {} run(s) duo/trio, {} session(s) PL arene",
        summary.data_summary.zones,
        summary.data_summary.dungeons,
        summary.data_summary.duo_trios,
        summary.data_summary.arenas,
    );

    if summary.data_summary.draft_count > 0 {
        message.push_str(&format!(
            ", {} brouillon(s)",
            summary.data_summary.draft_count
        ));
    }

    message.push('.');
    message
}

fn normalized_named_save_input(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.render(ctx);
    }
}

fn load_background_texture(ctx: &egui::Context) -> Result<TextureHandle, String> {
    load_texture_from_bytes(ctx, "app-background", APP_BACKGROUND_BYTES)
        .map_err(|error| format!("Image de fond invalide: {error}"))
}

fn load_app_icon_texture(ctx: &egui::Context) -> Result<TextureHandle, String> {
    load_texture_from_bytes(ctx, "app-icon", APP_ICON_BYTES)
        .map_err(|error| format!("Icône invalide: {error}"))
}

fn load_app_icon_data() -> Result<egui::IconData, String> {
    // Je conserve le même cadrage transparent pour la fenêtre et le binaire Windows.
    let canvas = icon_asset::square_icon(APP_ICON_BYTES, APP_ICON_SIZE)?;

    Ok(egui::IconData {
        rgba: canvas.into_raw(),
        width: APP_ICON_SIZE,
        height: APP_ICON_SIZE,
    })
}

fn class_texture_bytes(class: DofusClass) -> &'static [u8] {
    match class {
        DofusClass::Cra => CLASS_CRA_BYTES,
        DofusClass::Ecaflip => CLASS_ECAFLIP_BYTES,
        DofusClass::Eniripsa => CLASS_ENIRIPSA_BYTES,
        DofusClass::Enutrof => CLASS_ENUTROF_BYTES,
        DofusClass::Feca => CLASS_FECA_BYTES,
        DofusClass::Iop => CLASS_IOP_BYTES,
        DofusClass::Osamodas => CLASS_OSAMODAS_BYTES,
        DofusClass::Pandawa => CLASS_PANDAWA_BYTES,
        DofusClass::Sacrieur => CLASS_SACRIEUR_BYTES,
        DofusClass::Sadida => CLASS_SADIDA_BYTES,
        DofusClass::Sram => CLASS_SRAM_BYTES,
        DofusClass::Xelor => CLASS_XELOR_BYTES,
    }
}

fn load_class_textures(ctx: &egui::Context) -> HashMap<DofusClass, TextureHandle> {
    let mut textures = HashMap::new();

    for class in DofusClass::all() {
        let texture_id = format!("class-{}", class.storage_key());

        if let Ok(texture) = load_texture_from_bytes(ctx, &texture_id, class_texture_bytes(class)) {
            textures.insert(class, texture);
        }
    }

    textures
}

fn load_texture_from_bytes(
    ctx: &egui::Context,
    texture_id: &str,
    bytes: &[u8],
) -> Result<TextureHandle, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|error| error.to_string())?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color_image = ColorImage::from_rgba_unmultiplied(size, image.as_raw());

    Ok(ctx.load_texture(texture_id, color_image, TextureOptions::LINEAR))
}

fn require_character_class(class: Option<DofusClass>) -> Result<DofusClass, String> {
    class.ok_or_else(|| "La classe du personnage est obligatoire.".to_string())
}

fn build_zone_entry(form: &ZoneForm) -> Result<ZoneEntry, String> {
    let recorded_at = calculations::parse_recorded_at(&form.recorded_at_input)?;
    let session_time_seconds = calculations::parse_duration_input(&form.session_time)
        .map_err(|error| format!("Temps de session invalide. {error}"))?;
    let session_total_kamas = calculations::parse_non_negative_f32(
        &form.session_total_kamas,
        "Valeur totale de session",
    )?;

    calculations::sanitize_zone_entry(ZoneEntry {
        name: form.name.trim().to_string(),
        character_class: Some(require_character_class(form.character_class)?),
        recorded_at,
        session_time_seconds,
        session_total_kamas,
        kamas_per_hour: 0.0,
    })
}

fn build_dungeon_entry(form: &DungeonForm) -> Result<DungeonEntry, String> {
    let recorded_at = calculations::parse_recorded_at(&form.recorded_at_input)?;
    let run_time_seconds = calculations::parse_duration_input(&form.run_time)
        .map_err(|error| format!("Temps moyen du donjon invalide. {error}"))?;
    let gross_kamas_per_run =
        calculations::parse_non_negative_f32(&form.gross_kamas_per_run, "Gain brut moyen")?;
    let key_price = calculations::parse_non_negative_f32(&form.key_price, "Prix de la clé")?;

    calculations::sanitize_dungeon_entry(DungeonEntry {
        name: form.name.trim().to_string(),
        character_class: Some(require_character_class(form.character_class)?),
        recorded_at,
        run_time_minutes: calculations::seconds_to_minutes(run_time_seconds),
        gross_kamas_per_run,
        key_price,
        net_kamas_per_run: 0.0,
        kamas_per_hour: 0.0,
    })
}

fn build_duo_trio_entry(form: &DuoTrioForm) -> Result<DuoTrioEntry, String> {
    let recorded_at = calculations::parse_recorded_at(&form.recorded_at_input)?;
    let run_time_seconds = calculations::parse_duration_input(&form.run_time)
        .map_err(|error| format!("Temps du run invalide. {error}"))?;
    let loot_kamas_per_run =
        calculations::parse_non_negative_f32(&form.loot_kamas_per_run, "Loot total du run")?;
    let capture_stone_price = calculations::parse_non_negative_f32(
        &form.capture_stone_price,
        "Prix de la pierre de capture",
    )?;
    let key_unit_price =
        calculations::parse_non_negative_f32(&form.key_unit_price, "Prix unitaire de la clé")?;
    let full_soul_sale_price = calculations::parse_non_negative_f32(
        &form.full_soul_sale_price,
        "Prix de vente de la capture pleine",
    )?;

    calculations::sanitize_duo_trio_entry(DuoTrioEntry {
        name: form.name.trim().to_string(),
        character_class: Some(require_character_class(form.character_class)?),
        recorded_at,
        party_mode: form.party_mode,
        run_time_seconds,
        loot_kamas_per_run,
        capture_stone_price,
        key_unit_price,
        keys_count: 0,
        total_key_cost: 0.0,
        full_soul_sale_price,
        gross_kamas_per_run: 0.0,
        total_cost: 0.0,
        net_kamas_per_run: 0.0,
        kamas_per_hour: 0.0,
    })
}

fn build_arena_entry(form: &ArenaForm) -> Result<ArenaEntry, String> {
    let recorded_at = calculations::parse_recorded_at(&form.recorded_at_input)?;
    let round_time_seconds = calculations::parse_duration_input(&form.round_time)
        .map_err(|error| format!("Temps de ronde invalide. {error}"))?;
    let seat_price = calculations::parse_non_negative_f32(&form.seat_price, "Prix d'une place")?;
    let seats_sold = calculations::parse_u32(&form.seats_sold)
        .ok_or_else(|| "Nombre de places vendues invalide.".to_string())?;
    let capture_price =
        calculations::parse_non_negative_f32(&form.capture_price, "Prix d'une capture")?;
    let captures_count = calculations::parse_u32(&form.captures_count)
        .ok_or_else(|| "Nombre de captures invalide.".to_string())?;

    calculations::sanitize_arena_entry(ArenaEntry {
        name: form.name.trim().to_string(),
        character_class: Some(require_character_class(form.character_class)?),
        recorded_at,
        round_time_minutes: calculations::seconds_to_minutes(round_time_seconds),
        seat_price,
        seats_sold,
        capture_price,
        captures_count,
        gross_revenue: 0.0,
        total_capture_cost: 0.0,
        net_profit: 0.0,
        kamas_per_hour: 0.0,
    })
}

fn zone_form_from_entry(entry: &ZoneEntry) -> ZoneForm {
    ZoneForm {
        name: entry.name.clone(),
        character_class: entry.character_class,
        recorded_at_input: calculations::format_recorded_at(entry.recorded_at),
        session_time: calculations::duration_input_from_seconds(entry.session_time_seconds),
        session_total_kamas: calculations::format_number(entry.session_total_kamas),
    }
}

fn dungeon_form_from_entry(entry: &DungeonEntry) -> DungeonForm {
    DungeonForm {
        name: entry.name.clone(),
        character_class: entry.character_class,
        recorded_at_input: calculations::format_recorded_at(entry.recorded_at),
        run_time: calculations::duration_input_from_minutes(entry.run_time_minutes),
        gross_kamas_per_run: calculations::format_number(entry.gross_kamas_per_run),
        key_price: calculations::format_number(entry.key_price),
    }
}

fn duo_trio_form_from_entry(entry: &DuoTrioEntry) -> DuoTrioForm {
    DuoTrioForm {
        name: entry.name.clone(),
        character_class: entry.character_class,
        recorded_at_input: calculations::format_recorded_at(entry.recorded_at),
        party_mode: entry.party_mode,
        run_time: calculations::duration_input_from_seconds(entry.run_time_seconds),
        loot_kamas_per_run: calculations::format_number(entry.loot_kamas_per_run),
        capture_stone_price: calculations::format_number(entry.capture_stone_price),
        key_unit_price: calculations::format_number(entry.key_unit_price),
        full_soul_sale_price: calculations::format_number(entry.full_soul_sale_price),
    }
}

fn arena_form_from_entry(entry: &ArenaEntry) -> ArenaForm {
    ArenaForm {
        name: entry.name.clone(),
        character_class: entry.character_class,
        recorded_at_input: calculations::format_recorded_at(entry.recorded_at),
        round_time: calculations::duration_input_from_minutes(entry.round_time_minutes),
        seat_price: calculations::format_number(entry.seat_price),
        seats_sold: entry.seats_sold.to_string(),
        capture_price: calculations::format_number(entry.capture_price),
        captures_count: entry.captures_count.to_string(),
    }
}

pub fn push_and_sort_by<T, F>(items: &mut Vec<T>, item: T, score: F)
where
    F: Fn(&T) -> f32,
{
    items.push(item);
    items.sort_by(|left, right| score(right).total_cmp(&score(left)));
}

pub fn replace_and_sort_by<T, F>(items: &mut [T], index: usize, updated: T, score: F)
where
    F: Fn(&T) -> f32,
{
    if let Some(slot) = items.get_mut(index) {
        *slot = updated;
        items.sort_by(|left, right| score(right).total_cmp(&score(left)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DofusClass, DraftState, DurationInput, PartyMode, PersistedInlineEdit, PersistedState,
    };
    use chrono::NaiveDateTime;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn dt(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").unwrap()
    }

    fn duration_input(hours: &str, minutes: &str, seconds: &str) -> DurationInput {
        DurationInput {
            hours: hours.to_string(),
            minutes: minutes.to_string(),
            seconds: seconds.to_string(),
        }
    }

    #[test]
    fn embedded_visual_assets_load_once_and_cover_every_class() {
        let ctx = egui::Context::default();
        let mut app = MyApp::default();
        app.load_visual_assets(&ctx);

        let logo = app.app_icon_texture.as_ref().expect("Logo intégré valide");
        let background = app
            .background_texture
            .as_ref()
            .expect("Fond intégré valide");
        assert!(logo.size()[0] > 0 && logo.size()[1] > 0);
        assert!(background.size()[0] > background.size()[1]);
        assert_eq!(app.class_textures.len(), DofusClass::all().len());
        let logo_id = logo.id();
        let background_id = background.id();
        let class_ids = DofusClass::all().map(|class| app.class_textures[&class].id());

        // Je réutilise les textures déjà chargées au lieu de les recréer à chaque rendu.
        app.load_visual_assets(&ctx);
        assert_eq!(app.app_icon_texture.as_ref().unwrap().id(), logo_id);
        assert_eq!(app.background_texture.as_ref().unwrap().id(), background_id);
        for (class, id) in DofusClass::all().into_iter().zip(class_ids) {
            assert_eq!(app.class_textures[&class].id(), id);
        }
    }

    #[test]
    fn window_icon_uses_provided_logo_with_transparent_edges() {
        let icon = load_app_icon_data().expect("Icône de fenêtre valide");
        assert_eq!((icon.width, icon.height), (APP_ICON_SIZE, APP_ICON_SIZE));
        assert_eq!(
            icon.rgba.len(),
            (APP_ICON_SIZE * APP_ICON_SIZE * 4) as usize
        );
        assert!(icon.rgba.chunks_exact(4).any(|pixel| pixel[3] == 0));
        assert!(icon.rgba.chunks_exact(4).any(|pixel| pixel[3] == 255));
    }

    #[test]
    fn application_window_matches_desktop_identity_and_keeps_its_icon() {
        let viewport = app_viewport();
        assert_eq!(viewport.app_id.as_deref(), Some("fr.donj63000.evofarm"));
        assert_eq!(viewport.title.as_deref(), Some(APP_NAME));
        assert!(viewport.icon.is_some());
        assert_eq!(viewport.min_inner_size, Some(egui::vec2(920.0, 680.0)));
    }

    #[test]
    fn invalid_texture_returns_an_error_without_allocating_a_visual() {
        let ctx = egui::Context::default();
        assert!(load_texture_from_bytes(&ctx, "invalid-image", b"invalid PNG").is_err());
    }

    fn sample_loaded_data() -> AppData {
        AppData {
            zones: vec![ZoneEntry {
                name: "Chargée".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 3_600.0,
                session_total_kamas: 150_000.0,
                kamas_per_hour: 150_000.0,
            }],
            ..AppData::default()
        }
    }

    fn temp_state_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("evofarm-main-{name}-{unique}.json"))
    }

    fn backup_path_for(path: &Path) -> PathBuf {
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap();
        path.with_file_name(format!("{file_name}.bak"))
    }

    fn cleanup_temp_state(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(backup_path_for(path));
    }

    fn temp_named_saves_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("evofarm-main-saves-{name}-{unique}"))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    fn sample_dungeon_draft() -> DungeonForm {
        DungeonForm {
            name: "Draft Blop".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            run_time: duration_input("00", "20", "00"),
            gross_kamas_per_run: "120000".to_string(),
            key_price: "15000".to_string(),
        }
    }

    fn sample_zone_draft() -> ZoneForm {
        ZoneForm {
            name: "Brouillon zone".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            session_time: duration_input("01", "10", "00"),
            session_total_kamas: "250000".to_string(),
        }
    }

    #[test]
    fn apply_persisted_state_resets_transient_ui_state() {
        let mut app = MyApp {
            current_tab: Tab::Donjons,
            ..Default::default()
        };
        app.zone_form.name = "Zone".to_string();
        app.dungeon_form.name = "Donjon".to_string();
        app.duo_trio_form.name = "Run".to_string();
        app.arena_form.name = "Arena".to_string();
        app.zone_form_error = Some("zone".to_string());
        app.dungeon_form_error = Some("donjon".to_string());
        app.duo_trio_form_error = Some("run".to_string());
        app.arena_form_error = Some("arena".to_string());
        app.activity_search_class = Some(DofusClass::Cra);
        app.activity_search_time = duration_input("01", "15", "00");
        app.activity_search_time_touched = true;
        app.activity_search_time_error = Some("temps".to_string());
        app.zone_search = "plaine".to_string();
        app.dungeon_search = "blop".to_string();
        app.duo_trio_search = "trio".to_string();
        app.arena_search = "arene".to_string();
        app.zone_edit = Some(InlineEditState {
            index: 0,
            form: ZoneForm::default(),
            error: Some("edit".to_string()),
        });
        app.dungeon_edit = Some(InlineEditState {
            index: 0,
            form: DungeonForm::default(),
            error: Some("edit".to_string()),
        });
        app.duo_trio_edit = Some(InlineEditState {
            index: 0,
            form: DuoTrioForm::default(),
            error: Some("edit".to_string()),
        });
        app.arena_edit = Some(InlineEditState {
            index: 0,
            form: ArenaForm::default(),
            error: Some("edit".to_string()),
        });
        app.zone_delete_confirm = Some(0);
        app.dungeon_delete_confirm = Some(0);
        app.duo_trio_delete_confirm = Some(0);
        app.arena_delete_confirm = Some(0);
        app.show_clear_state_confirm = true;
        app.show_delete_local_save_confirm = true;
        app.show_load_confirm = true;
        app.pending_load_action = Some(LoadAction::ReloadLocal);
        app.show_named_save_dialog = true;
        app.show_named_load_dialog = true;
        app.show_named_save_confirm = true;
        app.pending_named_save_confirm_action = Some(NamedSaveConfirmAction::Load {
            save_id: "save-test".to_string(),
            display_name: "Save test".to_string(),
        });
        app.named_save_name_input = "Save test".to_string();
        app.named_save_error = Some("save".to_string());
        app.named_saves_error = Some("list".to_string());
        app.named_save_rename = Some(NamedSaveRenameState {
            save_id: "save-test".to_string(),
            value: "Rename".to_string(),
            error: Some("rename".to_string()),
        });

        app.apply_persisted_state(PersistedState {
            data: sample_loaded_data(),
            ..PersistedState::default()
        });

        assert_eq!(app.current_tab, Tab::Donjons);
        assert_eq!(app.data.zones.len(), 1);
        assert!(app.zone_form.name.is_empty());
        assert!(app.dungeon_form.name.is_empty());
        assert!(app.duo_trio_form.name.is_empty());
        assert!(app.arena_form.name.is_empty());
        assert!(app.zone_form_error.is_none());
        assert!(app.dungeon_form_error.is_none());
        assert!(app.duo_trio_form_error.is_none());
        assert!(app.arena_form_error.is_none());
        assert_eq!(app.activity_search_class, Some(DofusClass::Cra));
        assert_eq!(app.activity_search_time, duration_input("01", "15", "00"));
        assert!(!app.activity_search_time_touched);
        assert!(app.activity_search_time_error.is_none());
        assert!(app.zone_search.is_empty());
        assert!(app.dungeon_search.is_empty());
        assert!(app.duo_trio_search.is_empty());
        assert!(app.arena_search.is_empty());
        assert!(app.zone_edit.is_none());
        assert!(app.dungeon_edit.is_none());
        assert!(app.duo_trio_edit.is_none());
        assert!(app.arena_edit.is_none());
        assert!(app.zone_delete_confirm.is_none());
        assert!(app.dungeon_delete_confirm.is_none());
        assert!(app.duo_trio_delete_confirm.is_none());
        assert!(app.arena_delete_confirm.is_none());
        assert!(!app.show_clear_state_confirm);
        assert!(!app.show_delete_local_save_confirm);
        assert!(!app.show_load_confirm);
        assert!(app.pending_load_action.is_none());
        assert!(!app.show_named_save_dialog);
        assert!(!app.show_named_load_dialog);
        assert!(!app.show_named_save_confirm);
        assert!(app.pending_named_save_confirm_action.is_none());
        assert!(app.named_save_error.is_none());
        assert!(app.named_saves_error.is_none());
        assert!(app.named_save_rename.is_none());
    }

    #[test]
    fn clear_current_state_clears_memory_only() {
        let mut app = MyApp {
            data: sample_loaded_data(),
            zone_form: ZoneForm {
                name: "Brouillon".to_string(),
                ..ZoneForm::default()
            },
            zone_edit: Some(InlineEditState {
                index: 0,
                form: ZoneForm {
                    name: "Edition".to_string(),
                    ..ZoneForm::default()
                },
                error: Some("edit".to_string()),
            }),
            show_clear_state_confirm: true,
            show_delete_local_save_confirm: true,
            show_load_confirm: true,
            pending_load_action: Some(LoadAction::ReloadLocal),
            ..Default::default()
        };

        app.clear_current_state();

        assert!(app.data.zones.is_empty());
        assert!(app.zone_form.name.is_empty());
        assert!(app.zone_edit.is_none());
        assert!(!app.show_clear_state_confirm);
        assert!(!app.show_delete_local_save_confirm);
        assert!(!app.show_load_confirm);
        assert!(app.pending_load_action.is_none());
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Info)
        ));
    }

    #[test]
    fn persist_with_status_to_path_saves_data_and_drafts() {
        let path = temp_state_path("persist-with-status");
        let mut app = MyApp {
            data: sample_loaded_data(),
            dungeon_form: sample_dungeon_draft(),
            ..Default::default()
        };

        app.persist_with_status_to_path("Etat local sauvegarde.", &path);

        let reloaded = storage::load_state_from_path(&path).unwrap();

        assert_eq!(reloaded.state.data.zones.len(), 1);
        assert_eq!(
            reloaded.state.drafts.dungeon_form.as_ref().unwrap().name,
            "Draft Blop"
        );
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));
        assert!(app
            .status
            .as_ref()
            .unwrap()
            .message
            .contains("1 brouillon(s)"));

        cleanup_temp_state(&path);
    }

    #[test]
    fn save_named_state_to_dir_saves_data_and_drafts() {
        let directory = temp_named_saves_dir("save-named");
        let mut app = MyApp {
            data: sample_loaded_data(),
            dungeon_form: sample_dungeon_draft(),
            show_named_save_dialog: true,
            named_save_name_input: "  Route   Glours ".to_string(),
            ..Default::default()
        };

        app.save_named_state_to_dir(&directory, None);

        let saves = storage::list_named_saves_in_dir(&directory).unwrap();
        let loaded = storage::load_named_save_in_dir(&saves[0].meta.save_id, &directory).unwrap();

        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].meta.display_name, "Route Glours");
        assert_eq!(loaded.state.data.zones.len(), 1);
        assert_eq!(
            loaded.state.drafts.dungeon_form.as_ref().unwrap().name,
            "Draft Blop"
        );
        assert!(!app.show_named_save_dialog);
        assert!(app.named_save_name_input.is_empty());
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));

        cleanup_temp_dir(&directory);
    }

    #[test]
    fn open_named_dialogs_toggle_visibility() {
        let mut app = MyApp {
            show_named_load_dialog: true,
            named_save_name_input: "Ancien nom".to_string(),
            ..Default::default()
        };

        app.open_named_save_dialog();
        assert!(app.show_named_save_dialog);
        assert!(!app.show_named_load_dialog);
        assert!(app.named_save_name_input.is_empty());

        app.close_named_save_dialog();
        assert!(!app.show_named_save_dialog);
        assert!(app.named_save_name_input.is_empty());
    }

    #[test]
    fn request_named_save_in_dir_with_existing_name_opens_overwrite_confirmation() {
        let directory = temp_named_saves_dir("save-overwrite-confirm");
        let saved = storage::save_named_state_in_dir(
            "Route Glours",
            &PersistedState::default(),
            None,
            &directory,
        )
        .unwrap();
        let mut app = MyApp {
            show_named_save_dialog: true,
            named_save_name_input: " route   glours ".to_string(),
            ..Default::default()
        };

        app.request_named_save_in_dir(&directory);

        assert!(app.show_named_save_confirm);
        assert_eq!(
            app.pending_named_save_confirm_action,
            Some(NamedSaveConfirmAction::Overwrite {
                save_id: saved.meta.save_id,
                display_name: "Route Glours".to_string(),
            })
        );

        cleanup_temp_dir(&directory);
    }

    #[test]
    fn load_named_save_from_dir_restores_saved_state() {
        let directory = temp_named_saves_dir("load-named");
        let state = PersistedState {
            data: sample_loaded_data(),
            drafts: DraftState {
                zone_form: Some(sample_zone_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };
        let saved =
            storage::save_named_state_in_dir("Route load", &state, None, &directory).unwrap();
        let mut app = MyApp {
            current_tab: Tab::Bilans,
            ..Default::default()
        };

        app.load_named_save_from_dir(&directory, &saved.meta.save_id);

        assert_eq!(app.current_tab, Tab::Zones);
        assert_eq!(app.data.zones.len(), 1);
        assert_eq!(app.zone_form.name, "Brouillon zone");
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));

        cleanup_temp_dir(&directory);
    }

    #[test]
    fn save_named_save_rename_in_dir_refreshes_list() {
        let directory = temp_named_saves_dir("rename-named");
        let saved = storage::save_named_state_in_dir(
            "Route rename",
            &PersistedState::default(),
            None,
            &directory,
        )
        .unwrap();
        let mut app = MyApp::default();
        app.refresh_named_saves_from_dir(&directory).unwrap();
        app.named_save_rename = Some(NamedSaveRenameState {
            save_id: saved.meta.save_id.clone(),
            value: " Route   rename xl ".to_string(),
            error: None,
        });

        app.save_named_save_rename_in_dir(&directory, &saved.meta.save_id, " Route   rename xl ");

        assert!(app.named_save_rename.is_none());
        assert_eq!(app.named_saves.len(), 1);
        assert_eq!(app.named_saves[0].meta.display_name, "Route rename xl");
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));

        cleanup_temp_dir(&directory);
    }

    #[test]
    fn delete_named_save_in_dir_refreshes_list() {
        let directory = temp_named_saves_dir("delete-named");
        let saved = storage::save_named_state_in_dir(
            "Route delete",
            &PersistedState::default(),
            None,
            &directory,
        )
        .unwrap();
        let mut app = MyApp::default();
        app.refresh_named_saves_from_dir(&directory).unwrap();

        app.delete_named_save_in_dir(&directory, &saved.meta.save_id);

        assert!(app.named_saves.is_empty());
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));

        cleanup_temp_dir(&directory);
    }

    #[test]
    fn reload_local_data_from_path_restores_saved_state_after_clear() {
        let path = temp_state_path("reload-after-clear");
        let state = PersistedState {
            data: sample_loaded_data(),
            drafts: DraftState {
                dungeon_form: Some(sample_dungeon_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };
        storage::save_state_to_path(&state, &path).unwrap();

        let mut app = MyApp {
            current_tab: Tab::Bilans,
            data: sample_loaded_data(),
            ..Default::default()
        };

        app.clear_current_state();
        assert!(app.data.zones.is_empty());

        app.reload_local_data_from_path(&path);

        assert_eq!(app.current_tab, Tab::Zones);
        assert_eq!(app.data.zones.len(), 1);
        assert_eq!(app.dungeon_form.name, "Draft Blop");
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));
        assert!(app
            .status
            .as_ref()
            .unwrap()
            .message
            .contains("Sauvegarde locale rechargée."));

        cleanup_temp_state(&path);
    }

    #[test]
    fn reload_local_data_from_path_uses_backup_and_reports_it() {
        let path = temp_state_path("reload-backup");
        let backup_path = backup_path_for(&path);
        let state = PersistedState {
            drafts: DraftState {
                zone_form: Some(sample_zone_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        fs::write(&path, "{ invalid json }").unwrap();
        fs::write(&backup_path, serde_json::to_string_pretty(&state).unwrap()).unwrap();

        let mut app = MyApp {
            current_tab: Tab::Bilans,
            ..Default::default()
        };

        app.reload_local_data_from_path(&path);

        assert_eq!(app.current_tab, Tab::Zones);
        assert_eq!(app.zone_form.name, "Brouillon zone");
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Info)
        ));
        assert!(app
            .status
            .as_ref()
            .unwrap()
            .message
            .contains(".bak a été utilisée"));

        cleanup_temp_state(&path);
    }

    #[test]
    fn reload_local_data_from_path_keeps_current_state_on_error() {
        let path = temp_state_path("reload-missing");
        let mut app = MyApp {
            data: sample_loaded_data(),
            zone_form: sample_zone_draft(),
            current_tab: Tab::Donjons,
            ..Default::default()
        };

        app.reload_local_data_from_path(&path);

        assert_eq!(app.current_tab, Tab::Donjons);
        assert_eq!(app.data.zones.len(), 1);
        assert_eq!(app.zone_form.name, "Brouillon zone");
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Error)
        ));
    }

    #[test]
    fn apply_persisted_state_restores_drafts_and_inline_edits() {
        let mut app = MyApp {
            current_tab: Tab::Bilans,
            activity_search_class: Some(DofusClass::Cra),
            activity_search_time: duration_input("01", "15", "00"),
            activity_search_time_touched: true,
            activity_search_time_error: Some("temps".to_string()),
            ..Default::default()
        };

        let zone_draft = sample_zone_draft();

        let state = PersistedState {
            data: sample_loaded_data(),
            drafts: DraftState {
                zone_form: Some(zone_draft.clone()),
                zone_edit: Some(PersistedInlineEdit {
                    index: 0,
                    form: zone_draft.clone(),
                }),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        app.apply_persisted_state_and_focus(state);

        assert_eq!(app.current_tab, Tab::Zones);
        assert_eq!(app.zone_form.name, "Brouillon zone");
        assert!(app.zone_edit.is_some());
        assert_eq!(app.zone_edit.as_ref().unwrap().index, 0);
        assert_eq!(app.zone_edit.as_ref().unwrap().form.name, "Brouillon zone");
        assert!(app.zone_edit.as_ref().unwrap().error.is_none());
        assert_eq!(app.activity_search_class, Some(DofusClass::Cra));
        assert_eq!(app.activity_search_time, duration_input("01", "15", "00"));
        assert!(!app.activity_search_time_touched);
        assert!(app.activity_search_time_error.is_none());
    }

    #[test]
    fn failed_import_keeps_current_data() {
        let mut app = MyApp {
            data: sample_loaded_data(),
            ..Default::default()
        };

        let source_path = temp_state_path("missing-import-source");
        let target_path = temp_state_path("missing-import-destination");
        app.import_from_path_to_save_path(&source_path, &target_path);

        assert_eq!(app.data.zones.len(), 1);
        assert_eq!(app.data.zones[0].name, "Chargée");
        assert!(!target_path.exists());
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Error)
        ));
    }

    #[test]
    fn import_from_path_to_save_path_restores_state_and_creates_backup() {
        let source_path = temp_state_path("import-source");
        let target_path = temp_state_path("import-target");
        let target_backup_path = backup_path_for(&target_path);
        let state = PersistedState {
            data: sample_loaded_data(),
            drafts: DraftState {
                zone_form: Some(sample_zone_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        storage::save_state_to_path(&state, &source_path).unwrap();
        fs::write(&target_path, r#"{"marker":"ancienne-version"}"#).unwrap();

        let mut app = MyApp {
            current_tab: Tab::Bilans,
            ..Default::default()
        };

        app.import_from_path_to_save_path(&source_path, &target_path);

        let imported = storage::load_state_from_path(&target_path).unwrap();

        assert_eq!(app.current_tab, Tab::Zones);
        assert_eq!(imported.state.data.zones.len(), 1);
        assert_eq!(
            imported.state.drafts.zone_form.as_ref().unwrap().name,
            "Brouillon zone"
        );
        assert!(target_backup_path.exists());
        assert!(fs::read_to_string(&target_backup_path)
            .unwrap()
            .contains("ancienne-version"));
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));
        assert!(app
            .status
            .as_ref()
            .unwrap()
            .message
            .contains("Import réussi."));
        assert!(app
            .status
            .as_ref()
            .unwrap()
            .message
            .contains("sauvegarde de secours"));

        cleanup_temp_state(&source_path);
        cleanup_temp_state(&target_path);
    }

    #[test]
    fn import_from_path_to_save_path_uses_backup_source_when_needed() {
        let source_path = temp_state_path("import-source-backup");
        let source_backup_path = backup_path_for(&source_path);
        let target_path = temp_state_path("import-target-backup");
        let state = PersistedState {
            drafts: DraftState {
                dungeon_form: Some(sample_dungeon_draft()),
                ..DraftState::default()
            },
            ..PersistedState::default()
        };

        fs::write(&source_path, "{ invalid json }").unwrap();
        fs::write(
            &source_backup_path,
            serde_json::to_string_pretty(&state).unwrap(),
        )
        .unwrap();

        let mut app = MyApp {
            current_tab: Tab::Bilans,
            ..Default::default()
        };

        app.import_from_path_to_save_path(&source_path, &target_path);

        let imported = storage::load_state_from_path(&target_path).unwrap();

        assert_eq!(app.current_tab, Tab::Donjons);
        assert_eq!(
            imported.state.drafts.dungeon_form.as_ref().unwrap().name,
            "Draft Blop"
        );
        assert!(matches!(
            app.status.as_ref().map(|status| status.kind),
            Some(StatusKind::Success)
        ));
        assert!(app
            .status
            .as_ref()
            .unwrap()
            .message
            .contains(".bak du fichier importé a été utilisée"));

        cleanup_temp_state(&source_path);
        cleanup_temp_state(&target_path);
    }

    #[test]
    fn replace_and_sort_promotes_updated_entry() {
        let mut zones = vec![
            ZoneEntry {
                name: "Plaines".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 3_600.0,
                session_total_kamas: 200_000.0,
                kamas_per_hour: 200_000.0,
            },
            ZoneEntry {
                name: "Forêt".to_string(),
                character_class: Some(DofusClass::Sadida),
                recorded_at: None,
                session_time_seconds: 5_400.0,
                session_total_kamas: 135_000.0,
                kamas_per_hour: 90_000.0,
            },
        ];

        replace_and_sort_by(
            &mut zones,
            1,
            ZoneEntry {
                name: "Forêt".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 3_600.0,
                session_total_kamas: 300_000.0,
                kamas_per_hour: 300_000.0,
            },
            |entry| entry.kamas_per_hour,
        );

        assert_eq!(zones[0].name, "Forêt");
        assert_eq!(zones[0].kamas_per_hour, 300_000.0);
    }

    #[test]
    fn build_zone_entry_uses_session_values() {
        let form = ZoneForm {
            name: "Cimetiere".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            session_time: duration_input("01", "30", "00"),
            session_total_kamas: "450000".to_string(),
        };

        let entry = build_zone_entry(&form).unwrap();

        assert_eq!(entry.name, "Cimetiere");
        assert_eq!(entry.character_class, Some(DofusClass::Cra));
        assert_eq!(entry.recorded_at, Some(dt("2026-03-08 14:45")));
        assert_eq!(entry.session_time_seconds, 5_400.0);
        assert_eq!(entry.session_total_kamas, 450_000.0);
        assert_eq!(entry.kamas_per_hour, 300_000.0);
    }

    #[test]
    fn build_zone_entry_requires_character_class() {
        let form = ZoneForm {
            name: "Cimetiere".to_string(),
            character_class: None,
            recorded_at_input: "2026-03-08 14:45".to_string(),
            session_time: duration_input("01", "30", "00"),
            session_total_kamas: "450000".to_string(),
        };

        let error = build_zone_entry(&form).unwrap_err();

        assert_eq!(error, "La classe du personnage est obligatoire.");
    }

    #[test]
    fn build_duo_trio_entry_uses_party_size_and_capture_values() {
        let form = DuoTrioForm {
            name: "Trio Illy".to_string(),
            character_class: Some(DofusClass::Enutrof),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            party_mode: PartyMode::Trio,
            run_time: duration_input("01", "20", "00"),
            loot_kamas_per_run: "320000".to_string(),
            capture_stone_price: "45000".to_string(),
            key_unit_price: "12000".to_string(),
            full_soul_sale_price: "180000".to_string(),
        };

        let entry = build_duo_trio_entry(&form).unwrap();

        assert_eq!(entry.character_class, Some(DofusClass::Enutrof));
        assert_eq!(entry.party_mode, PartyMode::Trio);
        assert_eq!(entry.keys_count, 3);
        assert_eq!(entry.total_key_cost, 36_000.0);
        assert_eq!(entry.gross_kamas_per_run, 500_000.0);
        assert_eq!(entry.total_cost, 81_000.0);
        assert_eq!(entry.net_kamas_per_run, 419_000.0);
        assert_eq!(entry.kamas_per_hour, 314_250.0);
    }

    #[test]
    fn build_dungeon_entry_accepts_grouped_numeric_inputs() {
        let form = DungeonForm {
            name: "Blop".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            run_time: duration_input("00", "20", "00"),
            gross_kamas_per_run: "120 000".to_string(),
            key_price: "15_000".to_string(),
        };

        let entry = build_dungeon_entry(&form).unwrap();

        assert_eq!(entry.net_kamas_per_run, 105_000.0);
        assert_eq!(entry.kamas_per_hour, 315_000.0);
    }

    #[test]
    fn build_arena_entry_accepts_grouped_numeric_inputs() {
        let form = ArenaForm {
            name: "Bworker".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: "2026-03-08 14:45".to_string(),
            round_time: duration_input("00", "30", "00"),
            seat_price: "50 000".to_string(),
            seats_sold: "7".to_string(),
            capture_price: "120_000".to_string(),
            captures_count: "10".to_string(),
        };

        let entry = build_arena_entry(&form).unwrap();

        assert_eq!(entry.gross_revenue, 350_000.0);
        assert_eq!(entry.total_capture_cost, 1_200_000.0);
        assert_eq!(entry.net_profit, -850_000.0);
        assert_eq!(entry.kamas_per_hour, -1_700_000.0);
    }

    #[test]
    fn build_dungeon_entry_requires_character_class() {
        let form = DungeonForm {
            name: "Blop".to_string(),
            character_class: None,
            recorded_at_input: "2026-03-08 14:45".to_string(),
            run_time: duration_input("00", "20", "00"),
            gross_kamas_per_run: "120000".to_string(),
            key_price: "15000".to_string(),
        };

        let error = build_dungeon_entry(&form).unwrap_err();

        assert_eq!(error, "La classe du personnage est obligatoire.");
    }

    #[test]
    fn build_duo_trio_entry_requires_character_class() {
        let form = DuoTrioForm {
            name: "Trio Illy".to_string(),
            character_class: None,
            recorded_at_input: "2026-03-08 14:45".to_string(),
            party_mode: PartyMode::Trio,
            run_time: duration_input("01", "20", "00"),
            loot_kamas_per_run: "320000".to_string(),
            capture_stone_price: "45000".to_string(),
            key_unit_price: "12000".to_string(),
            full_soul_sale_price: "180000".to_string(),
        };

        let error = build_duo_trio_entry(&form).unwrap_err();

        assert_eq!(error, "La classe du personnage est obligatoire.");
    }

    #[test]
    fn build_arena_entry_requires_character_class() {
        let form = ArenaForm {
            name: "Bworker".to_string(),
            character_class: None,
            recorded_at_input: "2026-03-08 14:45".to_string(),
            round_time: duration_input("00", "30", "00"),
            seat_price: "50000".to_string(),
            seats_sold: "7".to_string(),
            capture_price: "120000".to_string(),
            captures_count: "10".to_string(),
        };

        let error = build_arena_entry(&form).unwrap_err();

        assert_eq!(error, "La classe du personnage est obligatoire.");
    }

    #[test]
    fn replace_and_sort_ignores_out_of_bounds_index() {
        let mut zones = vec![ZoneEntry {
            name: "Plaine".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at: None,
            session_time_seconds: 3_600.0,
            session_total_kamas: 100_000.0,
            kamas_per_hour: 100_000.0,
        }];

        replace_and_sort_by(
            &mut zones,
            10,
            ZoneEntry {
                name: "Foret".to_string(),
                character_class: Some(DofusClass::Feca),
                recorded_at: None,
                session_time_seconds: 1_800.0,
                session_total_kamas: 200_000.0,
                kamas_per_hour: 400_000.0,
            },
            |entry| entry.kamas_per_hour,
        );

        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].name, "Plaine");
    }

    #[test]
    fn replace_and_sort_promotes_duo_trio_edit() {
        let mut runs = vec![
            DuoTrioEntry {
                name: "Duo Qu'Tan".to_string(),
                character_class: Some(DofusClass::Iop),
                recorded_at: None,
                party_mode: PartyMode::Duo,
                run_time_seconds: 3_600.0,
                loot_kamas_per_run: 120_000.0,
                capture_stone_price: 40_000.0,
                key_unit_price: 10_000.0,
                keys_count: 2,
                total_key_cost: 20_000.0,
                full_soul_sale_price: 110_000.0,
                gross_kamas_per_run: 230_000.0,
                total_cost: 60_000.0,
                net_kamas_per_run: 170_000.0,
                kamas_per_hour: 170_000.0,
            },
            DuoTrioEntry {
                name: "Trio Illy".to_string(),
                character_class: Some(DofusClass::Enutrof),
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
            },
        ];

        replace_and_sort_by(
            &mut runs,
            0,
            DuoTrioEntry {
                name: "Duo Qu'Tan".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                party_mode: PartyMode::Trio,
                run_time_seconds: 3_600.0,
                loot_kamas_per_run: 350_000.0,
                capture_stone_price: 40_000.0,
                key_unit_price: 10_000.0,
                keys_count: 3,
                total_key_cost: 30_000.0,
                full_soul_sale_price: 200_000.0,
                gross_kamas_per_run: 550_000.0,
                total_cost: 70_000.0,
                net_kamas_per_run: 480_000.0,
                kamas_per_hour: 480_000.0,
            },
            |entry| entry.kamas_per_hour,
        );

        assert_eq!(runs[0].name, "Duo Qu'Tan");
        assert_eq!(runs[0].character_class, Some(DofusClass::Cra));
        assert_eq!(runs[0].party_mode, PartyMode::Trio);
        assert_eq!(runs[0].kamas_per_hour, 480_000.0);
    }
}
