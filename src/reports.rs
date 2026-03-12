use crate::calculations::{
    arena_kamas_per_hour, dungeon_kamas_per_hour, duo_trio_kamas_per_hour,
    normalize_text_for_matching, zone_kamas_per_hour,
};
use crate::models::{AppData, DofusClass};
use chrono::{Duration, Local, NaiveDateTime};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ActivityKind {
    Zone,
    Dungeon,
    DuoTrio,
    Arena,
}

impl ActivityKind {
    pub const fn all() -> [Self; 4] {
        [Self::Zone, Self::Dungeon, Self::DuoTrio, Self::Arena]
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Zone => "Zones",
            Self::Dungeon => "Donjons",
            Self::DuoTrio => "Duo / Trio",
            Self::Arena => "PL arene",
        }
    }

    pub const fn singular_label(self) -> &'static str {
        match self {
            Self::Zone => "Zone",
            Self::Dungeon => "Donjon",
            Self::DuoTrio => "Duo / Trio",
            Self::Arena => "PL arene",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportPeriod {
    Last24Hours,
    Last7Days,
    Last30Days,
    AllTime,
}

impl ReportPeriod {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Last24Hours => "24h",
            Self::Last7Days => "7j",
            Self::Last30Days => "30j",
            Self::AllTime => "Depuis le debut",
        }
    }

    pub fn start_at(self, now: NaiveDateTime) -> Option<NaiveDateTime> {
        match self {
            Self::Last24Hours => Some(now - Duration::hours(24)),
            Self::Last7Days => Some(now - Duration::days(7)),
            Self::Last30Days => Some(now - Duration::days(30)),
            Self::AllTime => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportCategoryFilter {
    pub zones: bool,
    pub dungeons: bool,
    pub duo_trios: bool,
    pub arenas: bool,
}

impl Default for ReportCategoryFilter {
    fn default() -> Self {
        Self {
            zones: true,
            dungeons: true,
            duo_trios: true,
            arenas: true,
        }
    }
}

impl ReportCategoryFilter {
    pub fn is_selected(self, kind: ActivityKind) -> bool {
        match kind {
            ActivityKind::Zone => self.zones,
            ActivityKind::Dungeon => self.dungeons,
            ActivityKind::DuoTrio => self.duo_trios,
            ActivityKind::Arena => self.arenas,
        }
    }

    pub fn any_selected(self) -> bool {
        self.zones || self.dungeons || self.duo_trios || self.arenas
    }

    pub fn selected_kinds(self) -> Vec<ActivityKind> {
        let mut selected = Vec::new();

        if self.zones {
            selected.push(ActivityKind::Zone);
        }
        if self.dungeons {
            selected.push(ActivityKind::Dungeon);
        }
        if self.duo_trios {
            selected.push(ActivityKind::DuoTrio);
        }
        if self.arenas {
            selected.push(ActivityKind::Arena);
        }

        selected
    }

    pub fn summary_label(self) -> String {
        match self.selected_kinds().as_slice() {
            [] => "Aucune categorie".to_string(),
            [kind] => kind.label().to_string(),
            selected if selected.len() == 4 => "Toutes les categories".to_string(),
            selected => selected
                .iter()
                .map(|kind| kind.singular_label())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReportSession {
    pub kind: ActivityKind,
    pub name: String,
    pub normalized_name: String,
    pub class: Option<DofusClass>,
    pub recorded_at: Option<NaiveDateTime>,
    pub duration_seconds: f32,
    pub accounting_value: f32,
    pub kamas_per_hour: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ActivitySearchFilters {
    pub class: Option<DofusClass>,
    pub available_time_seconds: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ActivityRecommendation {
    pub kind: ActivityKind,
    pub name: String,
    pub class: Option<DofusClass>,
    pub session_count: usize,
    pub average_duration_seconds: f32,
    pub average_value_per_session: f32,
    pub average_kamas_per_hour: f32,
    pub estimated_runs: Option<u32>,
    pub estimated_total_value: Option<f32>,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct ActivityRecommendationGroup {
    pub kind: ActivityKind,
    pub items: Vec<ActivityRecommendation>,
}

#[derive(Debug, Clone)]
pub struct ChartSessionDetail {
    pub kind: ActivityKind,
    pub name: String,
    pub duration_seconds: f32,
    pub delta_value: f32,
    pub kamas_per_hour: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CumulativePoint {
    pub recorded_at: NaiveDateTime,
    pub delta_value: f32,
    pub cumulative_value: f32,
    pub sessions: Vec<ChartSessionDetail>,
}

#[derive(Debug, Clone)]
pub struct CategorySeries {
    pub kind: ActivityKind,
    pub points: Vec<CumulativePoint>,
    pub total_in_period: f32,
    pub session_count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SessionBarSegment {
    pub kind: ActivityKind,
    pub name: String,
    pub session_start_at: NaiveDateTime,
    pub session_end_at: NaiveDateTime,
    pub segment_start_at: NaiveDateTime,
    pub segment_end_at: NaiveDateTime,
    pub session_duration_seconds: f32,
    pub value: f32,
    pub kamas_per_hour: f32,
    pub base_offset: f32,
    pub stack_total_value: f32,
    pub simultaneous_sessions: Vec<ChartSessionDetail>,
}

#[derive(Debug, Clone)]
pub struct CategoryBarSeries {
    pub kind: ActivityKind,
    pub segments: Vec<SessionBarSegment>,
}

#[derive(Debug, Clone)]
pub struct ActivityAggregate {
    pub kind: ActivityKind,
    pub name: String,
    pub sessions: usize,
    pub total_value: f32,
    pub best_session: f32,
    pub best_class: Option<DofusClass>,
}

#[derive(Debug, Clone)]
pub struct ClassSummary {
    pub class: DofusClass,
    pub sessions: usize,
    pub total_value: f32,
    pub best: f32,
    pub best_kind: ActivityKind,
}

#[derive(Debug, Clone)]
pub struct ReportSummary {
    pub session_count: usize,
    pub total_earned: f32,
    pub average_per_session: Option<f32>,
    pub best_session: Option<ReportSession>,
    pub best_activity: Option<ActivityAggregate>,
    pub top_activities: Vec<ActivityAggregate>,
    pub class_summaries: Vec<ClassSummary>,
    pub recent_sessions: Vec<ReportSession>,
    pub chart_series: Vec<CategorySeries>,
    pub chart_bars: Vec<CategoryBarSeries>,
    pub period_started_at: Option<NaiveDateTime>,
}

pub fn local_now() -> NaiveDateTime {
    Local::now().naive_local()
}

pub fn build_report_summary(
    data: &AppData,
    period: ReportPeriod,
    categories: ReportCategoryFilter,
    now: NaiveDateTime,
) -> ReportSummary {
    let all_sessions = collect_sessions(data);
    let filtered_sessions: Vec<ReportSession> = all_sessions
        .into_iter()
        .filter(|session| categories.is_selected(session.kind))
        .filter(|session| matches_period(session.recorded_at, period, now))
        .collect();

    let session_count = filtered_sessions.len();
    let total_earned = filtered_sessions
        .iter()
        .map(|session| session.accounting_value)
        .sum::<f32>();
    let average_per_session = (session_count > 0).then_some(total_earned / session_count as f32);

    let best_session = filtered_sessions
        .iter()
        .cloned()
        .max_by(compare_sessions_by_value);

    let mut top_activities = aggregate_activities(&filtered_sessions);
    top_activities.sort_by(compare_activity_aggregates);

    let best_activity = top_activities.first().cloned();
    let mut class_summaries = summarize_classes(&filtered_sessions);
    class_summaries.sort_by(compare_class_summaries);

    let mut recent_sessions = filtered_sessions.clone();
    recent_sessions.sort_by(compare_recent_sessions);
    recent_sessions.truncate(12);

    ReportSummary {
        session_count,
        total_earned,
        average_per_session,
        best_session,
        best_activity,
        top_activities,
        class_summaries,
        recent_sessions,
        chart_series: build_category_series(&filtered_sessions, categories),
        chart_bars: build_category_bar_series(&filtered_sessions, categories),
        period_started_at: period.start_at(now),
    }
}

pub fn collect_sessions(data: &AppData) -> Vec<ReportSession> {
    let mut sessions = Vec::new();

    for entry in &data.zones {
        let kamas_per_hour = if entry.session_time_seconds > 0.0 {
            zone_kamas_per_hour(entry.session_total_kamas, entry.session_time_seconds)
        } else {
            entry.kamas_per_hour
        };

        sessions.push(ReportSession {
            kind: ActivityKind::Zone,
            name: entry.name.clone(),
            normalized_name: normalize_name(&entry.name),
            class: entry.character_class,
            recorded_at: entry.recorded_at,
            duration_seconds: entry.session_time_seconds,
            accounting_value: entry.session_total_kamas,
            kamas_per_hour,
        });
    }

    for entry in &data.dungeons {
        let accounting_value = entry.gross_kamas_per_run - entry.key_price;
        let kamas_per_hour = if entry.run_time_minutes > 0.0 {
            dungeon_kamas_per_hour(accounting_value, entry.run_time_minutes)
        } else {
            entry.kamas_per_hour
        };

        sessions.push(ReportSession {
            kind: ActivityKind::Dungeon,
            name: entry.name.clone(),
            normalized_name: normalize_name(&entry.name),
            class: entry.character_class,
            recorded_at: entry.recorded_at,
            duration_seconds: entry.run_time_minutes * 60.0,
            accounting_value,
            kamas_per_hour,
        });
    }

    for entry in &data.duo_trios {
        let total_key_cost = entry.key_unit_price * entry.party_mode.player_count() as f32;
        let gross_kamas_per_run = entry.loot_kamas_per_run + entry.full_soul_sale_price;
        let total_cost = entry.capture_stone_price + total_key_cost;
        let accounting_value = gross_kamas_per_run - total_cost;
        let kamas_per_hour = if entry.run_time_seconds > 0.0 {
            duo_trio_kamas_per_hour(accounting_value, entry.run_time_seconds)
        } else {
            entry.kamas_per_hour
        };

        sessions.push(ReportSession {
            kind: ActivityKind::DuoTrio,
            name: entry.name.clone(),
            normalized_name: normalize_name(&entry.name),
            class: entry.character_class,
            recorded_at: entry.recorded_at,
            duration_seconds: entry.run_time_seconds,
            accounting_value,
            kamas_per_hour,
        });
    }

    for entry in &data.arenas {
        let gross_revenue = entry.seat_price * entry.seats_sold as f32;
        let total_capture_cost = entry.capture_price * entry.captures_count as f32;
        let accounting_value = gross_revenue - total_capture_cost;
        let kamas_per_hour = if entry.round_time_minutes > 0.0 {
            arena_kamas_per_hour(accounting_value, entry.round_time_minutes)
        } else {
            entry.kamas_per_hour
        };

        sessions.push(ReportSession {
            kind: ActivityKind::Arena,
            name: entry.name.clone(),
            normalized_name: normalize_name(&entry.name),
            class: entry.character_class,
            recorded_at: entry.recorded_at,
            duration_seconds: entry.round_time_minutes * 60.0,
            accounting_value,
            kamas_per_hour,
        });
    }

    sessions
}

pub fn build_activity_recommendations(
    data: &AppData,
    filters: &ActivitySearchFilters,
) -> Vec<ActivityRecommendationGroup> {
    let available_time_seconds = filters.available_time_seconds.filter(|value| *value > 0.0);
    let variants = aggregate_activity_variants(&collect_sessions(data), filters.class);
    let variants = collapse_activity_variants(variants, available_time_seconds, filters.class);
    let mut grouped: HashMap<ActivityKind, Vec<ActivityRecommendation>> = HashMap::new();

    for variant in variants {
        let Some(recommendation) = recommendation_from_variant(variant, available_time_seconds)
        else {
            continue;
        };

        grouped
            .entry(recommendation.kind)
            .or_default()
            .push(recommendation);
    }

    ActivityKind::all()
        .into_iter()
        .map(|kind| {
            let mut items = grouped.remove(&kind).unwrap_or_default();
            items.sort_by(compare_recommendations_for_sort);
            items.truncate(3);

            ActivityRecommendationGroup { kind, items }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct ActivityVariantStats {
    kind: ActivityKind,
    name: String,
    normalized_name: String,
    class: Option<DofusClass>,
    session_count: usize,
    average_duration_seconds: f32,
    average_value_per_session: f32,
    average_kamas_per_hour: f32,
}

fn aggregate_activity_variants(
    sessions: &[ReportSession],
    class_filter: Option<DofusClass>,
) -> Vec<ActivityVariantStats> {
    struct VariantAccumulator {
        kind: ActivityKind,
        name: String,
        normalized_name: String,
        class: Option<DofusClass>,
        session_count: usize,
        total_duration_seconds: f32,
        total_value: f32,
    }

    let mut grouped: HashMap<(ActivityKind, String, Option<DofusClass>), VariantAccumulator> =
        HashMap::new();

    for session in sessions {
        if session.duration_seconds <= 0.0 {
            continue;
        }

        if let Some(class_filter) = class_filter {
            if session.class != Some(class_filter) {
                continue;
            }
        }

        let key = (session.kind, session.normalized_name.clone(), session.class);
        let entry = grouped.entry(key).or_insert_with(|| VariantAccumulator {
            kind: session.kind,
            name: session.name.clone(),
            normalized_name: session.normalized_name.clone(),
            class: session.class,
            session_count: 0,
            total_duration_seconds: 0.0,
            total_value: 0.0,
        });

        entry.session_count += 1;
        entry.total_duration_seconds += session.duration_seconds;
        entry.total_value += session.accounting_value;
    }

    grouped
        .into_values()
        .filter(|entry| entry.session_count > 0 && entry.total_duration_seconds > 0.0)
        .map(|entry| ActivityVariantStats {
            kind: entry.kind,
            name: entry.name,
            normalized_name: entry.normalized_name,
            class: entry.class,
            session_count: entry.session_count,
            average_duration_seconds: entry.total_duration_seconds / entry.session_count as f32,
            average_value_per_session: entry.total_value / entry.session_count as f32,
            average_kamas_per_hour: entry.total_value * 3600.0 / entry.total_duration_seconds,
        })
        .collect()
}

fn collapse_activity_variants(
    variants: Vec<ActivityVariantStats>,
    available_time_seconds: Option<f32>,
    class_filter: Option<DofusClass>,
) -> Vec<ActivityVariantStats> {
    if class_filter.is_some() {
        return variants;
    }

    let mut grouped: HashMap<(ActivityKind, String), Vec<ActivityVariantStats>> = HashMap::new();

    for variant in variants {
        grouped
            .entry((variant.kind, variant.normalized_name.clone()))
            .or_default()
            .push(variant);
    }

    grouped
        .into_values()
        .filter_map(|variants_for_activity| {
            let has_known_class = variants_for_activity
                .iter()
                .any(|variant| variant.class.is_some());

            let candidates = variants_for_activity
                .into_iter()
                .filter(|variant| !has_known_class || variant.class.is_some())
                .filter_map(|variant| {
                    recommendation_from_variant(variant.clone(), available_time_seconds)
                        .map(|recommendation| (variant, recommendation))
                })
                .collect::<Vec<_>>();

            candidates
                .into_iter()
                .max_by(|(_, left), (_, right)| compare_recommendations_for_choice(left, right))
                .map(|(variant, _)| variant)
        })
        .collect()
}

fn recommendation_from_variant(
    variant: ActivityVariantStats,
    available_time_seconds: Option<f32>,
) -> Option<ActivityRecommendation> {
    let (estimated_runs, estimated_total_value, score) = match available_time_seconds {
        Some(available_time_seconds) => {
            if variant.average_duration_seconds <= 0.0 {
                return None;
            }

            let estimated_runs =
                (available_time_seconds / variant.average_duration_seconds).floor() as u32;

            if estimated_runs == 0 {
                return None;
            }

            let estimated_total_value = estimated_runs as f32 * variant.average_value_per_session;
            (
                Some(estimated_runs),
                Some(estimated_total_value),
                estimated_total_value,
            )
        }
        None => (None, None, variant.average_kamas_per_hour),
    };

    Some(ActivityRecommendation {
        kind: variant.kind,
        name: variant.name,
        class: variant.class,
        session_count: variant.session_count,
        average_duration_seconds: variant.average_duration_seconds,
        average_value_per_session: variant.average_value_per_session,
        average_kamas_per_hour: variant.average_kamas_per_hour,
        estimated_runs,
        estimated_total_value,
        score,
    })
}

fn compare_recommendations_for_choice(
    left: &ActivityRecommendation,
    right: &ActivityRecommendation,
) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| {
            left.average_kamas_per_hour
                .total_cmp(&right.average_kamas_per_hour)
        })
        .then_with(|| left.session_count.cmp(&right.session_count))
        .then_with(|| right.name.cmp(&left.name))
        .then_with(|| left.class.cmp(&right.class))
}

fn compare_recommendations_for_sort(
    left: &ActivityRecommendation,
    right: &ActivityRecommendation,
) -> Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| {
            right
                .average_kamas_per_hour
                .total_cmp(&left.average_kamas_per_hour)
        })
        .then_with(|| right.session_count.cmp(&left.session_count))
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.class.cmp(&right.class))
}

fn matches_period(
    recorded_at: Option<NaiveDateTime>,
    period: ReportPeriod,
    now: NaiveDateTime,
) -> bool {
    let Some(recorded_at) = recorded_at else {
        return false;
    };

    match period.start_at(now) {
        Some(start_at) => recorded_at >= start_at && recorded_at <= now,
        None => recorded_at <= now,
    }
}

fn build_category_series(
    sessions: &[ReportSession],
    categories: ReportCategoryFilter,
) -> Vec<CategorySeries> {
    let mut grouped: HashMap<ActivityKind, BTreeMap<NaiveDateTime, Vec<&ReportSession>>> =
        HashMap::new();

    for session in sessions {
        let Some(recorded_at) = session.recorded_at else {
            continue;
        };

        grouped
            .entry(session.kind)
            .or_default()
            .entry(recorded_at)
            .or_default()
            .push(session);
    }

    let mut series = Vec::new();

    for kind in categories.selected_kinds() {
        let mut cumulative_value = 0.0;
        let mut points = Vec::new();
        let mut total_in_period = 0.0;
        let mut session_count = 0usize;

        if let Some(events) = grouped.get(&kind) {
            for (recorded_at, sessions_at_time) in events {
                let delta_value = sessions_at_time
                    .iter()
                    .map(|session| session.accounting_value)
                    .sum::<f32>();

                total_in_period += delta_value;
                session_count += sessions_at_time.len();
                cumulative_value += delta_value;

                let mut details = sessions_at_time
                    .iter()
                    .map(|session| ChartSessionDetail {
                        kind: session.kind,
                        name: session.name.clone(),
                        duration_seconds: session.duration_seconds,
                        delta_value: session.accounting_value,
                        kamas_per_hour: session.kamas_per_hour,
                    })
                    .collect::<Vec<_>>();
                details.sort_by(|left, right| {
                    right
                        .delta_value
                        .total_cmp(&left.delta_value)
                        .then_with(|| left.name.cmp(&right.name))
                });

                points.push(CumulativePoint {
                    recorded_at: *recorded_at,
                    delta_value,
                    cumulative_value,
                    sessions: details,
                });
            }
        }

        series.push(CategorySeries {
            kind,
            points,
            total_in_period,
            session_count,
        });
    }

    series
}

#[derive(Clone, Copy)]
struct RenderableBarSession<'a> {
    source_index: usize,
    session: &'a ReportSession,
    start_at: NaiveDateTime,
    end_at: NaiveDateTime,
}

fn build_category_bar_series(
    sessions: &[ReportSession],
    categories: ReportCategoryFilter,
) -> Vec<CategoryBarSeries> {
    let renderable_sessions = sessions
        .iter()
        .enumerate()
        .filter_map(|(source_index, session)| {
            let start_at = session.recorded_at?;
            let duration_seconds = session.duration_seconds.round() as i64;

            if duration_seconds <= 0 || session.accounting_value.abs() <= f32::EPSILON {
                return None;
            }

            Some(RenderableBarSession {
                source_index,
                session,
                start_at,
                end_at: start_at + Duration::seconds(duration_seconds),
            })
        })
        .collect::<Vec<_>>();

    let mut boundaries = renderable_sessions
        .iter()
        .flat_map(|session| [session.start_at, session.end_at])
        .collect::<Vec<_>>();
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut segments_by_kind: HashMap<ActivityKind, Vec<SessionBarSegment>> = HashMap::new();

    for window in boundaries.windows(2) {
        let segment_start_at = window[0];
        let segment_end_at = window[1];

        if segment_end_at <= segment_start_at {
            continue;
        }

        let mut active_sessions = renderable_sessions
            .iter()
            .copied()
            .filter(|session| {
                session.start_at <= segment_start_at && session.end_at > segment_start_at
            })
            .collect::<Vec<_>>();

        if active_sessions.is_empty() {
            continue;
        }

        active_sessions.sort_by(compare_renderable_bar_sessions_for_stack);

        let simultaneous_sessions = active_sessions
            .iter()
            .map(|session| ChartSessionDetail {
                kind: session.session.kind,
                name: session.session.name.clone(),
                duration_seconds: session.session.duration_seconds,
                delta_value: session.session.accounting_value,
                kamas_per_hour: session.session.kamas_per_hour,
            })
            .collect::<Vec<_>>();
        let stacked_positive_total = active_sessions
            .iter()
            .filter(|session| session.session.accounting_value > 0.0)
            .map(|session| session.session.accounting_value)
            .sum::<f32>();
        let stacked_negative_total = active_sessions
            .iter()
            .filter(|session| session.session.accounting_value < 0.0)
            .map(|session| session.session.accounting_value)
            .sum::<f32>();
        let mut positive_base = 0.0f32;
        let mut negative_base = 0.0f32;

        for session in active_sessions {
            let value = session.session.accounting_value;
            let base_offset = if value.is_sign_positive() {
                let base = positive_base;
                positive_base += value;
                base
            } else if value.is_sign_negative() {
                let base = negative_base;
                negative_base += value;
                base
            } else {
                0.0
            };

            segments_by_kind
                .entry(session.session.kind)
                .or_default()
                .push(SessionBarSegment {
                    kind: session.session.kind,
                    name: session.session.name.clone(),
                    session_start_at: session.start_at,
                    session_end_at: session.end_at,
                    segment_start_at,
                    segment_end_at,
                    session_duration_seconds: session.session.duration_seconds,
                    value,
                    kamas_per_hour: session.session.kamas_per_hour,
                    base_offset,
                    stack_total_value: if value.is_sign_negative() {
                        stacked_negative_total
                    } else {
                        stacked_positive_total
                    },
                    simultaneous_sessions: simultaneous_sessions.clone(),
                });
        }
    }

    let mut series = Vec::new();

    for kind in categories.selected_kinds() {
        series.push(CategoryBarSeries {
            kind,
            segments: segments_by_kind.remove(&kind).unwrap_or_default(),
        });
    }

    series
}

fn compare_renderable_bar_sessions_for_stack(
    left: &RenderableBarSession<'_>,
    right: &RenderableBarSession<'_>,
) -> Ordering {
    left.start_at
        .cmp(&right.start_at)
        .then_with(|| left.session.kind.cmp(&right.session.kind))
        .then_with(|| left.session.name.cmp(&right.session.name))
        .then_with(|| left.session.class.cmp(&right.session.class))
        .then_with(|| left.source_index.cmp(&right.source_index))
}

fn aggregate_activities(sessions: &[ReportSession]) -> Vec<ActivityAggregate> {
    #[derive(Default)]
    struct ActivityAccumulator {
        name: String,
        sessions: usize,
        total_value: f32,
        best_session: f32,
        class_stats: HashMap<DofusClass, ClassAccumulator>,
    }

    #[derive(Default)]
    struct ClassAccumulator {
        sessions: usize,
        total_value: f32,
        best: Option<f32>,
    }

    let mut grouped: HashMap<(ActivityKind, String), ActivityAccumulator> = HashMap::new();

    for session in sessions {
        let key = (session.kind, session.normalized_name.clone());
        let entry = grouped.entry(key).or_insert_with(|| ActivityAccumulator {
            name: session.name.clone(),
            best_session: session.accounting_value,
            ..Default::default()
        });

        entry.sessions += 1;
        entry.total_value += session.accounting_value;
        entry.best_session = entry.best_session.max(session.accounting_value);

        if let Some(class) = session.class {
            let class_entry = entry.class_stats.entry(class).or_default();
            class_entry.sessions += 1;
            class_entry.total_value += session.accounting_value;
            update_best_value(&mut class_entry.best, session.accounting_value);
        }
    }

    grouped
        .into_iter()
        .map(|((kind, _normalized_name), entry)| {
            let best_class = entry
                .class_stats
                .into_iter()
                .max_by(|(left_class, left), (right_class, right)| {
                    let left_average = left.total_value / left.sessions as f32;
                    let right_average = right.total_value / right.sessions as f32;

                    left_average
                        .total_cmp(&right_average)
                        .then_with(|| compare_optional_best(left.best, right.best))
                        .then_with(|| left_class.cmp(right_class))
                })
                .map(|(class, _)| class);

            ActivityAggregate {
                kind,
                name: entry.name,
                sessions: entry.sessions,
                total_value: entry.total_value,
                best_session: entry.best_session,
                best_class,
            }
        })
        .collect()
}

fn summarize_classes(sessions: &[ReportSession]) -> Vec<ClassSummary> {
    #[derive(Default)]
    struct KindAccumulator {
        total_value: f32,
        best: Option<f32>,
    }

    #[derive(Default)]
    struct ClassAccumulator {
        sessions: usize,
        total_value: f32,
        best: Option<f32>,
        by_kind: HashMap<ActivityKind, KindAccumulator>,
    }

    let mut grouped: HashMap<DofusClass, ClassAccumulator> = HashMap::new();

    for session in sessions {
        let Some(class) = session.class else {
            continue;
        };

        let entry = grouped.entry(class).or_default();
        entry.sessions += 1;
        entry.total_value += session.accounting_value;
        update_best_value(&mut entry.best, session.accounting_value);

        let kind_entry = entry.by_kind.entry(session.kind).or_default();
        kind_entry.total_value += session.accounting_value;
        update_best_value(&mut kind_entry.best, session.accounting_value);
    }

    grouped
        .into_iter()
        .map(|(class, entry)| {
            let best_kind = entry
                .by_kind
                .into_iter()
                .max_by(|(left_kind, left), (right_kind, right)| {
                    left.total_value
                        .total_cmp(&right.total_value)
                        .then_with(|| compare_optional_best(left.best, right.best))
                        .then_with(|| left_kind.cmp(right_kind))
                })
                .map(|(kind, _)| kind)
                .unwrap_or(ActivityKind::Zone);

            ClassSummary {
                class,
                sessions: entry.sessions,
                total_value: entry.total_value,
                best: entry.best.unwrap_or(0.0),
                best_kind,
            }
        })
        .collect()
}

fn update_best_value(current: &mut Option<f32>, candidate: f32) {
    *current = Some(current.map_or(candidate, |best| best.max(candidate)));
}

fn compare_optional_best(left: Option<f32>, right: Option<f32>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.total_cmp(&right),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

fn normalize_name(name: &str) -> String {
    normalize_text_for_matching(name)
}

fn compare_sessions_by_value(left: &ReportSession, right: &ReportSession) -> Ordering {
    left.accounting_value
        .total_cmp(&right.accounting_value)
        .then_with(|| left.recorded_at.cmp(&right.recorded_at))
        .then_with(|| right.name.cmp(&left.name))
}

fn compare_recent_sessions(left: &ReportSession, right: &ReportSession) -> Ordering {
    right
        .recorded_at
        .cmp(&left.recorded_at)
        .then_with(|| right.accounting_value.total_cmp(&left.accounting_value))
        .then_with(|| left.name.cmp(&right.name))
}

fn compare_activity_aggregates(left: &ActivityAggregate, right: &ActivityAggregate) -> Ordering {
    right
        .total_value
        .total_cmp(&left.total_value)
        .then_with(|| right.best_session.total_cmp(&left.best_session))
        .then_with(|| right.sessions.cmp(&left.sessions))
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.name.cmp(&right.name))
}

fn compare_class_summaries(left: &ClassSummary, right: &ClassSummary) -> Ordering {
    right
        .total_value
        .total_cmp(&left.total_value)
        .then_with(|| right.best.total_cmp(&left.best))
        .then_with(|| left.class.cmp(&right.class))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArenaEntry, DungeonEntry, DuoTrioEntry, PartyMode, ZoneEntry};

    fn dt(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn rolling_period_filters_are_exact_and_inclusive() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Blop".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-01 18:00")),
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 120_000.0,
                    kamas_per_hour: 240_000.0,
                },
                ZoneEntry {
                    name: "Blop".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-01 17:59")),
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 110_000.0,
                    kamas_per_hour: 220_000.0,
                },
            ],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::Last7Days,
            ReportCategoryFilter::default(),
            dt("2026-03-08 18:00"),
        );

        assert_eq!(summary.session_count, 1);
        assert_eq!(summary.total_earned, 120_000.0);
        assert_eq!(summary.best_session.unwrap().accounting_value, 120_000.0);
    }

    #[test]
    fn all_time_ignores_undated_entries() {
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Craqueleurs".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 1_800.0,
                session_total_kamas: 120_000.0,
                kamas_per_hour: 240_000.0,
            }],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter::default(),
            dt("2026-03-08 18:00"),
        );

        assert_eq!(summary.session_count, 0);
        assert!(summary
            .chart_series
            .iter()
            .all(|series| series.points.is_empty()));
        assert!(summary
            .chart_bars
            .iter()
            .all(|series| series.segments.is_empty()));
    }

    #[test]
    fn activity_aggregation_keeps_same_names_from_different_kinds_separate() {
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 18:00")),
                session_time_seconds: 1_800.0,
                session_total_kamas: 120_000.0,
                kamas_per_hour: 240_000.0,
            }],
            dungeons: vec![DungeonEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Feca),
                recorded_at: Some(dt("2026-03-08 18:05")),
                run_time_minutes: 20.0,
                gross_kamas_per_run: 150_000.0,
                key_price: 20_000.0,
                net_kamas_per_run: 130_000.0,
                kamas_per_hour: 390_000.0,
            }],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter::default(),
            dt("2026-03-08 19:00"),
        );

        assert_eq!(summary.top_activities.len(), 2);
        assert!(summary
            .top_activities
            .iter()
            .any(|activity| activity.kind == ActivityKind::Zone));
        assert!(summary
            .top_activities
            .iter()
            .any(|activity| activity.kind == ActivityKind::Dungeon));
    }

    #[test]
    fn class_summary_selects_best_kind_from_total_gains() {
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Plaine".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 10:00")),
                session_time_seconds: 1_800.0,
                session_total_kamas: 120_000.0,
                kamas_per_hour: 240_000.0,
            }],
            dungeons: vec![DungeonEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 13:00")),
                run_time_minutes: 20.0,
                gross_kamas_per_run: 150_000.0,
                key_price: 20_000.0,
                net_kamas_per_run: 130_000.0,
                kamas_per_hour: 390_000.0,
            }],
            duo_trios: vec![DuoTrioEntry {
                name: "Trio".to_string(),
                character_class: Some(DofusClass::Enutrof),
                recorded_at: Some(dt("2026-03-08 14:00")),
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
            arenas: vec![ArenaEntry {
                name: "Session".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 15:00")),
                round_time_minutes: 30.0,
                seat_price: 50_000.0,
                seats_sold: 7,
                capture_price: 120_000.0,
                captures_count: 10,
                gross_revenue: 350_000.0,
                total_capture_cost: 1_200_000.0,
                net_profit: -850_000.0,
                kamas_per_hour: -1_700_000.0,
            }],
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter::default(),
            dt("2026-03-08 18:00"),
        );
        let cra = summary
            .class_summaries
            .iter()
            .find(|item| item.class == DofusClass::Cra)
            .unwrap();

        assert_eq!(cra.best_kind, ActivityKind::Dungeon);
    }

    #[test]
    fn cumulative_series_restart_at_zero_and_merge_same_timestamp() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Zone A".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 09:30")),
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 100_000.0,
                    kamas_per_hour: 200_000.0,
                },
                ZoneEntry {
                    name: "Zone B".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 09:30")),
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 50_000.0,
                    kamas_per_hour: 100_000.0,
                },
                ZoneEntry {
                    name: "Zone C".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 12:00")),
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 80_000.0,
                    kamas_per_hour: 160_000.0,
                },
            ],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::Last24Hours,
            ReportCategoryFilter {
                zones: true,
                dungeons: false,
                duo_trios: false,
                arenas: false,
            },
            dt("2026-03-08 18:00"),
        );

        let series = &summary.chart_series[0];
        assert_eq!(series.kind, ActivityKind::Zone);
        assert_eq!(series.session_count, 3);
        assert_eq!(series.total_in_period, 230_000.0);
        assert_eq!(series.points.len(), 2);
        assert_eq!(series.points[0].delta_value, 150_000.0);
        assert_eq!(series.points[0].cumulative_value, 150_000.0);
        assert_eq!(series.points[0].sessions.len(), 2);
        assert_eq!(series.points[1].cumulative_value, 230_000.0);
    }

    #[test]
    fn bar_series_keeps_session_width_without_overlap() {
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Zone solo".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 09:00")),
                session_time_seconds: 3_600.0,
                session_total_kamas: 125_000.0,
                kamas_per_hour: 125_000.0,
            }],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter {
                zones: true,
                dungeons: false,
                duo_trios: false,
                arenas: false,
            },
            dt("2026-03-08 12:00"),
        );

        let series = &summary.chart_bars[0];
        assert_eq!(series.kind, ActivityKind::Zone);
        assert_eq!(series.segments.len(), 1);
        assert_eq!(series.segments[0].session_start_at, dt("2026-03-08 09:00"));
        assert_eq!(series.segments[0].session_end_at, dt("2026-03-08 10:00"));
        assert_eq!(series.segments[0].segment_start_at, dt("2026-03-08 09:00"));
        assert_eq!(series.segments[0].segment_end_at, dt("2026-03-08 10:00"));
        assert_eq!(series.segments[0].base_offset, 0.0);
        assert_eq!(series.segments[0].value, 125_000.0);
        assert_eq!(series.segments[0].stack_total_value, 125_000.0);
        assert_eq!(series.segments[0].simultaneous_sessions.len(), 1);
    }

    #[test]
    fn bar_series_splits_and_stacks_overlapping_sessions() {
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Zone A".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 09:00")),
                session_time_seconds: 3_600.0,
                session_total_kamas: 100_000.0,
                kamas_per_hour: 100_000.0,
            }],
            dungeons: vec![DungeonEntry {
                name: "Donjon B".to_string(),
                character_class: Some(DofusClass::Feca),
                recorded_at: Some(dt("2026-03-08 09:30")),
                run_time_minutes: 60.0,
                gross_kamas_per_run: 120_000.0,
                key_price: 40_000.0,
                net_kamas_per_run: 80_000.0,
                kamas_per_hour: 80_000.0,
            }],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter {
                zones: true,
                dungeons: true,
                duo_trios: false,
                arenas: false,
            },
            dt("2026-03-08 12:00"),
        );

        let zone_series = summary
            .chart_bars
            .iter()
            .find(|series| series.kind == ActivityKind::Zone)
            .unwrap();
        let dungeon_series = summary
            .chart_bars
            .iter()
            .find(|series| series.kind == ActivityKind::Dungeon)
            .unwrap();

        assert_eq!(zone_series.segments.len(), 2);
        assert_eq!(
            zone_series.segments[0].segment_start_at,
            dt("2026-03-08 09:00")
        );
        assert_eq!(
            zone_series.segments[0].segment_end_at,
            dt("2026-03-08 09:30")
        );
        assert_eq!(zone_series.segments[0].base_offset, 0.0);
        assert_eq!(zone_series.segments[0].stack_total_value, 100_000.0);
        assert_eq!(
            zone_series.segments[1].segment_start_at,
            dt("2026-03-08 09:30")
        );
        assert_eq!(
            zone_series.segments[1].segment_end_at,
            dt("2026-03-08 10:00")
        );
        assert_eq!(zone_series.segments[1].base_offset, 0.0);
        assert_eq!(zone_series.segments[1].stack_total_value, 180_000.0);
        assert_eq!(zone_series.segments[1].simultaneous_sessions.len(), 2);

        assert_eq!(dungeon_series.segments.len(), 2);
        assert_eq!(
            dungeon_series.segments[0].segment_start_at,
            dt("2026-03-08 09:30")
        );
        assert_eq!(
            dungeon_series.segments[0].segment_end_at,
            dt("2026-03-08 10:00")
        );
        assert_eq!(dungeon_series.segments[0].base_offset, 100_000.0);
        assert_eq!(dungeon_series.segments[0].stack_total_value, 180_000.0);
        assert_eq!(
            dungeon_series.segments[1].segment_start_at,
            dt("2026-03-08 10:00")
        );
        assert_eq!(
            dungeon_series.segments[1].segment_end_at,
            dt("2026-03-08 10:30")
        );
        assert_eq!(dungeon_series.segments[1].base_offset, 0.0);
        assert_eq!(dungeon_series.segments[1].stack_total_value, 80_000.0);
        assert_eq!(dungeon_series.segments[1].simultaneous_sessions.len(), 1);
    }

    #[test]
    fn activity_search_returns_top_three_per_category() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Zone A".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 180_000.0,
                    kamas_per_hour: 360_000.0,
                },
                ZoneEntry {
                    name: "Zone B".to_string(),
                    character_class: Some(DofusClass::Enutrof),
                    recorded_at: None,
                    session_time_seconds: 2_400.0,
                    session_total_kamas: 150_000.0,
                    kamas_per_hour: 225_000.0,
                },
                ZoneEntry {
                    name: "Zone C".to_string(),
                    character_class: Some(DofusClass::Feca),
                    recorded_at: None,
                    session_time_seconds: 1_500.0,
                    session_total_kamas: 120_000.0,
                    kamas_per_hour: 288_000.0,
                },
                ZoneEntry {
                    name: "Zone D".to_string(),
                    character_class: Some(DofusClass::Sadida),
                    recorded_at: None,
                    session_time_seconds: 1_200.0,
                    session_total_kamas: 80_000.0,
                    kamas_per_hour: 240_000.0,
                },
            ],
            dungeons: vec![DungeonEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                run_time_minutes: 20.0,
                gross_kamas_per_run: 150_000.0,
                key_price: 20_000.0,
                net_kamas_per_run: 130_000.0,
                kamas_per_hour: 390_000.0,
            }],
            duo_trios: vec![DuoTrioEntry {
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
            }],
            arenas: vec![ArenaEntry {
                name: "Bworker".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                round_time_minutes: 30.0,
                seat_price: 50_000.0,
                seats_sold: 7,
                capture_price: 120_000.0,
                captures_count: 10,
                gross_revenue: 350_000.0,
                total_capture_cost: 1_200_000.0,
                net_profit: -850_000.0,
                kamas_per_hour: -1_700_000.0,
            }],
        };

        let groups = build_activity_recommendations(&data, &ActivitySearchFilters::default());

        assert_eq!(groups.len(), 4);
        assert_eq!(groups[0].kind, ActivityKind::Zone);
        assert_eq!(groups[0].items.len(), 3);
        assert_eq!(groups[1].kind, ActivityKind::Dungeon);
        assert_eq!(groups[1].items.len(), 1);
        assert_eq!(groups[2].kind, ActivityKind::DuoTrio);
        assert_eq!(groups[2].items.len(), 1);
        assert_eq!(groups[3].kind, ActivityKind::Arena);
        assert_eq!(groups[3].items.len(), 1);
    }

    #[test]
    fn activity_search_class_filter_keeps_only_matching_class() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Plaine".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 150_000.0,
                    kamas_per_hour: 300_000.0,
                },
                ZoneEntry {
                    name: "Plaine".to_string(),
                    character_class: Some(DofusClass::Sadida),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 120_000.0,
                    kamas_per_hour: 240_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(
            &data,
            &ActivitySearchFilters {
                class: Some(DofusClass::Cra),
                available_time_seconds: None,
            },
        );

        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[0].items[0].class, Some(DofusClass::Cra));
    }

    #[test]
    fn activity_search_without_time_sorts_by_average_kamas_per_hour() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Forteresse".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 200_000.0,
                    kamas_per_hour: 400_000.0,
                },
                ZoneEntry {
                    name: "Mine".to_string(),
                    character_class: Some(DofusClass::Enutrof),
                    recorded_at: None,
                    session_time_seconds: 900.0,
                    session_total_kamas: 90_000.0,
                    kamas_per_hour: 360_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(&data, &ActivitySearchFilters::default());

        assert_eq!(groups[0].items[0].name, "Forteresse");
        assert_eq!(groups[0].items[1].name, "Mine");
    }

    #[test]
    fn activity_search_with_time_sorts_by_realizable_gain_and_can_change_order() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Forteresse".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 200_000.0,
                    kamas_per_hour: 400_000.0,
                },
                ZoneEntry {
                    name: "Mine".to_string(),
                    character_class: Some(DofusClass::Enutrof),
                    recorded_at: None,
                    session_time_seconds: 900.0,
                    session_total_kamas: 90_000.0,
                    kamas_per_hour: 360_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(
            &data,
            &ActivitySearchFilters {
                class: None,
                available_time_seconds: Some(2_700.0),
            },
        );

        assert_eq!(groups[0].items[0].name, "Mine");
        assert_eq!(groups[0].items[0].estimated_runs, Some(3));
        assert_eq!(groups[0].items[0].estimated_total_value, Some(270_000.0));
        assert_eq!(groups[0].items[1].name, "Forteresse");
    }

    #[test]
    fn activity_search_excludes_activities_that_do_not_fit_time_budget() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Rapide".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 900.0,
                    session_total_kamas: 80_000.0,
                    kamas_per_hour: 320_000.0,
                },
                ZoneEntry {
                    name: "Trop long".to_string(),
                    character_class: Some(DofusClass::Feca),
                    recorded_at: None,
                    session_time_seconds: 3_600.0,
                    session_total_kamas: 300_000.0,
                    kamas_per_hour: 300_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(
            &data,
            &ActivitySearchFilters {
                class: None,
                available_time_seconds: Some(1_000.0),
            },
        );

        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[0].items[0].name, "Rapide");
    }

    #[test]
    fn activity_search_without_class_filter_keeps_best_known_class_per_activity() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Plaine".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 150_000.0,
                    kamas_per_hour: 300_000.0,
                },
                ZoneEntry {
                    name: "plaine".to_string(),
                    character_class: Some(DofusClass::Sadida),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 120_000.0,
                    kamas_per_hour: 240_000.0,
                },
                ZoneEntry {
                    name: "Plaine".to_string(),
                    character_class: None,
                    recorded_at: None,
                    session_time_seconds: 1_500.0,
                    session_total_kamas: 150_000.0,
                    kamas_per_hour: 360_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(&data, &ActivitySearchFilters::default());

        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[0].items[0].name, "Plaine");
        assert_eq!(groups[0].items[0].class, Some(DofusClass::Cra));
    }

    #[test]
    fn collect_sessions_recomputes_stale_derived_values_for_all_kinds() {
        let data = AppData {
            zones: vec![ZoneEntry {
                name: "Zone".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: Some(dt("2026-03-08 10:00")),
                session_time_seconds: 3_600.0,
                session_total_kamas: 120_000.0,
                kamas_per_hour: 1.0,
            }],
            dungeons: vec![DungeonEntry {
                name: "Donjon".to_string(),
                character_class: Some(DofusClass::Feca),
                recorded_at: Some(dt("2026-03-08 11:00")),
                run_time_minutes: 20.0,
                gross_kamas_per_run: 150_000.0,
                key_price: 30_000.0,
                net_kamas_per_run: 1.0,
                kamas_per_hour: 1.0,
            }],
            duo_trios: vec![DuoTrioEntry {
                name: "Duo".to_string(),
                character_class: Some(DofusClass::Enutrof),
                recorded_at: Some(dt("2026-03-08 12:00")),
                party_mode: PartyMode::Duo,
                run_time_seconds: 3_600.0,
                loot_kamas_per_run: 200_000.0,
                capture_stone_price: 20_000.0,
                key_unit_price: 15_000.0,
                keys_count: 99,
                total_key_cost: 999_999.0,
                full_soul_sale_price: 100_000.0,
                gross_kamas_per_run: 1.0,
                total_cost: 1.0,
                net_kamas_per_run: 1.0,
                kamas_per_hour: 1.0,
            }],
            arenas: vec![ArenaEntry {
                name: "Arena".to_string(),
                character_class: Some(DofusClass::Iop),
                recorded_at: Some(dt("2026-03-08 13:00")),
                round_time_minutes: 30.0,
                seat_price: 50_000.0,
                seats_sold: 7,
                capture_price: 80_000.0,
                captures_count: 9,
                gross_revenue: 1.0,
                total_capture_cost: 1.0,
                net_profit: 1.0,
                kamas_per_hour: 1.0,
            }],
        };

        let sessions = collect_sessions(&data);

        assert_eq!(sessions[0].accounting_value, 120_000.0);
        assert_eq!(sessions[0].kamas_per_hour, 120_000.0);
        assert_eq!(sessions[1].accounting_value, 120_000.0);
        assert_eq!(sessions[1].kamas_per_hour, 360_000.0);
        assert_eq!(sessions[2].accounting_value, 250_000.0);
        assert_eq!(sessions[2].kamas_per_hour, 250_000.0);
        assert_eq!(sessions[3].accounting_value, -370_000.0);
        assert_eq!(sessions[3].kamas_per_hour, -740_000.0);
    }

    #[test]
    fn activity_search_groups_accentless_variants_together() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Cimetière".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 1_800.0,
                    session_total_kamas: 150_000.0,
                    kamas_per_hour: 300_000.0,
                },
                ZoneEntry {
                    name: "Cimetiere".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: None,
                    session_time_seconds: 3_600.0,
                    session_total_kamas: 180_000.0,
                    kamas_per_hour: 180_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(
            &data,
            &ActivitySearchFilters {
                class: Some(DofusClass::Cra),
                available_time_seconds: None,
            },
        );

        let item = groups[0]
            .items
            .iter()
            .find(|item| item.name == "Cimetière")
            .unwrap();

        assert_eq!(item.session_count, 2);
        assert_eq!(item.class, Some(DofusClass::Cra));
    }

    #[test]
    fn best_activity_prefers_least_negative_total() {
        let data = AppData {
            arenas: vec![
                ArenaEntry {
                    name: "Perte A".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 10:00")),
                    round_time_minutes: 30.0,
                    seat_price: 50_000.0,
                    seats_sold: 7,
                    capture_price: 90_000.0,
                    captures_count: 10,
                    gross_revenue: 1.0,
                    total_capture_cost: 1.0,
                    net_profit: -550_000.0,
                    kamas_per_hour: -1_100_000.0,
                },
                ArenaEntry {
                    name: "Perte B".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 11:00")),
                    round_time_minutes: 30.0,
                    seat_price: 50_000.0,
                    seats_sold: 7,
                    capture_price: 80_000.0,
                    captures_count: 8,
                    gross_revenue: 1.0,
                    total_capture_cost: 1.0,
                    net_profit: -290_000.0,
                    kamas_per_hour: -580_000.0,
                },
            ],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter::default(),
            dt("2026-03-08 18:00"),
        );

        assert_eq!(summary.best_activity.unwrap().name, "Perte B");
    }

    #[test]
    fn recommendations_use_weighted_kamas_per_hour() {
        let data = AppData {
            zones: vec![
                ZoneEntry {
                    name: "Blop".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 10:00")),
                    session_time_seconds: 300.0,
                    session_total_kamas: 100_000.0,
                    kamas_per_hour: 1_200_000.0,
                },
                ZoneEntry {
                    name: "Blop".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 11:00")),
                    session_time_seconds: 3_600.0,
                    session_total_kamas: 100_000.0,
                    kamas_per_hour: 100_000.0,
                },
            ],
            ..AppData::default()
        };

        let groups = build_activity_recommendations(
            &data,
            &ActivitySearchFilters {
                class: Some(DofusClass::Cra),
                available_time_seconds: None,
            },
        );

        let item = groups
            .iter()
            .find(|group| group.kind == ActivityKind::Zone)
            .and_then(|group| group.items.iter().find(|item| item.name == "Blop"))
            .unwrap();

        assert!((item.average_kamas_per_hour - 184_615.39).abs() < 0.1);
    }

    #[test]
    fn class_summary_preserves_negative_best_value() {
        let data = AppData {
            arenas: vec![
                ArenaEntry {
                    name: "Bworker".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 10:00")),
                    round_time_minutes: 30.0,
                    seat_price: 50_000.0,
                    seats_sold: 7,
                    capture_price: 85_000.0,
                    captures_count: 9,
                    gross_revenue: 1.0,
                    total_capture_cost: 1.0,
                    net_profit: -415_000.0,
                    kamas_per_hour: -830_000.0,
                },
                ArenaEntry {
                    name: "Bworker".to_string(),
                    character_class: Some(DofusClass::Cra),
                    recorded_at: Some(dt("2026-03-08 12:00")),
                    round_time_minutes: 30.0,
                    seat_price: 50_000.0,
                    seats_sold: 7,
                    capture_price: 50_000.0,
                    captures_count: 9,
                    gross_revenue: 1.0,
                    total_capture_cost: 1.0,
                    net_profit: -100_000.0,
                    kamas_per_hour: -200_000.0,
                },
            ],
            ..AppData::default()
        };

        let summary = build_report_summary(
            &data,
            ReportPeriod::AllTime,
            ReportCategoryFilter::default(),
            dt("2026-03-08 18:00"),
        );

        let cra = summary
            .class_summaries
            .iter()
            .find(|item| item.class == DofusClass::Cra)
            .unwrap();

        assert_eq!(cra.best, -100_000.0);
    }
}
