use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Bilans,
    RechercheActivite,
    Zones,
    Donjons,
    DuoTrio,
    PlArene,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bilans => "Bilans",
            Self::RechercheActivite => "Recherche d’activité",
            Self::Zones => "Zones",
            Self::Donjons => "Donjons",
            Self::DuoTrio => "Duo / Trio",
            Self::PlArene => "PL arène",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PartyMode {
    #[default]
    #[serde(alias = "Duo")]
    Duo,
    #[serde(alias = "Trio")]
    Trio,
}

impl PartyMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Duo => "Duo",
            Self::Trio => "Trio",
        }
    }

    pub fn player_count(self) -> u32 {
        match self {
            Self::Duo => 2,
            Self::Trio => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DofusClass {
    Cra,
    Ecaflip,
    Eniripsa,
    Enutrof,
    Feca,
    Iop,
    Osamodas,
    Pandawa,
    Sacrieur,
    Sadida,
    Sram,
    Xelor,
}

impl DofusClass {
    pub const fn all() -> [Self; 12] {
        [
            Self::Cra,
            Self::Ecaflip,
            Self::Eniripsa,
            Self::Enutrof,
            Self::Feca,
            Self::Iop,
            Self::Osamodas,
            Self::Pandawa,
            Self::Sacrieur,
            Self::Sadida,
            Self::Sram,
            Self::Xelor,
        ]
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Cra => "Cr\u{00E2}",
            Self::Ecaflip => "Ecaflip",
            Self::Eniripsa => "Eniripsa",
            Self::Enutrof => "Enutrof",
            Self::Feca => "F\u{00E9}ca",
            Self::Iop => "Iop",
            Self::Osamodas => "Osamodas",
            Self::Pandawa => "Pandawa",
            Self::Sacrieur => "Sacrieur",
            Self::Sadida => "Sadida",
            Self::Sram => "Sram",
            Self::Xelor => "X\u{00E9}lor",
        }
    }

    pub const fn asset_file_name(self) -> &'static str {
        match self {
            Self::Cra => "cra.png",
            Self::Ecaflip => "ecaflip.png",
            Self::Eniripsa => "eniripsa.png",
            Self::Enutrof => "enutrof.png",
            Self::Feca => "feca.png",
            Self::Iop => "iop.png",
            Self::Osamodas => "osamodas.png",
            Self::Pandawa => "pandawa.png",
            Self::Sacrieur => "sacrieur.png",
            Self::Sadida => "sadida.png",
            Self::Sram => "sram.png",
            Self::Xelor => "xelor.png",
        }
    }

    pub const fn storage_key(self) -> &'static str {
        match self {
            Self::Cra => "cra",
            Self::Ecaflip => "ecaflip",
            Self::Eniripsa => "eniripsa",
            Self::Enutrof => "enutrof",
            Self::Feca => "feca",
            Self::Iop => "iop",
            Self::Osamodas => "osamodas",
            Self::Pandawa => "pandawa",
            Self::Sacrieur => "sacrieur",
            Self::Sadida => "sadida",
            Self::Sram => "sram",
            Self::Xelor => "xelor",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ZoneEntry {
    pub name: String,
    #[serde(default)]
    pub character_class: Option<DofusClass>,
    #[serde(default)]
    pub recorded_at: Option<NaiveDateTime>,
    pub session_time_seconds: f32,
    pub session_total_kamas: f32,
    pub kamas_per_hour: f32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct DungeonEntry {
    pub name: String,
    #[serde(default)]
    pub character_class: Option<DofusClass>,
    #[serde(default)]
    pub recorded_at: Option<NaiveDateTime>,
    pub run_time_minutes: f32,
    pub gross_kamas_per_run: f32,
    pub key_price: f32,
    pub net_kamas_per_run: f32,
    pub kamas_per_hour: f32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct DuoTrioEntry {
    pub name: String,
    #[serde(default)]
    pub character_class: Option<DofusClass>,
    #[serde(default)]
    pub recorded_at: Option<NaiveDateTime>,
    pub party_mode: PartyMode,
    pub run_time_seconds: f32,
    pub loot_kamas_per_run: f32,
    pub capture_stone_price: f32,
    pub key_unit_price: f32,
    pub keys_count: u32,
    pub total_key_cost: f32,
    pub full_soul_sale_price: f32,
    pub gross_kamas_per_run: f32,
    pub total_cost: f32,
    pub net_kamas_per_run: f32,
    pub kamas_per_hour: f32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ArenaEntry {
    pub name: String,
    #[serde(default)]
    pub character_class: Option<DofusClass>,
    #[serde(default)]
    pub recorded_at: Option<NaiveDateTime>,
    pub round_time_minutes: f32,
    pub seat_price: f32,
    pub seats_sold: u32,
    pub capture_price: f32,
    pub captures_count: u32,
    pub gross_revenue: f32,
    pub total_capture_cost: f32,
    pub net_profit: f32,
    pub kamas_per_hour: f32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(default)]
pub struct AppData {
    pub zones: Vec<ZoneEntry>,
    pub dungeons: Vec<DungeonEntry>,
    pub duo_trios: Vec<DuoTrioEntry>,
    pub arenas: Vec<ArenaEntry>,
}

pub const PERSISTED_STATE_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedInlineEdit<T> {
    pub index: usize,
    pub form: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DraftState {
    pub zone_form: Option<ZoneForm>,
    pub dungeon_form: Option<DungeonForm>,
    pub duo_trio_form: Option<DuoTrioForm>,
    pub arena_form: Option<ArenaForm>,
    pub zone_edit: Option<PersistedInlineEdit<ZoneForm>>,
    pub dungeon_edit: Option<PersistedInlineEdit<DungeonForm>>,
    pub duo_trio_edit: Option<PersistedInlineEdit<DuoTrioForm>>,
    pub arena_edit: Option<PersistedInlineEdit<ArenaForm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedState {
    pub schema_version: u32,
    pub data: AppData,
    pub drafts: DraftState,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: PERSISTED_STATE_VERSION,
            data: AppData::default(),
            drafts: DraftState::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DurationInput {
    pub hours: String,
    pub minutes: String,
    pub seconds: String,
}

impl Default for DurationInput {
    fn default() -> Self {
        Self {
            hours: "00".to_string(),
            minutes: "00".to_string(),
            seconds: "00".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoneForm {
    pub name: String,
    pub character_class: Option<DofusClass>,
    pub recorded_at_input: String,
    pub session_time: DurationInput,
    pub session_total_kamas: String,
}

impl Default for ZoneForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            character_class: None,
            recorded_at_input: default_recorded_at_input(),
            session_time: DurationInput::default(),
            session_total_kamas: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DungeonForm {
    pub name: String,
    pub character_class: Option<DofusClass>,
    pub recorded_at_input: String,
    pub run_time: DurationInput,
    pub gross_kamas_per_run: String,
    pub key_price: String,
}

impl Default for DungeonForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            character_class: None,
            recorded_at_input: default_recorded_at_input(),
            run_time: DurationInput::default(),
            gross_kamas_per_run: String::new(),
            key_price: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DuoTrioForm {
    pub name: String,
    pub character_class: Option<DofusClass>,
    pub recorded_at_input: String,
    pub party_mode: PartyMode,
    pub run_time: DurationInput,
    pub loot_kamas_per_run: String,
    pub capture_stone_price: String,
    pub key_unit_price: String,
    pub full_soul_sale_price: String,
}

impl Default for DuoTrioForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            character_class: None,
            recorded_at_input: default_recorded_at_input(),
            party_mode: PartyMode::Duo,
            run_time: DurationInput::default(),
            loot_kamas_per_run: String::new(),
            capture_stone_price: String::new(),
            key_unit_price: String::new(),
            full_soul_sale_price: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ArenaForm {
    pub name: String,
    pub character_class: Option<DofusClass>,
    pub recorded_at_input: String,
    pub round_time: DurationInput,
    pub seat_price: String,
    pub seats_sold: String,
    pub capture_price: String,
    pub captures_count: String,
}

impl Default for ArenaForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            character_class: None,
            recorded_at_input: default_recorded_at_input(),
            round_time: DurationInput::default(),
            seat_price: String::new(),
            seats_sold: "7".to_string(),
            capture_price: String::new(),
            captures_count: "10".to_string(),
        }
    }
}

impl ZoneForm {
    pub fn has_user_input(&self) -> bool {
        !self.name.trim().is_empty()
            || self.character_class.is_some()
            || self.session_time != DurationInput::default()
            || !self.session_total_kamas.trim().is_empty()
    }
}

impl DungeonForm {
    pub fn has_user_input(&self) -> bool {
        !self.name.trim().is_empty()
            || self.character_class.is_some()
            || self.run_time != DurationInput::default()
            || !self.gross_kamas_per_run.trim().is_empty()
            || !self.key_price.trim().is_empty()
    }
}

impl DuoTrioForm {
    pub fn has_user_input(&self) -> bool {
        !self.name.trim().is_empty()
            || self.character_class.is_some()
            || self.party_mode != PartyMode::Duo
            || self.run_time != DurationInput::default()
            || !self.loot_kamas_per_run.trim().is_empty()
            || !self.capture_stone_price.trim().is_empty()
            || !self.key_unit_price.trim().is_empty()
            || !self.full_soul_sale_price.trim().is_empty()
    }
}

impl ArenaForm {
    pub fn has_user_input(&self) -> bool {
        !self.name.trim().is_empty()
            || self.character_class.is_some()
            || self.round_time != DurationInput::default()
            || !self.seat_price.trim().is_empty()
            || self.seats_sold.trim() != "7"
            || !self.capture_price.trim().is_empty()
            || self.captures_count.trim() != "10"
    }
}

impl DraftState {
    pub fn restored_items_count(&self) -> usize {
        let mut count = 0;

        if self.zone_form.is_some() {
            count += 1;
        }
        if self.dungeon_form.is_some() {
            count += 1;
        }
        if self.duo_trio_form.is_some() {
            count += 1;
        }
        if self.arena_form.is_some() {
            count += 1;
        }
        if self.zone_edit.is_some() {
            count += 1;
        }
        if self.dungeon_edit.is_some() {
            count += 1;
        }
        if self.duo_trio_edit.is_some() {
            count += 1;
        }
        if self.arena_edit.is_some() {
            count += 1;
        }

        count
    }

    pub fn has_any_draft(&self) -> bool {
        self.restored_items_count() > 0
    }
}

fn default_recorded_at_input() -> String {
    Local::now()
        .naive_local()
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn party_mode_serializes_and_deserializes_lowercase_values() {
        let duo = serde_json::from_str::<PartyMode>(r#""duo""#).unwrap();
        let trio = serde_json::from_str::<PartyMode>(r#""trio""#).unwrap();

        assert_eq!(duo, PartyMode::Duo);
        assert_eq!(trio, PartyMode::Trio);
        assert_eq!(serde_json::to_string(&PartyMode::Duo).unwrap(), r#""duo""#);
        assert_eq!(
            serde_json::to_string(&PartyMode::Trio).unwrap(),
            r#""trio""#
        );
    }

    #[test]
    fn party_mode_deserializes_legacy_capitalized_values() {
        let duo = serde_json::from_str::<PartyMode>(r#""Duo""#).unwrap();
        let trio = serde_json::from_str::<PartyMode>(r#""Trio""#).unwrap();

        assert_eq!(duo, PartyMode::Duo);
        assert_eq!(trio, PartyMode::Trio);
    }

    #[test]
    fn class_asset_file_names_are_ascii_and_match_expected_bundle_names() {
        let expected = [
            (DofusClass::Cra, "cra.png"),
            (DofusClass::Ecaflip, "ecaflip.png"),
            (DofusClass::Eniripsa, "eniripsa.png"),
            (DofusClass::Enutrof, "enutrof.png"),
            (DofusClass::Feca, "feca.png"),
            (DofusClass::Iop, "iop.png"),
            (DofusClass::Osamodas, "osamodas.png"),
            (DofusClass::Pandawa, "pandawa.png"),
            (DofusClass::Sacrieur, "sacrieur.png"),
            (DofusClass::Sadida, "sadida.png"),
            (DofusClass::Sram, "sram.png"),
            (DofusClass::Xelor, "xelor.png"),
        ];

        for (class, expected_file_name) in expected {
            assert_eq!(class.asset_file_name(), expected_file_name);
            assert!(class.asset_file_name().is_ascii());
        }
    }

    #[test]
    fn class_storage_keys_stay_unique() {
        let mut keys = DofusClass::all()
            .into_iter()
            .map(DofusClass::storage_key)
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();

        assert_eq!(keys.len(), DofusClass::all().len());
    }
}
