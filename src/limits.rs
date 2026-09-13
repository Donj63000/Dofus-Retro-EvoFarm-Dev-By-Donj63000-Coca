//! Contrat commun des entrées, du stockage et des graphiques.
//!
//! Ces plafonds sont des budgets techniques, pas des règles du jeu. Une valeur
//! hors contrat est refusée explicitement : aucune session n'est tronquée à l'import.

pub const MAX_STATE_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_TOTAL_ENTRIES: usize = 10_000;
pub const MAX_NAME_BYTES: usize = 512;
pub const MAX_DRAFT_FIELD_BYTES: usize = 4_096;
pub const MAX_NUMBER_INPUT_BYTES: usize = 128;
pub const MAX_DURATION_SECONDS: u32 = 24 * 3600 + 59 * 60 + 59;
pub const MAX_INPUT_KAMAS: f32 = 1_000_000_000_000.0;
pub const MAX_QUANTITY: u32 = 100_000;
// Avec les bornes ci-dessus, un taux horaire reste inférieur à ce plafond.
// Même 10 000 taux additionnés restent très loin de la limite d'un f32.
pub const MAX_CALCULATED_VALUE: f32 = 1.0e21;
pub const MAX_EXACT_BAR_SESSIONS: usize = 128;
pub const MAX_BAR_SEGMENTS: usize = 8_192;
pub const AGGREGATED_BAR_BUCKETS: usize = 256;
pub const MAX_TOOLTIP_SESSIONS: usize = 16;
pub const MAX_NAMED_SAVES: usize = 128;
pub const MAX_SAVE_DIRECTORY_ENTRIES: usize = 1_024;
pub const MAX_SAVE_LIST_BYTES: u64 = 64 * 1024 * 1024;
