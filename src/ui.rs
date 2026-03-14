use crate::calculations::{
    arena_kamas_per_hour, dungeon_kamas_per_hour, duo_trio_kamas_per_hour, format_duration_hms,
    format_kamas, format_recorded_at, normalize_text_for_matching, parse_duration_input, parse_f32,
    parse_u32, seconds_to_minutes, zone_kamas_per_hour,
};
use crate::models::{
    ArenaEntry, ArenaForm, DofusClass, DungeonEntry, DungeonForm, DuoTrioEntry, DuoTrioForm,
    DurationInput, PartyMode, Tab, ZoneEntry, ZoneForm,
};
use crate::reports::{
    build_activity_recommendations, build_report_summary, local_now, ActivityKind,
    ActivityRecommendation, ActivityRecommendationGroup, ActivitySearchFilters, CategoryBarSeries,
    CategorySeries, ReportPeriod, ReportSummary, SessionBarSegment,
};
use crate::theme;
use crate::{named_save_summary, persisted_state_summary, MyApp, StatusBanner, StatusKind, APP_NAME};
use chrono::{Local, NaiveDateTime};
use eframe::egui;
use eframe::egui::TextureHandle;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoint, PlotPoints, Points};
use std::collections::HashMap;

const REPORT_CHART_HEIGHT: f32 = 280.0;
const REPORT_TABLE_LIMIT: usize = 10;
const DURATION_COMBO_WIDTH: f32 = 78.0;
const DURATION_HOURS_MAX: u32 = 24;
const DURATION_SEGMENT_MAX: u32 = 59;
const SECTION_GAP: f32 = 14.0;
const FORM_PANEL_MIN_WIDTH: f32 = 420.0;
const LIST_PANEL_MIN_WIDTH: f32 = 920.0;
const REPORT_PANEL_MIN_WIDTH: f32 = 580.0;
const ACTIVITY_FILTER_PANEL_MIN_WIDTH: f32 = 360.0;
const ACTIVITY_RESULTS_PANEL_MIN_WIDTH: f32 = 640.0;
const ACTIVITY_RESULT_GROUP_MIN_WIDTH: f32 = 600.0;
const REPORT_SERIES_MIN_WIDTH: f32 = 220.0;
const REPORT_SERIES_MAX_WIDTH: f32 = 320.0;
const REPORT_KPI_MIN_WIDTH: f32 = 260.0;
const REPORT_KPI_MAX_COLUMNS: usize = 3;
const CARD_HEADER_STACK_BREAKPOINT: f32 = 720.0;
const CARD_COMPACT_METRICS_BREAKPOINT: f32 = 980.0;
const ACTIVITY_CARD_COMPACT_BREAKPOINT: f32 = 1_120.0;
const HEADER_LOGO_SIZE: f32 = 52.0;
const HEADER_BRAND_TITLE_SIZE: f32 = 30.0;
const HEADER_BRAND_ANIMATION_SPEED: f32 = 0.42;
const HEADER_BRAND_CHARACTER_OFFSET: f32 = 0.14;

#[derive(Default)]
struct DurationInputResponse {
    changed: bool,
    has_focus: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KpiSummary {
    pub count: usize,
    pub best: Option<f32>,
    pub average: Option<f32>,
}

#[derive(Clone)]
struct PreviewBlock {
    value: String,
    detail: String,
    tone: PreviewTone,
}

#[derive(Clone, Copy)]
enum PreviewTone {
    Accent,
    Positive,
    Danger,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ZoneListAction {
    StartEdit(usize),
    RequestDelete(usize),
    CancelDelete(usize),
    ConfirmDelete(usize),
    SaveEdit,
    CancelEdit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DungeonListAction {
    StartEdit(usize),
    RequestDelete(usize),
    CancelDelete(usize),
    ConfirmDelete(usize),
    SaveEdit,
    CancelEdit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DuoTrioListAction {
    StartEdit(usize),
    RequestDelete(usize),
    CancelDelete(usize),
    ConfirmDelete(usize),
    SaveEdit,
    CancelEdit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArenaListAction {
    StartEdit(usize),
    RequestDelete(usize),
    CancelDelete(usize),
    ConfirmDelete(usize),
    SaveEdit,
    CancelEdit,
}

pub fn summarize_values(values: impl Iterator<Item = f32>) -> KpiSummary {
    let mut count = 0usize;
    let mut total = 0.0f32;
    let mut best: Option<f32> = None;

    for value in values {
        count += 1;
        total += value;
        best = Some(best.map_or(value, |current| current.max(value)));
    }

    KpiSummary {
        count,
        best,
        average: (count > 0).then_some(total / count as f32),
    }
}

pub fn matches_search(name: &str, query: &str) -> bool {
    let normalized_query = normalize_text_for_matching(query);

    normalized_query.is_empty() || normalize_text_for_matching(name).contains(&normalized_query)
}

fn styled_tab_button(ui: &mut egui::Ui, current: &mut Tab, tab: Tab, label: &str) {
    let colors = theme::palette();
    let selected = *current == tab;
    let fill = if selected {
        colors.surface_alt
    } else {
        colors.surface
    };
    let stroke = if selected {
        colors.accent
    } else {
        colors.border
    };
    let text_color = if selected {
        colors.accent
    } else {
        colors.text_secondary
    };

    let button = egui::Button::new(
        egui::RichText::new(label)
            .size(14.0)
            .strong()
            .color(text_color),
    )
    .fill(fill)
    .stroke(egui::Stroke::new(1.0, stroke))
    .rounding(10.0);

    if ui.add(button).clicked() {
        *current = tab;
    }
}

fn toggle_chip_button<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    value: T,
    label: &str,
) {
    let colors = theme::palette();
    let selected = *current == value;
    let fill = if selected {
        colors.accent_soft
    } else {
        colors.surface_alt
    };
    let stroke = if selected {
        colors.accent
    } else {
        colors.border
    };
    let text_color = if selected {
        colors.accent
    } else {
        colors.text_primary
    };

    let button = egui::Button::new(
        egui::RichText::new(label)
            .size(12.0)
            .strong()
            .color(text_color),
    )
    .fill(fill)
    .stroke(egui::Stroke::new(1.0, stroke))
    .rounding(999.0);

    if ui.add(button).clicked() {
        *current = value;
    }
}

fn toggle_multi_chip_button(ui: &mut egui::Ui, selected: &mut bool, label: &str) {
    let colors = theme::palette();
    let fill = if *selected {
        colors.accent_soft
    } else {
        colors.surface_alt
    };
    let stroke = if *selected {
        colors.accent
    } else {
        colors.border
    };
    let text_color = if *selected {
        colors.accent
    } else {
        colors.text_primary
    };

    let button = egui::Button::new(
        egui::RichText::new(label)
            .size(12.0)
            .strong()
            .color(text_color),
    )
    .fill(fill)
    .stroke(egui::Stroke::new(1.0, stroke))
    .rounding(999.0);

    if ui.add(button).clicked() {
        *selected = !*selected;
    }
}

fn clamped_content(ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        ui.set_max_width(available_width);
        add_content(ui);
    });
}

fn clamped_wrapped_row(ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui)) {
    clamped_content(ui, |ui| {
        ui.horizontal_wrapped(|ui| add_content(ui));
    });
}

fn can_fit_two_columns(total_width: f32, left_min_width: f32, right_min_width: f32) -> bool {
    total_width >= left_min_width + right_min_width + SECTION_GAP
}

fn split_two_column_widths(
    total_width: f32,
    left_ratio: f32,
    left_min_width: f32,
    right_min_width: f32,
) -> (f32, f32) {
    let available_width = (total_width - SECTION_GAP).max(left_min_width + right_min_width);
    let max_left_width = (available_width - right_min_width).max(left_min_width);
    let left_width = (available_width * left_ratio).clamp(left_min_width, max_left_width);
    let right_width = (available_width - left_width).max(right_min_width);

    (left_width, right_width)
}

fn responsive_columns(
    total_width: f32,
    min_item_width: f32,
    gap: f32,
    max_columns: usize,
) -> usize {
    let mut columns = max_columns.max(1);

    while columns > 1 {
        let row_width = total_width - gap * (columns.saturating_sub(1)) as f32;
        if row_width / columns as f32 >= min_item_width {
            break;
        }
        columns -= 1;
    }

    columns.max(1)
}

fn grid_item_width(total_width: f32, columns: usize, gap: f32) -> f32 {
    let gaps_width = gap * columns.saturating_sub(1) as f32;
    ((total_width - gaps_width) / columns.max(1) as f32).max(0.0)
}

fn activity_result_columns(total_width: f32) -> usize {
    responsive_columns(total_width, ACTIVITY_RESULT_GROUP_MIN_WIDTH, SECTION_GAP, 2)
}

fn kpi_card(ui: &mut egui::Ui, label: &str, value: &str, tone: PreviewTone) {
    let colors = theme::palette();
    let (fill, stroke, text) = tone_colors(tone);

    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .rounding(12.0)
        .inner_margin(egui::Margin::symmetric(12.0, 9.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(11.0)
                        .color(colors.text_secondary),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(value)
                        .monospace()
                        .size(16.0)
                        .strong()
                        .color(text),
                );
            });
        });
}

fn metric_badge(ui: &mut egui::Ui, label: &str, value: String) {
    let colors = theme::palette();

    egui::Frame::none()
        .fill(colors.surface)
        .stroke(egui::Stroke::new(1.0, colors.border))
        .rounding(999.0)
        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(11.0)
                        .color(colors.text_secondary),
                );
                ui.label(
                    egui::RichText::new(value)
                        .monospace()
                        .size(12.0)
                        .color(colors.text_primary),
                );
            });
        });
}

fn compact_metric_badge(ui: &mut egui::Ui, label: &str, value: String) {
    let colors = theme::palette();

    egui::Frame::none()
        .fill(colors.surface)
        .stroke(egui::Stroke::new(1.0, colors.border))
        .rounding(999.0)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(10.0)
                        .color(colors.text_secondary),
                );
                ui.label(
                    egui::RichText::new(value)
                        .monospace()
                        .size(11.0)
                        .color(colors.text_primary),
                );
            });
        });
}

fn render_metric_row(ui: &mut egui::Ui, compact: bool, metrics: Vec<(&str, String)>) {
    clamped_wrapped_row(ui, |ui| {
        for (label, value) in metrics {
            if compact {
                compact_metric_badge(ui, label, value);
            } else {
                metric_badge(ui, label, value);
            }
        }
    });
}

fn render_chart_series_footer(ui: &mut egui::Ui, chart_series: &[CategorySeries]) {
    if chart_series.is_empty() {
        return;
    }

    let max_columns = chart_series.len().min(4);
    let columns = responsive_columns(
        ui.available_width(),
        REPORT_SERIES_MIN_WIDTH,
        SECTION_GAP,
        max_columns,
    );
    let chip_width = grid_item_width(ui.available_width(), columns, SECTION_GAP)
        .clamp(REPORT_SERIES_MIN_WIDTH, REPORT_SERIES_MAX_WIDTH);
    let compact = chip_width < 250.0;

    for (row_index, row) in chart_series.chunks(columns).enumerate() {
        clamped_content(ui, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for (index, series) in row.iter().rev().enumerate() {
                    render_chart_series_chip(ui, chip_width, series, compact);
                    if index + 1 < row.len() {
                        ui.add_space(SECTION_GAP);
                    }
                }
            });
        });

        if row_index + 1 < chart_series.len().div_ceil(columns) {
            ui.add_space(SECTION_GAP);
        }
    }
}

fn render_chart_series_chip(
    ui: &mut egui::Ui,
    chip_width: f32,
    series: &CategorySeries,
    compact: bool,
) {
    let (_fill, stroke, text) = activity_kind_colors(series.kind);
    let detail = if compact {
        format!(
            "{} kamas | {} sess.",
            format_kamas(series.total_in_period),
            series.session_count
        )
    } else {
        format!(
            "{} kamas | {} session(s)",
            format_kamas(series.total_in_period),
            series.session_count
        )
    };

    ui.allocate_ui_with_layout(
        egui::vec2(chip_width, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(chip_width);
            ui.set_max_width(chip_width);

            egui::Frame::none()
                .fill(theme::palette().surface_alt)
                .stroke(egui::Stroke::new(1.0, stroke))
                .rounding(999.0)
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_max_width(ui.available_width());

                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(text, "\u{25cf}");
                        ui.label(
                            egui::RichText::new(series.kind.label())
                                .size(11.0)
                                .strong()
                                .color(theme::palette().text_primary),
                        );
                    });

                    ui.add_space(4.0);
                    ui.add(
                        egui::Label::new(egui::RichText::new(detail).monospace().size(11.0).color(
                            if series.total_in_period >= 0.0 {
                                text
                            } else {
                                theme::palette().danger
                            },
                        ))
                        .wrap(true),
                    );
                });
        },
    );
}

fn activity_kind_colors(kind: ActivityKind) -> (egui::Color32, egui::Color32, egui::Color32) {
    match kind {
        ActivityKind::Zone => (
            egui::Color32::from_rgba_unmultiplied(91, 62, 27, 214),
            egui::Color32::from_rgb(231, 173, 86),
            egui::Color32::from_rgb(244, 208, 148),
        ),
        ActivityKind::Dungeon => (
            egui::Color32::from_rgba_unmultiplied(33, 63, 42, 214),
            egui::Color32::from_rgb(118, 191, 132),
            egui::Color32::from_rgb(184, 228, 192),
        ),
        ActivityKind::DuoTrio => (
            egui::Color32::from_rgba_unmultiplied(34, 58, 82, 214),
            egui::Color32::from_rgb(113, 171, 226),
            egui::Color32::from_rgb(181, 215, 244),
        ),
        ActivityKind::Arena => (
            egui::Color32::from_rgba_unmultiplied(77, 39, 36, 214),
            egui::Color32::from_rgb(215, 108, 98),
            egui::Color32::from_rgb(235, 176, 169),
        ),
    }
}

fn form_field_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    unit: &str,
    hint: &str,
) -> egui::Response {
    let colors = theme::palette();

    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .strong()
                .color(colors.text_primary),
        );
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(colors.card)
            .stroke(egui::Stroke::new(1.0, colors.border))
            .rounding(12.0)
            .inner_margin(egui::Margin::symmetric(12.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let spacing = if unit.is_empty() {
                        0.0
                    } else {
                        ui.spacing().item_spacing.x
                    };
                    let unit_width = if unit.is_empty() {
                        0.0
                    } else {
                        let galley = ui.painter().layout_no_wrap(
                            unit.to_owned(),
                            egui::TextStyle::Small.resolve(ui.style()),
                            colors.text_secondary,
                        );
                        galley.size().x + 14.0
                    };
                    let input_width = (ui.available_width() - unit_width - spacing).max(120.0);

                    let response = ui.add_sized(
                        [input_width, 24.0],
                        egui::TextEdit::singleline(value)
                            .hint_text(hint)
                            .frame(false)
                            .margin(egui::vec2(0.0, 0.0)),
                    );

                    if !unit.is_empty() {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(unit)
                                .size(11.0)
                                .color(colors.text_secondary),
                        );
                    }

                    response
                })
                .inner
            })
            .inner
    })
    .inner
}

fn date_field_row(ui: &mut egui::Ui, label: &str, value: &mut String) -> egui::Response {
    form_field_row(ui, label, value, "", "YYYY-MM-DD HH:MM")
}

fn duration_input_row(
    ui: &mut egui::Ui,
    id_source: &str,
    label: &str,
    value: &mut DurationInput,
) -> DurationInputResponse {
    let colors = theme::palette();
    normalize_duration_input_for_selector(value);
    let hours_id = egui::Id::new((id_source, "hours"));
    let minutes_id = egui::Id::new((id_source, "minutes"));
    let seconds_id = egui::Id::new((id_source, "seconds"));
    let mut field_response = DurationInputResponse::default();

    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .strong()
                .color(colors.text_primary),
        );
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(colors.card)
            .stroke(egui::Stroke::new(1.0, colors.border))
            .rounding(12.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    merge_duration_input_response(
                        &mut field_response,
                        duration_segment_combo(
                            ui,
                            hours_id,
                            "HH",
                            &mut value.hours,
                            DURATION_HOURS_MAX,
                        ),
                    );

                    ui.label(
                        egui::RichText::new(":")
                            .size(18.0)
                            .strong()
                            .color(colors.text_secondary),
                    );

                    merge_duration_input_response(
                        &mut field_response,
                        duration_segment_combo(
                            ui,
                            minutes_id,
                            "MM",
                            &mut value.minutes,
                            DURATION_SEGMENT_MAX,
                        ),
                    );

                    ui.label(
                        egui::RichText::new(":")
                            .size(18.0)
                            .strong()
                            .color(colors.text_secondary),
                    );

                    merge_duration_input_response(
                        &mut field_response,
                        duration_segment_combo(
                            ui,
                            seconds_id,
                            "SS",
                            &mut value.seconds,
                            DURATION_SEGMENT_MAX,
                        ),
                    );
                });
            });
    });

    field_response
}

fn merge_duration_input_response(
    aggregate: &mut DurationInputResponse,
    response: DurationInputResponse,
) {
    aggregate.changed |= response.changed;
    aggregate.has_focus |= response.has_focus;
}

fn duration_segment_combo(
    ui: &mut egui::Ui,
    id: egui::Id,
    segment_label: &str,
    value: &mut String,
    max: u32,
) -> DurationInputResponse {
    let colors = theme::palette();
    *value = normalize_duration_segment_for_selector(value, max);
    let before = value.clone();
    let selected_text = before.clone();

    let response = ui
        .vertical(|ui| {
            ui.label(
                egui::RichText::new(segment_label)
                    .size(10.0)
                    .strong()
                    .color(colors.text_secondary),
            );
            egui::ComboBox::from_id_source(id)
                .selected_text(
                    egui::RichText::new(selected_text.as_str())
                        .monospace()
                        .size(13.0)
                        .color(colors.text_primary),
                )
                .width(DURATION_COMBO_WIDTH)
                .show_ui(ui, |ui| {
                    for option in 0..=max {
                        let formatted = format!("{option:02}");
                        ui.selectable_value(
                            value,
                            formatted.clone(),
                            egui::RichText::new(formatted)
                                .monospace()
                                .size(13.0)
                                .color(colors.text_primary),
                        );
                    }
                })
                .response
        })
        .inner;

    DurationInputResponse {
        changed: *value != before,
        has_focus: response.has_focus(),
    }
}

fn normalize_duration_input_for_selector(value: &mut DurationInput) {
    value.hours = normalize_duration_segment_for_selector(&value.hours, DURATION_HOURS_MAX);
    value.minutes = normalize_duration_segment_for_selector(&value.minutes, DURATION_SEGMENT_MAX);
    value.seconds = normalize_duration_segment_for_selector(&value.seconds, DURATION_SEGMENT_MAX);
}

fn normalize_duration_segment_for_selector(value: &str, max: u32) -> String {
    let parsed = value.trim().parse::<u32>().unwrap_or(0).min(max);
    format!("{parsed:02}")
}

fn party_mode_selector(ui: &mut egui::Ui, label: &str, mode: &mut PartyMode) -> bool {
    let colors = theme::palette();
    let mut changed = false;

    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .strong()
                .color(colors.text_primary),
        );
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(colors.card)
            .stroke(egui::Stroke::new(1.0, colors.border))
            .rounding(12.0)
            .inner_margin(egui::Margin::symmetric(8.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for candidate in [PartyMode::Duo, PartyMode::Trio] {
                        let selected = *mode == candidate;
                        let fill = if selected {
                            colors.accent_soft
                        } else {
                            colors.surface
                        };
                        let stroke = if selected {
                            colors.accent
                        } else {
                            colors.border
                        };
                        let text = if selected {
                            colors.accent
                        } else {
                            colors.text_secondary
                        };

                        let response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(candidate.label())
                                    .size(12.0)
                                    .strong()
                                    .color(text),
                            )
                            .fill(fill)
                            .stroke(egui::Stroke::new(1.0, stroke))
                            .rounding(10.0),
                        );

                        if response.clicked() && *mode != candidate {
                            *mode = candidate;
                            changed = true;
                        }
                    }
                });
            });
    });

    changed
}

fn class_selector(
    ui: &mut egui::Ui,
    label: &str,
    selected_class: &mut Option<DofusClass>,
    class_textures: &HashMap<DofusClass, TextureHandle>,
) -> bool {
    let colors = theme::palette();
    let mut changed = false;

    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .strong()
                .color(colors.text_primary),
        );
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(colors.card)
            .stroke(egui::Stroke::new(1.0, colors.border))
            .rounding(12.0)
            .inner_margin(egui::Margin::symmetric(8.0, 8.0))
            .show(ui, |ui| {
                clamped_wrapped_row(ui, |ui| {
                    for class in DofusClass::all() {
                        let selected = *selected_class == Some(class);
                        let fill = if selected {
                            colors.accent_soft
                        } else {
                            colors.surface
                        };
                        let stroke = if selected {
                            colors.accent
                        } else {
                            colors.border
                        };
                        let text = if selected {
                            colors.accent
                        } else {
                            colors.text_primary
                        };

                        let button = if let Some(texture) = class_textures.get(&class) {
                            egui::Button::image_and_text(
                                egui::Image::from_texture(texture)
                                    .fit_to_exact_size(egui::vec2(22.0, 22.0))
                                    .rounding(5.0),
                                egui::RichText::new(class.label())
                                    .size(12.0)
                                    .strong()
                                    .color(text),
                            )
                        } else {
                            egui::Button::new(
                                egui::RichText::new(class.label())
                                    .size(12.0)
                                    .strong()
                                    .color(text),
                            )
                        }
                        .fill(fill)
                        .stroke(egui::Stroke::new(1.0, stroke))
                        .rounding(10.0)
                        .min_size(egui::vec2(108.0, 34.0));

                        if ui.add(button).clicked() && *selected_class != Some(class) {
                            *selected_class = Some(class);
                            changed = true;
                        }
                    }
                });
            });
    });

    changed
}

fn class_badge(
    ui: &mut egui::Ui,
    label: &str,
    class: Option<DofusClass>,
    tone: PreviewTone,
    class_textures: &HashMap<DofusClass, TextureHandle>,
) {
    let colors = theme::palette();
    let (fill, stroke, text) = tone_colors(tone);

    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .rounding(999.0)
        .inner_margin(egui::Margin::symmetric(9.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(10.0)
                        .color(colors.text_secondary),
                );

                if let Some(class) = class {
                    if let Some(texture) = class_textures.get(&class) {
                        ui.add(
                            egui::Image::from_texture(texture)
                                .fit_to_exact_size(egui::vec2(18.0, 18.0))
                                .rounding(4.0),
                        );
                    }
                }

                ui.label(
                    egui::RichText::new(class_label(class))
                        .size(11.0)
                        .strong()
                        .color(text),
                );
            });
        });
}

fn recommendation_badge(
    ui: &mut egui::Ui,
    recommended_class: DofusClass,
    current_class: Option<DofusClass>,
    class_textures: &HashMap<DofusClass, TextureHandle>,
) {
    let tone = if current_class == Some(recommended_class) {
        PreviewTone::Positive
    } else {
        PreviewTone::Accent
    };

    class_badge(
        ui,
        "Meilleure classe",
        Some(recommended_class),
        tone,
        class_textures,
    );
}

fn class_label(class: Option<DofusClass>) -> &'static str {
    class.map(DofusClass::label).unwrap_or("Classe inconnue")
}

fn duration_input_is_zero(value: &DurationInput) -> bool {
    value.hours.trim() == "00" && value.minutes.trim() == "00" && value.seconds.trim() == "00"
}

fn optional_duration_input_seconds(
    value: &DurationInput,
    touched: bool,
) -> Result<Option<f32>, String> {
    if !touched && duration_input_is_zero(value) {
        Ok(None)
    } else {
        parse_duration_input(value).map(Some)
    }
}

fn confirm_dialog(
    ctx: &egui::Context,
    open: &mut bool,
    title: &str,
    body: &str,
    confirm_label: &str,
) -> bool {
    if !*open {
        return false;
    }

    let mut confirmed = false;

    if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        *open = false;
        return false;
    }

    egui::Window::new(title)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .default_width(420.0)
        .show(ctx, |ui| {
            let colors = theme::palette();
            ui.label(
                egui::RichText::new(body)
                    .size(14.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                if secondary_button(ui, "Annuler").clicked() {
                    *open = false;
                }
                if danger_button(ui, confirm_label).clicked() {
                    confirmed = true;
                    *open = false;
                }
            });
        });

    confirmed
}

impl MyApp {
    pub fn render(&mut self, ctx: &egui::Context) {
        paint_background(ctx, self.background_texture.as_ref());
        self.render_header(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::symmetric(14.0, 8.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([true, false])
                    .show(ui, |ui| {
                        let available_width = ui.available_width();
                        ui.set_width(available_width);
                        ui.set_max_width(available_width);

                        match self.current_tab {
                            Tab::Bilans => self.ui_bilans(ui),
                            Tab::RechercheActivite => self.ui_activity_search(ui),
                            Tab::Zones => self.ui_zones(ui),
                            Tab::Donjons => self.ui_dungeons(ui),
                            Tab::DuoTrio => self.ui_duo_trios(ui),
                            Tab::PlArene => self.ui_arenas(ui),
                        }
                    });
            });

        if self.show_load_confirm {
            if let Some((title, body, confirm_label)) = self.pending_load_confirmation() {
                let confirmed = confirm_dialog(
                    ctx,
                    &mut self.show_load_confirm,
                    &title,
                    &body,
                    &confirm_label,
                );

                if confirmed {
                    self.confirm_pending_load();
                } else if !self.show_load_confirm {
                    self.cancel_load_confirmation();
                }
            } else {
                self.cancel_load_confirmation();
            }
        }

        if self.show_named_save_dialog {
            self.render_named_save_dialog(ctx);
        }

        if self.show_named_load_dialog {
            self.render_named_load_dialog(ctx);
        }

        if self.show_named_save_confirm {
            if let Some((title, body, confirm_label)) = self.pending_named_save_confirmation() {
                let confirmed = confirm_dialog(
                    ctx,
                    &mut self.show_named_save_confirm,
                    &title,
                    &body,
                    &confirm_label,
                );

                if confirmed {
                    self.confirm_pending_named_save_action();
                } else if !self.show_named_save_confirm {
                    self.cancel_named_save_confirmation();
                }
            } else {
                self.cancel_named_save_confirmation();
            }
        }

        if confirm_dialog(
            ctx,
            &mut self.show_clear_state_confirm,
            "Effacer l'etat",
            "Cette action vide seulement l'etat actuellement affiche. La sauvegarde locale n'est pas modifiee et pourra etre restauree avec l'option de rechargement.",
            "Effacer l'etat",
        ) {
            self.clear_current_state();
        }

        if confirm_dialog(
            ctx,
            &mut self.show_delete_local_save_confirm,
            "Supprimer la sauvegarde locale",
            "Cette action supprime definitivement le fichier local et son fichier .bak lorsqu'ils existent. L'etat actuellement affiche n'est pas efface automatiquement.",
            "Supprimer la sauvegarde",
        ) {
            self.delete_local_save();
        }
    }

    fn render_named_save_dialog(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();
        let mut open = self.show_named_save_dialog;
        let mut submit = false;

        egui::Window::new("Sauvegarder")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "Cree une sauvegarde nommee de l'etat actuel. L'autosave locale reste active en arriere-plan.",
                    )
                    .size(14.0)
                    .color(colors.text_secondary),
                );
                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new("Nom de la sauvegarde")
                        .size(13.0)
                        .strong()
                        .color(colors.text_primary),
                );
                ui.add_space(4.0);
                let response = dialog_text_input(
                    ui,
                    &mut self.named_save_name_input,
                    "Ex. Route Glours solo",
                );
                let submit_with_enter =
                    response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Resume de l'etat actuel")
                        .size(12.0)
                        .strong()
                        .color(colors.text_primary),
                );
                ui.label(
                    egui::RichText::new(persisted_state_summary(&self.build_persisted_state()))
                        .size(12.0)
                        .color(colors.text_secondary),
                );

                if let Some(error) = self.named_save_error.as_ref() {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(error)
                            .size(12.0)
                            .color(colors.danger),
                    );
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if secondary_button(ui, "Annuler").clicked() {
                        open = false;
                    }

                    if primary_button(ui, "Enregistrer").clicked() || submit_with_enter {
                        submit = true;
                    }
                });
            });

        if submit {
            self.request_named_save();
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            open = false;
        }

        if !open {
            self.close_named_save_dialog();
        }
    }

    fn render_named_load_dialog(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();
        let mut open = self.show_named_load_dialog;
        let named_saves = self.named_saves.clone();
        let mut load_action: Option<String> = None;
        let mut rename_action: Option<String> = None;
        let mut save_rename = false;
        let mut cancel_rename = false;
        let mut delete_action: Option<String> = None;
        let mut refresh_requested = false;

        egui::Window::new("Charger")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(true)
            .default_width(760.0)
            .default_height(560.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "Sauvegardes nommees triees de la plus recente a la plus ancienne.",
                        )
                        .size(14.0)
                        .color(colors.text_secondary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if secondary_button(ui, "Actualiser").clicked() {
                            refresh_requested = true;
                        }
                    });
                });

                if let Some(error) = self.named_saves_error.as_ref() {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(error)
                            .size(12.0)
                            .color(colors.danger),
                    );
                }

                ui.add_space(12.0);

                if named_saves.is_empty() {
                    ui.label(
                        egui::RichText::new("Aucune sauvegarde nommee disponible pour le moment.")
                            .size(13.0)
                            .color(colors.text_secondary),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([true, false])
                        .show(ui, |ui| {
                            for save in named_saves {
                                let rename_open = self
                                    .named_save_rename
                                    .as_ref()
                                    .is_some_and(|rename| rename.save_id == save.meta.save_id);

                                egui::Frame::none()
                                    .fill(colors.surface_alt)
                                    .stroke(egui::Stroke::new(1.0, colors.border))
                                    .rounding(12.0)
                                    .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                                    .show(ui, |ui| {
                                        ui.horizontal_top(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&save.meta.display_name)
                                                        .size(16.0)
                                                        .strong()
                                                        .color(colors.text_primary),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Mis a jour le {}",
                                                        format_named_save_updated_at(
                                                            save.meta.updated_at
                                                        )
                                                    ))
                                                    .size(11.0)
                                                    .color(colors.text_secondary),
                                                );
                                                ui.add_space(4.0);
                                                ui.label(
                                                    egui::RichText::new(named_save_summary(&save))
                                                        .size(12.0)
                                                        .color(colors.text_secondary),
                                                );

                                                if save.used_backup {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(
                                                            "Chargement de secours disponible via le .bak.",
                                                        )
                                                        .size(11.0)
                                                        .color(colors.accent),
                                                    );
                                                }
                                            });

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Min),
                                                |ui| {
                                                    if danger_button(ui, "Supprimer").clicked() {
                                                        delete_action =
                                                            Some(save.meta.save_id.clone());
                                                    }

                                                    if rename_open {
                                                        if secondary_button(ui, "Annuler").clicked()
                                                        {
                                                            cancel_rename = true;
                                                        }

                                                        if primary_button(ui, "Enregistrer")
                                                            .clicked()
                                                        {
                                                            save_rename = true;
                                                        }
                                                    } else if secondary_button(ui, "Renommer")
                                                        .clicked()
                                                    {
                                                        rename_action =
                                                            Some(save.meta.save_id.clone());
                                                    }

                                                    if primary_button(ui, "Charger").clicked() {
                                                        load_action =
                                                            Some(save.meta.save_id.clone());
                                                    }
                                                },
                                            );
                                        });

                                        if rename_open {
                                            ui.add_space(10.0);

                                            if let Some(rename) = self.named_save_rename.as_mut() {
                                                let response = dialog_text_input(
                                                    ui,
                                                    &mut rename.value,
                                                    "Nouveau nom",
                                                );
                                                if response.lost_focus()
                                                    && ui.input(|input| {
                                                        input.key_pressed(egui::Key::Enter)
                                                    })
                                                {
                                                    save_rename = true;
                                                }

                                                if let Some(error) = rename.error.as_ref() {
                                                    ui.add_space(6.0);
                                                    ui.label(
                                                        egui::RichText::new(error)
                                                            .size(11.0)
                                                            .color(colors.danger),
                                                    );
                                                }
                                            }
                                        }
                                    });

                                ui.add_space(8.0);
                            }
                        });
                }

                ui.add_space(8.0);
                if secondary_button(ui, "Fermer").clicked() {
                    open = false;
                }
            });

        if refresh_requested {
            let _ = self.refresh_named_saves();
        }

        if let Some(save_id) = rename_action {
            self.start_named_save_rename(&save_id);
        }

        if save_rename {
            self.save_named_save_rename();
        }

        if cancel_rename {
            self.cancel_named_save_rename();
        }

        if let Some(save_id) = delete_action {
            self.start_named_save_delete(&save_id);
        }

        if let Some(save_id) = load_action {
            self.start_named_save_load(&save_id);
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            open = false;
        }

        if !open {
            self.close_named_load_dialog();
        }
    }

    fn render_header(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();

        egui::TopBottomPanel::top("app_header")
            .frame(
                egui::Frame::none()
                    .fill(colors.background)
                    .inner_margin(egui::Margin::symmetric(14.0, 12.0)),
            )
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(colors.surface)
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .rounding(14.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 12.0;

                                if let Some(texture) = self.app_icon_texture.as_ref() {
                                    ui.add(
                                        egui::Image::from_texture(texture)
                                            .fit_to_exact_size(egui::vec2(
                                                HEADER_LOGO_SIZE,
                                                HEADER_LOGO_SIZE,
                                            ))
                                            .rounding(12.0),
                                    );
                                }

                                ui.vertical(|ui| {
                                    render_brand_title(ui, APP_NAME);
                                });
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if secondary_button(ui, "Charger").clicked() {
                                    self.open_named_load_dialog();
                                }

                                if primary_button(ui, "Sauvegarder").clicked() {
                                    self.open_named_save_dialog();
                                }

                                let options_menu = ui.menu_button(
                                    egui::RichText::new("⚙")
                                        .size(20.0)
                                        .strong()
                                        .color(colors.text_primary),
                                    |ui| {
                                        ui.set_min_width(240.0);

                                        let reload_response =
                                            secondary_button(ui, "Recharger l'autosave");
                                        let reload_clicked = reload_response.clicked();
                                        reload_response.on_hover_text(
                                            "Recharge la derniere sauvegarde locale disponible.",
                                        );
                                        if reload_clicked {
                                            self.request_reload_local();
                                            ui.close_menu();
                                        }

                                        let import_response =
                                            secondary_button(ui, "Importer un fichier JSON");
                                        let import_clicked = import_response.clicked();
                                        import_response.on_hover_text(
                                            "Importe un fichier JSON puis met a jour la sauvegarde locale.",
                                        );
                                        if import_clicked {
                                            ui.close_menu();
                                            if let Some(path) = rfd::FileDialog::new()
                                                .add_filter("JSON", &["json"])
                                                .pick_file()
                                            {
                                                self.request_import_external(path);
                                            }
                                        }

                                        ui.separator();

                                        let clear_response = secondary_button(ui, "Effacer l'etat");
                                        let clear_clicked = clear_response.clicked();
                                        clear_response.on_hover_text(
                                            "Vide seulement l'etat affiche.",
                                        );
                                        if clear_clicked {
                                            self.open_clear_state_dialog();
                                            ui.close_menu();
                                        }

                                        let delete_response =
                                            danger_button(ui, "Supprimer l'autosave");
                                        let delete_clicked = delete_response.clicked();
                                        delete_response.on_hover_text(
                                            "Supprime definitivement data.json et son fichier .bak.",
                                        );
                                        if delete_clicked {
                                            self.open_delete_local_save_dialog();
                                            ui.close_menu();
                                        }
                                    },
                                );
                                options_menu.response.on_hover_text("Options");
                            });
                        });

                        if let Some(status) = self.status.as_ref() {
                            ui.add_space(10.0);
                            if status_banner(ui, status) {
                                self.clear_status();
                            }
                        }

                        ui.add_space(10.0);
                        clamped_wrapped_row(ui, |ui| {
                            styled_tab_button(ui, &mut self.current_tab, Tab::Bilans, "Bilans");
                            styled_tab_button(
                                ui,
                                &mut self.current_tab,
                                Tab::RechercheActivite,
                                "Recherche d'activite",
                            );
                            styled_tab_button(ui, &mut self.current_tab, Tab::Zones, "Zones");
                            styled_tab_button(ui, &mut self.current_tab, Tab::Donjons, "Donjons");
                            styled_tab_button(
                                ui,
                                &mut self.current_tab,
                                Tab::DuoTrio,
                                "Duo / Trio",
                            );
                            styled_tab_button(ui, &mut self.current_tab, Tab::PlArene, "PL arene");
                        });
                    });
            });
    }

    pub fn ui_bilans(&mut self, ui: &mut egui::Ui) {
        let now = local_now();
        let summary =
            build_report_summary(&self.data, self.report_period, self.report_categories, now);

        section_frame(
            ui,
            "Graphique des gains",
            "Courbes cumulees par categorie ou batons temporels par session.",
            |ui| {
                self.render_report_controls(ui);
                ui.add_space(8.0);
                self.render_report_chart(ui, &summary);
            },
        );

        ui.add_space(6.0);
        self.render_bilans_overview(ui, &summary);

        if !can_fit_two_columns(
            ui.available_width(),
            REPORT_PANEL_MIN_WIDTH,
            REPORT_PANEL_MIN_WIDTH,
        ) {
            ui.add_space(10.0);
            section_frame(
                ui,
                "Top activites",
                "Cumuls nets par activite et meilleure session sur la periode visible.",
                |ui| self.render_top_activities(ui, &summary),
            );
            ui.add_space(10.0);
            section_frame(
                ui,
                "Bilan par classe",
                "Lecture comptable par classe sur les sessions horodatees visibles.",
                |ui| self.render_class_summaries(ui, &summary),
            );
            ui.add_space(10.0);
            section_frame(
                ui,
                "Sessions recentes",
                "Historique detaille, date+heure et gain net des saisies visibles.",
                |ui| self.render_recent_sessions(ui, &summary),
            );
            return;
        }

        ui.add_space(10.0);
        let (left_width, right_width) = split_two_column_widths(
            ui.available_width(),
            0.5,
            REPORT_PANEL_MIN_WIDTH,
            REPORT_PANEL_MIN_WIDTH,
        );

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    section_frame(
                        ui,
                        "Top activites",
                        "Cumuls nets par activite et meilleure session sur la periode visible.",
                        |ui| self.render_top_activities(ui, &summary),
                    );
                },
            );
            ui.add_space(SECTION_GAP);
            ui.allocate_ui_with_layout(
                egui::vec2(right_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    section_frame(
                        ui,
                        "Bilan par classe",
                        "Lecture comptable par classe sur les sessions horodatees visibles.",
                        |ui| self.render_class_summaries(ui, &summary),
                    );
                },
            );
        });

        ui.add_space(10.0);
        section_frame(
            ui,
            "Sessions recentes",
            "Historique detaille, date+heure et gain net des saisies visibles.",
            |ui| self.render_recent_sessions(ui, &summary),
        );
    }

    pub fn ui_activity_search(&mut self, ui: &mut egui::Ui) {
        let filter_title = "Filtres de recherche";
        let filter_subtitle =
            "Classe et temps disponible sont optionnels. Laissez vide pour voir le top historique.";

        if !can_fit_two_columns(
            ui.available_width(),
            ACTIVITY_FILTER_PANEL_MIN_WIDTH,
            ACTIVITY_RESULTS_PANEL_MIN_WIDTH,
        ) {
            section_frame(ui, filter_title, filter_subtitle, |ui| {
                self.render_activity_search_filters(ui);
            });
            ui.add_space(12.0);

            let available_time_seconds = self.activity_search_available_time_seconds();
            let groups = build_activity_recommendations(
                &self.data,
                &ActivitySearchFilters {
                    class: self.activity_search_class,
                    available_time_seconds,
                },
            );

            self.render_activity_search_results(ui, &groups, available_time_seconds.is_some());
            return;
        }

        let (left_width, right_width) = split_two_column_widths(
            ui.available_width(),
            0.33,
            ACTIVITY_FILTER_PANEL_MIN_WIDTH,
            ACTIVITY_RESULTS_PANEL_MIN_WIDTH,
        );

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    section_frame(ui, filter_title, filter_subtitle, |ui| {
                        self.render_activity_search_filters(ui);
                    });
                },
            );

            ui.add_space(SECTION_GAP);

            let available_time_seconds = self.activity_search_available_time_seconds();
            let groups = build_activity_recommendations(
                &self.data,
                &ActivitySearchFilters {
                    class: self.activity_search_class,
                    available_time_seconds,
                },
            );

            ui.allocate_ui_with_layout(
                egui::vec2(right_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.render_activity_search_results(
                        ui,
                        &groups,
                        available_time_seconds.is_some(),
                    );
                },
            );
        });
    }

    pub fn ui_zones(&mut self, ui: &mut egui::Ui) {
        self.render_kpis(
            ui,
            summarize_values(self.data.zones.iter().map(|entry| entry.kamas_per_hour)),
        );
        ui.add_space(14.0);
        self.render_responsive_sections(
            ui,
            "Ajouter une zone",
            "Session de farm et estimation HDV.",
            "Classement des zones",
            "Comparaison compacte des sessions.",
            |ui, app| app.render_zone_form(ui),
            |ui, app| app.render_zone_list(ui),
        );
    }

    pub fn ui_dungeons(&mut self, ui: &mut egui::Ui) {
        self.render_kpis(
            ui,
            summarize_values(self.data.dungeons.iter().map(|entry| entry.kamas_per_hour)),
        );
        ui.add_space(14.0);
        self.render_responsive_sections(
            ui,
            "Ajouter un donjon",
            "Net par run et rendement horaire.",
            "Classement des donjons",
            "Lecture rapide des meilleurs runs.",
            |ui, app| app.render_dungeon_form(ui),
            |ui, app| app.render_dungeon_list(ui),
        );
    }

    pub fn ui_duo_trios(&mut self, ui: &mut egui::Ui) {
        self.render_kpis(
            ui,
            summarize_values(self.data.duo_trios.iter().map(|entry| entry.kamas_per_hour)),
        );
        ui.add_space(14.0);
        self.render_responsive_sections(
            ui,
            "Ajouter un run duo/trio",
            "Capture pleine, loot et couts du run.",
            "Classement duo/trio",
            "Comparaison compacte des runs captures.",
            |ui, app| app.render_duo_trio_form(ui),
            |ui, app| app.render_duo_trio_list(ui),
        );
    }

    pub fn ui_arenas(&mut self, ui: &mut egui::Ui) {
        self.render_kpis(
            ui,
            summarize_values(self.data.arenas.iter().map(|entry| entry.kamas_per_hour)),
        );
        ui.add_space(14.0);
        self.render_responsive_sections(
            ui,
            "Ajouter un PL arene",
            "Ronde, places et captures.",
            "Classement du PL arene",
            "Comparaison compacte des sessions.",
            |ui, app| app.render_arena_form(ui),
            |ui, app| app.render_arena_list(ui),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_responsive_sections(
        &mut self,
        ui: &mut egui::Ui,
        left_title: &str,
        left_subtitle: &str,
        right_title: &str,
        right_subtitle: &str,
        mut left_content: impl FnMut(&mut egui::Ui, &mut Self),
        mut right_content: impl FnMut(&mut egui::Ui, &mut Self),
    ) {
        if !can_fit_two_columns(
            ui.available_width(),
            FORM_PANEL_MIN_WIDTH,
            LIST_PANEL_MIN_WIDTH,
        ) {
            section_frame(ui, left_title, left_subtitle, |ui| left_content(ui, self));
            ui.add_space(12.0);
            section_frame(ui, right_title, right_subtitle, |ui| {
                right_content(ui, self)
            });
        } else {
            let (left_width, right_width) = split_two_column_widths(
                ui.available_width(),
                0.39,
                FORM_PANEL_MIN_WIDTH,
                LIST_PANEL_MIN_WIDTH,
            );

            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(left_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        section_frame(ui, left_title, left_subtitle, |ui| left_content(ui, self));
                    },
                );
                ui.add_space(SECTION_GAP);
                ui.allocate_ui_with_layout(
                    egui::vec2(right_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        section_frame(ui, right_title, right_subtitle, |ui| {
                            right_content(ui, self)
                        });
                    },
                );
            });
        }
    }

    fn activity_search_available_time_seconds(&mut self) -> Option<f32> {
        match optional_duration_input_seconds(
            &self.activity_search_time,
            self.activity_search_time_touched,
        ) {
            Ok(value) => {
                self.activity_search_time_error = None;
                value
            }
            Err(error) => {
                self.activity_search_time_error = Some(error);
                None
            }
        }
    }

    fn render_activity_search_filters(&mut self, ui: &mut egui::Ui) {
        let colors = theme::palette();

        class_selector(
            ui,
            "Classe jouee",
            &mut self.activity_search_class,
            &self.class_textures,
        );

        ui.add_space(8.0);
        clamped_wrapped_row(ui, |ui| {
            if secondary_button(ui, "Effacer la classe").clicked() {
                self.activity_search_class = None;
            }

            if secondary_button(ui, "Reinitialiser les filtres").clicked() {
                self.activity_search_class = None;
                self.activity_search_time = DurationInput::default();
                self.activity_search_time_touched = false;
                self.activity_search_time_error = None;
            }
        });

        ui.add_space(10.0);
        let response = duration_input_row(
            ui,
            "activity-search-time",
            "Temps disponible",
            &mut self.activity_search_time,
        );

        if response.changed {
            self.activity_search_time_touched = true;
            self.activity_search_time_error = None;
        }

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "Choisissez un temps supérieur à 00:00:00 pour activer le filtre de durée.",
            )
            .size(11.0)
            .color(colors.text_secondary),
        );

        if let Some(error) = self.activity_search_time_error.as_ref() {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(error).size(12.0).color(colors.danger));
        }
    }

    fn render_activity_search_results(
        &self,
        ui: &mut egui::Ui,
        groups: &[ActivityRecommendationGroup],
        show_time_projection: bool,
    ) {
        let available_width = ui.available_width();
        let columns = activity_result_columns(available_width);
        let card_width = grid_item_width(available_width, columns, SECTION_GAP);
        let row_count = groups.len().div_ceil(columns);

        for (row_index, row) in groups.chunks(columns).enumerate() {
            clamped_content(ui, |ui| {
                ui.horizontal_top(|ui| {
                    for (index, group) in row.iter().enumerate() {
                        ui.allocate_ui_with_layout(
                            egui::vec2(card_width, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.render_activity_search_group(ui, group, show_time_projection);
                            },
                        );

                        if index + 1 < row.len() {
                            ui.add_space(SECTION_GAP);
                        }
                    }
                });
            });

            if row_index + 1 < row_count {
                ui.add_space(SECTION_GAP);
            }
        }
    }

    fn render_activity_search_group(
        &self,
        ui: &mut egui::Ui,
        group: &ActivityRecommendationGroup,
        show_time_projection: bool,
    ) {
        ui.set_width(ui.available_width());
        ui.set_max_width(ui.available_width());

        let subtitle = if show_time_projection {
            "Top 3 par gain estime sur la duree disponible."
        } else {
            "Top 3 historique par kamas/h moyen."
        };

        clamped_content(ui, |ui| {
            section_frame(ui, group.kind.label(), subtitle, |ui| {
                if group.items.is_empty() {
                    empty_state(
                        ui,
                        true,
                        "Aucune activite exploitable avec ces filtres.",
                        "Aucune activite exploitable avec ces filtres.",
                    );
                    return;
                }

                for (rank, item) in group.items.iter().enumerate() {
                    self.render_activity_recommendation_card(ui, rank, item);

                    if rank + 1 < group.items.len() {
                        ui.add_space(8.0);
                    }
                }
            });
        });
    }

    fn render_activity_recommendation_card(
        &self,
        ui: &mut egui::Ui,
        rank: usize,
        item: &ActivityRecommendation,
    ) {
        ui.set_width(ui.available_width());
        ui.set_max_width(ui.available_width());

        let colors = theme::palette();
        let (_fill, stroke, _text) = activity_kind_colors(item.kind);
        let compact_metrics = ui.available_width() < ACTIVITY_CARD_COMPACT_BREAKPOINT;
        let headline_value = item
            .estimated_total_value
            .map(|value| format!("{} kamas", format_kamas(value)))
            .unwrap_or_else(|| format!("{} kamas/h", format_kamas(item.average_kamas_per_hour)));
        let headline_color = profit_color(
            item.estimated_total_value
                .unwrap_or(item.average_kamas_per_hour),
        );

        egui::Frame::none()
            .fill(colors.surface_alt)
            .stroke(egui::Stroke::new(1.0, stroke))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_max_width(ui.available_width());

                render_ranked_card_header(
                    ui,
                    rank,
                    &item.name,
                    item.kind.singular_label(),
                    headline_value,
                    headline_color,
                );

                ui.add_space(8.0);
                class_badge(
                    ui,
                    "Classe",
                    item.class,
                    PreviewTone::Accent,
                    &self.class_textures,
                );

                ui.add_space(8.0);
                render_metric_row(
                    ui,
                    compact_metrics,
                    vec![
                        ("Sessions", item.session_count.to_string()),
                        (
                            "Duree moy.",
                            format_duration_hms(item.average_duration_seconds),
                        ),
                        (
                            "Gain / session",
                            format!("{} kamas", format_kamas(item.average_value_per_session)),
                        ),
                        (
                            "Kamas / h",
                            format!("{} kamas/h", format_kamas(item.average_kamas_per_hour)),
                        ),
                    ],
                );

                if let (Some(estimated_runs), Some(estimated_total_value)) =
                    (item.estimated_runs, item.estimated_total_value)
                {
                    ui.add_space(6.0);
                    render_metric_row(
                        ui,
                        compact_metrics,
                        vec![
                            ("Runs realisables", estimated_runs.to_string()),
                            (
                                "Gain estime",
                                format!("{} kamas", format_kamas(estimated_total_value)),
                            ),
                        ],
                    );
                }
            });
    }

    fn render_kpis(&self, ui: &mut egui::Ui, summary: KpiSummary) {
        let best = summary
            .best
            .map(|value| format!("{} kamas/h", format_kamas(value)))
            .unwrap_or_else(|| "Aucune donnée".to_string());
        let average = summary
            .average
            .map(|value| format!("{} kamas/h", format_kamas(value)))
            .unwrap_or_else(|| "Aucune donnée".to_string());
        let count = summary.count.to_string();

        clamped_wrapped_row(ui, |ui| {
            kpi_card(ui, "Meilleur", &best, PreviewTone::Positive);
            kpi_card(ui, "Moyenne", &average, PreviewTone::Accent);
            kpi_card(ui, "Entrees", &count, PreviewTone::Neutral);
        });
    }

    fn render_report_controls(&mut self, ui: &mut egui::Ui) {
        clamped_wrapped_row(ui, |ui| {
            ui.label(
                egui::RichText::new("Periode")
                    .size(12.0)
                    .color(theme::palette().text_secondary),
            );
            toggle_chip_button(
                ui,
                &mut self.report_period,
                ReportPeriod::Last24Hours,
                ReportPeriod::Last24Hours.label(),
            );
            toggle_chip_button(
                ui,
                &mut self.report_period,
                ReportPeriod::Last7Days,
                ReportPeriod::Last7Days.label(),
            );
            toggle_chip_button(
                ui,
                &mut self.report_period,
                ReportPeriod::Last30Days,
                ReportPeriod::Last30Days.label(),
            );
            toggle_chip_button(
                ui,
                &mut self.report_period,
                ReportPeriod::AllTime,
                "Depuis le debut",
            );
        });

        ui.add_space(6.0);
        clamped_wrapped_row(ui, |ui| {
            ui.label(
                egui::RichText::new("Categories")
                    .size(12.0)
                    .color(theme::palette().text_secondary),
            );
            toggle_multi_chip_button(ui, &mut self.report_categories.zones, "Zones");
            toggle_multi_chip_button(ui, &mut self.report_categories.dungeons, "Donjons");
            toggle_multi_chip_button(ui, &mut self.report_categories.duo_trios, "Duo / Trio");
            toggle_multi_chip_button(ui, &mut self.report_categories.arenas, "PL arene");
        });

        ui.add_space(6.0);
        clamped_wrapped_row(ui, |ui| {
            ui.checkbox(&mut self.report_bar_mode, "Mode baton");
            ui.label(
                egui::RichText::new(if self.report_bar_mode {
                    "Chevauchements empiles automatiquement par creneau temporel."
                } else {
                    "Courbes cumulees affichees par defaut."
                })
                .size(11.0)
                .color(theme::palette().text_secondary),
            );
        });

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("Filtres globaux du bilan. Sessions horodatees uniquement.")
                .size(11.0)
                .color(theme::palette().text_secondary),
        );
    }

    fn render_bilans_overview(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        let total_earned = format!("{} kamas", format_kamas(summary.total_earned));
        let average = summary
            .average_per_session
            .map(|value| format!("{} kamas", format_kamas(value)))
            .unwrap_or_else(|| "Aucune donnee".to_string());
        let best_session_value = summary
            .best_session
            .as_ref()
            .map(|session| format!("{} kamas", format_kamas(session.accounting_value)))
            .unwrap_or_else(|| "Aucune donnee".to_string());
        let best_session_detail = summary
            .best_session
            .as_ref()
            .map(|session| {
                format!(
                    "{} | {} | {}",
                    session.kind.singular_label(),
                    session.name,
                    recorded_at_label(session.recorded_at)
                )
            })
            .unwrap_or_else(|| {
                "Ajoutez des sessions horodatees pour alimenter le bilan.".to_string()
            });
        let best_activity_value = summary
            .best_activity
            .as_ref()
            .map(|activity| activity.name.clone())
            .unwrap_or_else(|| "Aucune donnee".to_string());
        let best_activity_detail = summary
            .best_activity
            .as_ref()
            .map(|activity| {
                format!(
                    "{} | Total {} kamas | {} session(s)",
                    activity.kind.singular_label(),
                    format_kamas(activity.total_value),
                    activity.sessions
                )
            })
            .unwrap_or_else(|| "Aucune activite agregee pour le filtre actif.".to_string());

        let cards = [
            (
                "Sessions",
                PreviewBlock {
                    value: summary.session_count.to_string(),
                    detail: format!(
                        "{} | {}",
                        self.report_categories.summary_label(),
                        self.report_period.label()
                    ),
                    tone: PreviewTone::Neutral,
                },
            ),
            (
                "Total cumule",
                PreviewBlock {
                    value: total_earned,
                    detail: "Somme nette des sessions horodatees visibles.".to_string(),
                    tone: if summary.total_earned >= 0.0 {
                        PreviewTone::Accent
                    } else {
                        PreviewTone::Danger
                    },
                },
            ),
            (
                "Meilleure session",
                PreviewBlock {
                    value: best_session_value,
                    detail: best_session_detail,
                    tone: summary
                        .best_session
                        .as_ref()
                        .map(|session| {
                            if session.accounting_value > 0.0 {
                                PreviewTone::Positive
                            } else {
                                PreviewTone::Danger
                            }
                        })
                        .unwrap_or(PreviewTone::Neutral),
                },
            ),
            (
                "Moyenne / session",
                PreviewBlock {
                    value: average,
                    detail: "Gain moyen par session horodatee visible.".to_string(),
                    tone: PreviewTone::Accent,
                },
            ),
            (
                "Activite dominante",
                PreviewBlock {
                    value: best_activity_value,
                    detail: best_activity_detail,
                    tone: PreviewTone::Accent,
                },
            ),
        ];

        let available_width = ui.available_width();
        let columns = responsive_columns(
            available_width,
            REPORT_KPI_MIN_WIDTH,
            SECTION_GAP,
            REPORT_KPI_MAX_COLUMNS,
        );
        let card_width = grid_item_width(available_width, columns, SECTION_GAP);
        let row_count = cards.len().div_ceil(columns);

        for (row_index, row) in cards.chunks(columns).enumerate() {
            clamped_content(ui, |ui| {
                ui.horizontal_top(|ui| {
                    for (index, (title, preview)) in row.iter().enumerate() {
                        ui.allocate_ui_with_layout(
                            egui::vec2(card_width, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                preview_card_sized(ui, title, preview, card_width);
                            },
                        );

                        if index + 1 < row.len() {
                            ui.add_space(SECTION_GAP);
                        }
                    }
                });
            });

            if row_index + 1 < row_count {
                ui.add_space(SECTION_GAP);
            }
        }
    }

    fn render_report_chart(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        if !self.report_categories.any_selected() {
            empty_state(
                ui,
                true,
                "Activez au moins une categorie pour tracer le graphique.",
                "Activez au moins une categorie pour tracer le graphique.",
            );
            return;
        }

        let has_chart_data = if self.report_bar_mode {
            summary
                .chart_bars
                .iter()
                .any(|series| !series.segments.is_empty())
        } else {
            summary
                .chart_series
                .iter()
                .any(|series| !series.points.is_empty())
        };

        if !has_chart_data {
            empty_state(
                ui,
                true,
                "Aucune session horodatee visible dans cette periode.",
                "Aucune session horodatee visible dans cette periode.",
            );
            return;
        }

        if self.report_bar_mode {
            self.render_timeline_bar_chart(ui, summary);
        } else {
            self.render_cumulative_chart(ui, summary);
        }
    }

    fn render_timeline_bar_chart(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        ui.label(
            egui::RichText::new(
                "Survolez un baton pour voir la session, sa duree et les empilements sur le meme creneau.",
            )
            .size(11.0)
            .color(theme::palette().text_secondary),
        );
        ui.add_space(6.0);

        let mut plot = Plot::new("bilans_timeline_bar_plot")
            .height(REPORT_CHART_HEIGHT)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .show_axes([true, true])
            .include_y(0.0)
            .label_formatter(|_, _| String::new())
            .x_axis_formatter(|mark, _, range| format_plot_time_mark(mark.value, range))
            .y_axis_formatter(|mark, _, _| format_plot_kamas_mark(mark.value));

        if let Some(start_at) = summary.period_started_at {
            plot = plot.include_x(datetime_to_plot_x(start_at));
        }

        let plot_response = plot.show(ui, |plot_ui| {
            for series in summary
                .chart_bars
                .iter()
                .filter(|series| !series.segments.is_empty())
            {
                plot_ui.bar_chart(build_category_bar_chart(series));
            }
        });

        if plot_response.hovered_plot_item.is_some() {
            if let Some(pointer_pos) = plot_response.response.hover_pos() {
                if let Some(segment) =
                    find_hovered_bar_segment(summary, &plot_response.transform, pointer_pos)
                {
                    plot_response.response.clone().on_hover_ui_at_pointer(|ui| {
                        render_bar_segment_tooltip(ui, segment);
                    });
                }
            }
        }

        ui.add_space(6.0);
        render_chart_series_footer(ui, &summary.chart_series);
    }

    fn render_cumulative_chart(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        if !self.report_categories.any_selected() {
            empty_state(
                ui,
                true,
                "Activez au moins une categorie pour tracer la courbe.",
                "Activez au moins une categorie pour tracer la courbe.",
            );
            return;
        }

        if summary
            .chart_series
            .iter()
            .all(|series| series.points.is_empty())
        {
            empty_state(
                ui,
                true,
                "Aucune session horodatee visible dans cette periode.",
                "Aucune session horodatee visible dans cette periode.",
            );
            return;
        }

        ui.label(
            egui::RichText::new("Survolez un point pour voir ce qui a ete farme a cet instant.")
                .size(11.0)
                .color(theme::palette().text_secondary),
        );
        ui.add_space(6.0);

        let Some((min_x, max_x)) = cumulative_chart_x_bounds(summary) else {
            empty_state(
                ui,
                true,
                "Aucune session horodatee visible dans cette periode.",
                "Aucune session horodatee visible dans cette periode.",
            );
            return;
        };

        let plot_response = Plot::new("bilans_cumulative_plot")
            .height(REPORT_CHART_HEIGHT)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .show_axes([true, true])
            .label_formatter(|_, _| String::new())
            .x_axis_formatter(|mark, _, range| format_plot_time_mark(mark.value, range))
            .y_axis_formatter(|mark, _, _| format_plot_kamas_mark(mark.value))
            .include_x(min_x)
            .include_x(max_x)
            .include_y(0.0)
            .show(ui, |plot_ui| {
                for series in summary
                    .chart_series
                    .iter()
                    .filter(|series| !series.points.is_empty())
                {
                    let (_fill, stroke, text) = activity_kind_colors(series.kind);

                    plot_ui.line(
                        Line::new(PlotPoints::from_iter(
                            build_cumulative_line_points(series, summary.period_started_at)
                                .into_iter(),
                        ))
                        .color(stroke)
                        .width(2.5),
                    );
                    plot_ui.points(
                        Points::new(PlotPoints::from_iter(series.points.iter().map(|point| {
                            [
                                datetime_to_plot_x(point.recorded_at),
                                point.cumulative_value as f64,
                            ]
                        })))
                        .color(text)
                        .radius(5.0),
                    );
                }
            });

        if let Some(pointer_pos) = plot_response.response.hover_pos() {
            let hovered_point = summary
                .chart_series
                .iter()
                .flat_map(|series| series.points.iter())
                .filter_map(|point| {
                    let plot_point = PlotPoint::new(
                        datetime_to_plot_x(point.recorded_at),
                        point.cumulative_value as f64,
                    );
                    let screen_pos = plot_response.transform.position_from_point(&plot_point);
                    let distance = screen_pos.distance(pointer_pos);
                    (distance <= 18.0).then_some((distance, point))
                })
                .min_by(
                    |(left_distance, left_point), (right_distance, right_point)| {
                        left_distance.total_cmp(right_distance).then_with(|| {
                            right_point
                                .cumulative_value
                                .total_cmp(&left_point.cumulative_value)
                        })
                    },
                );

            if let Some((_, point)) = hovered_point {
                plot_response.response.clone().on_hover_ui_at_pointer(|ui| {
                    render_cumulative_point_tooltip(ui, point);
                });
            }
        }

        ui.add_space(6.0);
        render_chart_series_footer(ui, &summary.chart_series);
        if false {
            clamped_wrapped_row(ui, |ui| {
                for series in &summary.chart_series {
                    let (_fill, stroke, text) = activity_kind_colors(series.kind);
                    egui::Frame::none()
                        .fill(theme::palette().surface_alt)
                        .stroke(egui::Stroke::new(1.0, stroke))
                        .rounding(999.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(text, "●");
                                ui.label(
                                    egui::RichText::new(series.kind.label())
                                        .size(11.0)
                                        .strong()
                                        .color(theme::palette().text_primary),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} kamas | {} session(s)",
                                        format_kamas(series.total_in_period),
                                        series.session_count
                                    ))
                                    .monospace()
                                    .size(11.0)
                                    .color(
                                        if series.total_in_period >= 0.0 {
                                            text
                                        } else {
                                            theme::palette().danger
                                        },
                                    ),
                                );
                            });
                        });
                }
            });
        }
    }

    fn render_top_activities(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        if summary.top_activities.is_empty() {
            empty_state(
                ui,
                true,
                "Aucune activite a comparer.",
                "Aucune activite a comparer.",
            );
            return;
        }

        egui::Grid::new("bilans_top_activities_grid")
            .num_columns(6)
            .spacing(egui::vec2(12.0, 8.0))
            .striped(true)
            .show(ui, |ui| {
                table_header(ui, "Activite");
                table_header(ui, "Type");
                table_header(ui, "Classe");
                table_header(ui, "Total");
                table_header(ui, "Meilleur");
                table_header(ui, "Sessions");
                ui.end_row();

                for activity in summary.top_activities.iter().take(REPORT_TABLE_LIMIT) {
                    ui.label(activity.name.clone());
                    muted_text(ui, activity.kind.singular_label());
                    muted_text(
                        ui,
                        activity
                            .best_class
                            .map(DofusClass::label)
                            .unwrap_or("Inconnue"),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{} kamas",
                            format_kamas(activity.total_value)
                        ))
                        .monospace()
                        .color(profit_color(activity.total_value)),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{} kamas",
                            format_kamas(activity.best_session)
                        ))
                        .monospace()
                        .color(profit_color(activity.best_session)),
                    );
                    muted_text(ui, &activity.sessions.to_string());
                    ui.end_row();
                }
            });
    }

    fn render_class_summaries(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        if summary.class_summaries.is_empty() {
            empty_state(
                ui,
                true,
                "Aucune classe exploitable dans ce filtre.",
                "Aucune classe exploitable dans ce filtre.",
            );
            return;
        }

        egui::Grid::new("bilans_class_summary_grid")
            .num_columns(5)
            .spacing(egui::vec2(12.0, 8.0))
            .striped(true)
            .show(ui, |ui| {
                table_header(ui, "Classe");
                table_header(ui, "Sessions");
                table_header(ui, "Total");
                table_header(ui, "Meilleur");
                table_header(ui, "Type fort");
                ui.end_row();

                for item in &summary.class_summaries {
                    ui.label(item.class.label());
                    muted_text(ui, &item.sessions.to_string());
                    ui.label(
                        egui::RichText::new(format!("{} kamas", format_kamas(item.total_value)))
                            .monospace()
                            .color(profit_color(item.total_value)),
                    );
                    ui.label(
                        egui::RichText::new(format!("{} kamas", format_kamas(item.best)))
                            .monospace()
                            .color(profit_color(item.best)),
                    );
                    muted_text(ui, item.best_kind.singular_label());
                    ui.end_row();
                }
            });
    }

    fn render_recent_sessions(&self, ui: &mut egui::Ui, summary: &ReportSummary) {
        if summary.recent_sessions.is_empty() {
            empty_state(
                ui,
                true,
                "Aucune session recente pour ce filtre.",
                "Aucune session recente pour ce filtre.",
            );
            return;
        }

        let colors = theme::palette();

        for session in &summary.recent_sessions {
            egui::Frame::none()
                .fill(colors.surface_alt)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .rounding(14.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    if ui.available_width() < CARD_HEADER_STACK_BREAKPOINT {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(session.name.clone())
                                    .size(15.0)
                                    .strong()
                                    .color(colors.text_primary),
                            );
                            ui.label(
                                egui::RichText::new(session.kind.singular_label())
                                    .size(11.0)
                                    .color(colors.text_secondary),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} kamas",
                                    format_kamas(session.accounting_value)
                                ))
                                .monospace()
                                .size(16.0)
                                .strong()
                                .color(profit_color(session.accounting_value)),
                            );
                        });
                    } else {
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(session.name.clone())
                                        .size(15.0)
                                        .strong()
                                        .color(colors.text_primary),
                                );
                                ui.label(
                                    egui::RichText::new(session.kind.singular_label())
                                        .size(11.0)
                                        .color(colors.text_secondary),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} kamas",
                                            format_kamas(session.accounting_value)
                                        ))
                                        .monospace()
                                        .size(16.0)
                                        .strong()
                                        .color(profit_color(session.accounting_value)),
                                    );
                                },
                            );
                        });
                    }

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        metric_badge(ui, "Date + heure", recorded_at_label(session.recorded_at));
                        metric_badge(
                            ui,
                            "Classe",
                            session
                                .class
                                .map(|class| class.label().to_string())
                                .unwrap_or_else(|| "Inconnue".to_string()),
                        );
                        metric_badge(
                            ui,
                            "Rendement",
                            format!("{} kamas/h", format_kamas(session.kamas_per_hour)),
                        );
                    });
                });

            ui.add_space(8.0);
        }
    }
}

pub fn recommended_zone_class(entries: &[ZoneEntry], activity_name: &str) -> Option<DofusClass> {
    recommended_class_for_name(
        entries,
        activity_name,
        |entry| &entry.name,
        |entry| {
            let kamas_per_hour = if entry.session_time_seconds > 0.0 {
                zone_kamas_per_hour(entry.session_total_kamas, entry.session_time_seconds)
            } else {
                entry.kamas_per_hour
            };

            (entry.character_class, kamas_per_hour)
        },
    )
}

pub fn recommended_dungeon_class(
    entries: &[DungeonEntry],
    activity_name: &str,
) -> Option<DofusClass> {
    recommended_class_for_name(
        entries,
        activity_name,
        |entry| &entry.name,
        |entry| {
            let net_kamas_per_run = entry.gross_kamas_per_run - entry.key_price;
            let kamas_per_hour = if entry.run_time_minutes > 0.0 {
                dungeon_kamas_per_hour(net_kamas_per_run, entry.run_time_minutes)
            } else {
                entry.kamas_per_hour
            };

            (entry.character_class, kamas_per_hour)
        },
    )
}

pub fn recommended_duo_trio_class(
    entries: &[DuoTrioEntry],
    activity_name: &str,
) -> Option<DofusClass> {
    recommended_class_for_name(
        entries,
        activity_name,
        |entry| &entry.name,
        |entry| {
            let total_key_cost = entry.key_unit_price * entry.party_mode.player_count() as f32;
            let gross_kamas_per_run = entry.loot_kamas_per_run + entry.full_soul_sale_price;
            let total_cost = entry.capture_stone_price + total_key_cost;
            let net_kamas_per_run = gross_kamas_per_run - total_cost;
            let kamas_per_hour = if entry.run_time_seconds > 0.0 {
                duo_trio_kamas_per_hour(net_kamas_per_run, entry.run_time_seconds)
            } else {
                entry.kamas_per_hour
            };

            (entry.character_class, kamas_per_hour)
        },
    )
}

pub fn recommended_arena_class(entries: &[ArenaEntry], activity_name: &str) -> Option<DofusClass> {
    recommended_class_for_name(
        entries,
        activity_name,
        |entry| &entry.name,
        |entry| {
            let gross_revenue = entry.seat_price * entry.seats_sold as f32;
            let total_capture_cost = entry.capture_price * entry.captures_count as f32;
            let net_profit = gross_revenue - total_capture_cost;
            let kamas_per_hour = if entry.round_time_minutes > 0.0 {
                arena_kamas_per_hour(net_profit, entry.round_time_minutes)
            } else {
                entry.kamas_per_hour
            };

            (entry.character_class, kamas_per_hour)
        },
    )
}

fn recommended_class_for_name<T>(
    entries: &[T],
    activity_name: &str,
    name_of: impl Fn(&T) -> &str,
    metrics_of: impl Fn(&T) -> (Option<DofusClass>, f32),
) -> Option<DofusClass> {
    let normalized_target = normalize_activity_name(activity_name);
    let mut best_per_class: HashMap<DofusClass, f32> = HashMap::new();

    for entry in entries {
        if normalize_activity_name(name_of(entry)) != normalized_target {
            continue;
        }

        let (character_class, kamas_per_hour) = metrics_of(entry);
        let Some(character_class) = character_class else {
            continue;
        };

        best_per_class
            .entry(character_class)
            .and_modify(|best| *best = best.max(kamas_per_hour))
            .or_insert(kamas_per_hour);
    }

    best_per_class
        .into_iter()
        .max_by(|(left_class, left_score), (right_class, right_score)| {
            left_score
                .total_cmp(right_score)
                .then_with(|| left_class.cmp(right_class))
        })
        .map(|(class, _)| class)
}

fn normalize_activity_name(name: &str) -> String {
    normalize_text_for_matching(name)
}

impl MyApp {
    fn apply_zone_list_action(&mut self, action: ZoneListAction) {
        match action {
            ZoneListAction::StartEdit(index) => self.start_zone_edit(index),
            ZoneListAction::RequestDelete(index) => {
                self.zone_delete_confirm = Some(index);
                self.zone_edit = None;
            }
            ZoneListAction::CancelDelete(index) => {
                if self.zone_delete_confirm == Some(index) {
                    self.zone_delete_confirm = None;
                }
            }
            ZoneListAction::ConfirmDelete(index) => self.confirm_zone_delete(index),
            ZoneListAction::SaveEdit => self.save_zone_edit(),
            ZoneListAction::CancelEdit => self.cancel_zone_edit(),
        }
    }

    fn apply_dungeon_list_action(&mut self, action: DungeonListAction) {
        match action {
            DungeonListAction::StartEdit(index) => self.start_dungeon_edit(index),
            DungeonListAction::RequestDelete(index) => {
                self.dungeon_delete_confirm = Some(index);
                self.dungeon_edit = None;
            }
            DungeonListAction::CancelDelete(index) => {
                if self.dungeon_delete_confirm == Some(index) {
                    self.dungeon_delete_confirm = None;
                }
            }
            DungeonListAction::ConfirmDelete(index) => self.confirm_dungeon_delete(index),
            DungeonListAction::SaveEdit => self.save_dungeon_edit(),
            DungeonListAction::CancelEdit => self.cancel_dungeon_edit(),
        }
    }

    fn apply_duo_trio_list_action(&mut self, action: DuoTrioListAction) {
        match action {
            DuoTrioListAction::StartEdit(index) => self.start_duo_trio_edit(index),
            DuoTrioListAction::RequestDelete(index) => {
                self.duo_trio_delete_confirm = Some(index);
                self.duo_trio_edit = None;
            }
            DuoTrioListAction::CancelDelete(index) => {
                if self.duo_trio_delete_confirm == Some(index) {
                    self.duo_trio_delete_confirm = None;
                }
            }
            DuoTrioListAction::ConfirmDelete(index) => self.confirm_duo_trio_delete(index),
            DuoTrioListAction::SaveEdit => self.save_duo_trio_edit(),
            DuoTrioListAction::CancelEdit => self.cancel_duo_trio_edit(),
        }
    }

    fn apply_arena_list_action(&mut self, action: ArenaListAction) {
        match action {
            ArenaListAction::StartEdit(index) => self.start_arena_edit(index),
            ArenaListAction::RequestDelete(index) => {
                self.arena_delete_confirm = Some(index);
                self.arena_edit = None;
            }
            ArenaListAction::CancelDelete(index) => {
                if self.arena_delete_confirm == Some(index) {
                    self.arena_delete_confirm = None;
                }
            }
            ArenaListAction::ConfirmDelete(index) => self.confirm_arena_delete(index),
            ArenaListAction::SaveEdit => self.save_arena_edit(),
            ArenaListAction::CancelEdit => self.cancel_arena_edit(),
        }
    }
}

impl MyApp {
    fn render_zone_form(&mut self, ui: &mut egui::Ui) {
        let mut has_focus = false;
        let mut changed = false;

        let response = form_field_row(
            ui,
            "Nom de la zone",
            &mut self.zone_form.name,
            "",
            "Exemple : Cimetière, Craqueleurs, Scarafeuilles...",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        changed |= class_selector(
            ui,
            "Classe jouee",
            &mut self.zone_form.character_class,
            &self.class_textures,
        );

        let response = date_field_row(
            ui,
            "Date + heure de session",
            &mut self.zone_form.recorded_at_input,
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = duration_input_row(
            ui,
            "zone-form-session-time",
            "Temps de la session",
            &mut self.zone_form.session_time,
        );
        has_focus |= response.has_focus;
        changed |= response.changed;

        let response = form_field_row(
            ui,
            "Valeur totale mise en vente",
            &mut self.zone_form.session_total_kamas,
            "kamas",
            "Valeur brute estimee des ressources de la session.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        if changed {
            self.zone_form_error = None;
        }

        ui.add_space(8.0);
        preview_card(ui, "Prévision immédiate", preview_zone(&self.zone_form));

        if let Some(error) = self.zone_form_error.as_deref() {
            ui.add_space(8.0);
            local_error_box(ui, error);
        }

        let submit_with_enter = has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
        ui.add_space(10.0);
        if wide_primary_button(ui, "Ajouter la zone").clicked() || submit_with_enter {
            self.submit_zone_form();
        }
    }

    fn render_dungeon_form(&mut self, ui: &mut egui::Ui) {
        let mut has_focus = false;
        let mut changed = false;

        let response = form_field_row(
            ui,
            "Nom du donjon",
            &mut self.dungeon_form.name,
            "",
            "Exemple : Blop, Rats, Dragon Cochon...",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        changed |= class_selector(
            ui,
            "Classe jouee",
            &mut self.dungeon_form.character_class,
            &self.class_textures,
        );

        let response = date_field_row(
            ui,
            "Date + heure du run",
            &mut self.dungeon_form.recorded_at_input,
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = duration_input_row(
            ui,
            "dungeon-form-run-time",
            "Temps moyen du run",
            &mut self.dungeon_form.run_time,
        );
        has_focus |= response.has_focus;
        changed |= response.changed;

        let response = form_field_row(
            ui,
            "Gain brut par run",
            &mut self.dungeon_form.gross_kamas_per_run,
            "kamas",
            "Valeur totale avant le coût de la clé.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Prix de la clé",
            &mut self.dungeon_form.key_price,
            "kamas",
            "Déduisez ici le coût du consommable.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        if changed {
            self.dungeon_form_error = None;
        }

        ui.add_space(8.0);
        preview_card(
            ui,
            "Prévision immédiate",
            preview_dungeon(&self.dungeon_form),
        );

        if let Some(error) = self.dungeon_form_error.as_deref() {
            ui.add_space(8.0);
            local_error_box(ui, error);
        }

        let submit_with_enter = has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
        ui.add_space(10.0);
        if wide_primary_button(ui, "Ajouter le donjon").clicked() || submit_with_enter {
            self.submit_dungeon_form();
        }
    }

    fn render_duo_trio_form(&mut self, ui: &mut egui::Ui) {
        let mut has_focus = false;
        let mut changed = false;

        let response = form_field_row(
            ui,
            "Nom / reference",
            &mut self.duo_trio_form.name,
            "",
            "Exemple : Duo Qu'Tan, Trio Illy...",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        changed |= class_selector(
            ui,
            "Classe jouee",
            &mut self.duo_trio_form.character_class,
            &self.class_textures,
        );

        let response = date_field_row(
            ui,
            "Date + heure du run",
            &mut self.duo_trio_form.recorded_at_input,
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        changed |= party_mode_selector(ui, "Mode", &mut self.duo_trio_form.party_mode);

        let response = duration_input_row(
            ui,
            "duo-trio-form-run-time",
            "Temps du run",
            &mut self.duo_trio_form.run_time,
        );
        has_focus |= response.has_focus;
        changed |= response.changed;

        let response = form_field_row(
            ui,
            "Loot total du run",
            &mut self.duo_trio_form.loot_kamas_per_run,
            "kamas",
            "Valeur du loot hors capture pleine.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Prix unitaire d'une clef",
            &mut self.duo_trio_form.key_unit_price,
            "kamas",
            "Le total est calcule automatiquement selon duo ou trio.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Prix de la pierre de capture",
            &mut self.duo_trio_form.capture_stone_price,
            "kamas",
            "Une pierre est prise en compte par run.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Prix de vente de la capture pleine",
            &mut self.duo_trio_form.full_soul_sale_price,
            "kamas",
            "Valeur de revente de la pierre pleine.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        if changed {
            self.duo_trio_form_error = None;
        }

        ui.add_space(8.0);
        preview_card(
            ui,
            "Prevision immediate",
            preview_duo_trio(&self.duo_trio_form),
        );

        if let Some(error) = self.duo_trio_form_error.as_deref() {
            ui.add_space(8.0);
            local_error_box(ui, error);
        }

        let submit_with_enter = has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
        ui.add_space(10.0);
        if wide_primary_button(ui, "Ajouter le run").clicked() || submit_with_enter {
            self.submit_duo_trio_form();
        }
    }

    fn render_arena_form(&mut self, ui: &mut egui::Ui) {
        let mut has_focus = false;
        let mut changed = false;

        let response = form_field_row(
            ui,
            "Nom / référence",
            &mut self.arena_form.name,
            "",
            "Nom de la session ou du boss concerné.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        changed |= class_selector(
            ui,
            "Classe jouee",
            &mut self.arena_form.character_class,
            &self.class_textures,
        );

        let response = date_field_row(
            ui,
            "Date + heure de session",
            &mut self.arena_form.recorded_at_input,
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = duration_input_row(
            ui,
            "arena-form-round-time",
            "Temps de la ronde",
            &mut self.arena_form.round_time,
        );
        has_focus |= response.has_focus;
        changed |= response.changed;

        let response = form_field_row(
            ui,
            "Prix d'une place",
            &mut self.arena_form.seat_price,
            "kamas",
            "Tarif vendu par siège.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Places vendues",
            &mut self.arena_form.seats_sold,
            "",
            "Nombre de sièges réellement vendus.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Prix d'une capture",
            &mut self.arena_form.capture_price,
            "kamas",
            "Coût unitaire d'une capture.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        let response = form_field_row(
            ui,
            "Nombre de captures",
            &mut self.arena_form.captures_count,
            "",
            "Quantité consommée pour la ronde.",
        );
        has_focus |= response.has_focus();
        changed |= response.changed();

        if changed {
            self.arena_form_error = None;
        }

        ui.add_space(8.0);
        preview_card(ui, "Prévision immédiate", preview_arena(&self.arena_form));

        if let Some(error) = self.arena_form_error.as_deref() {
            ui.add_space(8.0);
            local_error_box(ui, error);
        }

        let submit_with_enter = has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
        ui.add_space(10.0);
        if wide_primary_button(ui, "Ajouter le PL arene").clicked() || submit_with_enter {
            self.submit_arena_form();
        }
    }

    fn render_zone_list(&mut self, ui: &mut egui::Ui) {
        list_toolbar(ui, &mut self.zone_search, "Rechercher une zone");
        ui.add_space(10.0);

        let filtered: Vec<usize> = self
            .data
            .zones
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches_search(&entry.name, &self.zone_search))
            .map(|(index, _)| index)
            .collect();

        if filtered.is_empty() {
            empty_state(
                ui,
                self.data.zones.is_empty(),
                "Aucune zone enregistrée.",
                "Aucune zone ne correspond à cette recherche.",
            );
            return;
        }

        let colors = theme::palette();
        let mut pending_action: Option<ZoneListAction> = None;

        for (rank, index) in filtered.into_iter().enumerate() {
            if self.zone_edit.as_ref().map(|edit| edit.index) == Some(index) {
                pending_action = self.render_zone_edit_card(ui, rank);
                if pending_action.is_some() {
                    break;
                }
                ui.add_space(10.0);
                continue;
            }

            let Some(entry) = self.data.zones.get(index).cloned() else {
                continue;
            };

            let recommended_class = recommended_zone_class(&self.data.zones, &entry.name);
            let mut edit_clicked = false;
            let mut request_delete = false;
            let mut confirm_delete = false;
            let mut cancel_delete = false;

            egui::Frame::none()
                .fill(colors.surface_alt)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .rounding(14.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    let compact_metrics = ui.available_width() < CARD_COMPACT_METRICS_BREAKPOINT;

                    render_ranked_card_header(
                        ui,
                        rank,
                        &entry.name,
                        "Zone",
                        format!("{} kamas/h", format_kamas(entry.kamas_per_hour)),
                        profit_color(entry.kamas_per_hour),
                    );

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        class_badge(
                            ui,
                            "Classe",
                            entry.character_class,
                            PreviewTone::Neutral,
                            &self.class_textures,
                        );

                        if let Some(recommended_class) = recommended_class {
                            recommendation_badge(
                                ui,
                                recommended_class,
                                entry.character_class,
                                &self.class_textures,
                            );
                        }
                    });

                    ui.add_space(8.0);
                    render_metric_row(
                        ui,
                        compact_metrics,
                        vec![
                            ("Date + heure", recorded_at_label(entry.recorded_at)),
                            ("Session", format_duration_hms(entry.session_time_seconds)),
                            (
                                "Valeur",
                                format!("{} kamas", format_kamas(entry.session_total_kamas)),
                            ),
                        ],
                    );

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        if secondary_button(ui, "Modifier").clicked() {
                            edit_clicked = true;
                        }

                        if self.zone_delete_confirm == Some(index) {
                            ui.label(
                                egui::RichText::new("Confirmer la suppression ?")
                                    .size(13.0)
                                    .color(colors.warning),
                            );
                            if danger_button(ui, "Confirmer").clicked() {
                                confirm_delete = true;
                            }
                            if secondary_button(ui, "Annuler").clicked() {
                                cancel_delete = true;
                            }
                        } else if danger_button(ui, "Supprimer").clicked() {
                            request_delete = true;
                        }
                    });
                });

            if edit_clicked {
                pending_action = Some(ZoneListAction::StartEdit(index));
            } else if request_delete {
                pending_action = Some(ZoneListAction::RequestDelete(index));
            } else if cancel_delete {
                pending_action = Some(ZoneListAction::CancelDelete(index));
            } else if confirm_delete {
                pending_action = Some(ZoneListAction::ConfirmDelete(index));
            }

            ui.add_space(8.0);

            if pending_action.is_some() {
                break;
            }
        }

        if let Some(action) = pending_action {
            self.apply_zone_list_action(action);
        }
    }
}

impl MyApp {
    fn render_dungeon_list(&mut self, ui: &mut egui::Ui) {
        list_toolbar(ui, &mut self.dungeon_search, "Rechercher un donjon");
        ui.add_space(10.0);

        let filtered: Vec<usize> = self
            .data
            .dungeons
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches_search(&entry.name, &self.dungeon_search))
            .map(|(index, _)| index)
            .collect();

        if filtered.is_empty() {
            empty_state(
                ui,
                self.data.dungeons.is_empty(),
                "Aucun donjon enregistré.",
                "Aucun donjon ne correspond à cette recherche.",
            );
            return;
        }

        let colors = theme::palette();
        let mut pending_action: Option<DungeonListAction> = None;

        for (rank, index) in filtered.into_iter().enumerate() {
            if self.dungeon_edit.as_ref().map(|edit| edit.index) == Some(index) {
                pending_action = self.render_dungeon_edit_card(ui, rank);
                if pending_action.is_some() {
                    break;
                }
                ui.add_space(10.0);
                continue;
            }

            let Some(entry) = self.data.dungeons.get(index).cloned() else {
                continue;
            };

            let recommended_class = recommended_dungeon_class(&self.data.dungeons, &entry.name);
            let mut edit_clicked = false;
            let mut request_delete = false;
            let mut confirm_delete = false;
            let mut cancel_delete = false;

            egui::Frame::none()
                .fill(colors.surface_alt)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .rounding(14.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    let _compact_metrics = ui.available_width() < CARD_COMPACT_METRICS_BREAKPOINT;

                    render_ranked_card_header(
                        ui,
                        rank,
                        &entry.name,
                        "Donjon",
                        format!("{} kamas/h", format_kamas(entry.kamas_per_hour)),
                        profit_color(entry.kamas_per_hour),
                    );

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        class_badge(
                            ui,
                            "Classe",
                            entry.character_class,
                            PreviewTone::Neutral,
                            &self.class_textures,
                        );

                        if let Some(recommended_class) = recommended_class {
                            recommendation_badge(
                                ui,
                                recommended_class,
                                entry.character_class,
                                &self.class_textures,
                            );
                        }
                    });

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        metric_badge(ui, "Date + heure", recorded_at_label(entry.recorded_at));
                        metric_badge(
                            ui,
                            "Temps",
                            format_duration_hms(entry.run_time_minutes * 60.0),
                        );
                        metric_badge(
                            ui,
                            "Brut / run",
                            format!("{} kamas", format_kamas(entry.gross_kamas_per_run)),
                        );
                        metric_badge(
                            ui,
                            "Clé",
                            format!("{} kamas", format_kamas(entry.key_price)),
                        );
                        metric_badge(
                            ui,
                            "Net / run",
                            format!("{} kamas", format_kamas(entry.net_kamas_per_run)),
                        );
                    });

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        if secondary_button(ui, "Modifier").clicked() {
                            edit_clicked = true;
                        }

                        if self.dungeon_delete_confirm == Some(index) {
                            ui.label(
                                egui::RichText::new("Confirmer la suppression ?")
                                    .size(13.0)
                                    .color(colors.warning),
                            );
                            if danger_button(ui, "Confirmer").clicked() {
                                confirm_delete = true;
                            }
                            if secondary_button(ui, "Annuler").clicked() {
                                cancel_delete = true;
                            }
                        } else if danger_button(ui, "Supprimer").clicked() {
                            request_delete = true;
                        }
                    });
                });

            if edit_clicked {
                pending_action = Some(DungeonListAction::StartEdit(index));
            } else if request_delete {
                pending_action = Some(DungeonListAction::RequestDelete(index));
            } else if cancel_delete {
                pending_action = Some(DungeonListAction::CancelDelete(index));
            } else if confirm_delete {
                pending_action = Some(DungeonListAction::ConfirmDelete(index));
            }

            ui.add_space(8.0);

            if pending_action.is_some() {
                break;
            }
        }

        if let Some(action) = pending_action {
            self.apply_dungeon_list_action(action);
        }
    }

    fn render_duo_trio_list(&mut self, ui: &mut egui::Ui) {
        list_toolbar(ui, &mut self.duo_trio_search, "Rechercher un run duo/trio");
        ui.add_space(10.0);

        let filtered: Vec<usize> = self
            .data
            .duo_trios
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches_search(&entry.name, &self.duo_trio_search))
            .map(|(index, _)| index)
            .collect();

        if filtered.is_empty() {
            empty_state(
                ui,
                self.data.duo_trios.is_empty(),
                "Aucun run duo/trio enregistre.",
                "Aucun run duo/trio ne correspond a cette recherche.",
            );
            return;
        }

        let colors = theme::palette();
        let mut pending_action: Option<DuoTrioListAction> = None;

        for (rank, index) in filtered.into_iter().enumerate() {
            if self.duo_trio_edit.as_ref().map(|edit| edit.index) == Some(index) {
                pending_action = self.render_duo_trio_edit_card(ui, rank);
                if pending_action.is_some() {
                    break;
                }
                ui.add_space(10.0);
                continue;
            }

            let Some(entry) = self.data.duo_trios.get(index).cloned() else {
                continue;
            };

            let recommended_class = recommended_duo_trio_class(&self.data.duo_trios, &entry.name);
            let mut edit_clicked = false;
            let mut request_delete = false;
            let mut confirm_delete = false;
            let mut cancel_delete = false;

            egui::Frame::none()
                .fill(colors.surface_alt)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .rounding(14.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    let compact_metrics = ui.available_width() < CARD_COMPACT_METRICS_BREAKPOINT;

                    render_ranked_card_header(
                        ui,
                        rank,
                        &entry.name,
                        entry.party_mode.label(),
                        format!("{} kamas/h", format_kamas(entry.kamas_per_hour)),
                        profit_color(entry.kamas_per_hour),
                    );

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        class_badge(
                            ui,
                            "Classe",
                            entry.character_class,
                            PreviewTone::Neutral,
                            &self.class_textures,
                        );

                        if let Some(recommended_class) = recommended_class {
                            recommendation_badge(
                                ui,
                                recommended_class,
                                entry.character_class,
                                &self.class_textures,
                            );
                        }
                    });

                    ui.add_space(8.0);
                    render_metric_row(
                        ui,
                        compact_metrics,
                        vec![
                            ("Date + heure", recorded_at_label(entry.recorded_at)),
                            ("Temps", format_duration_hms(entry.run_time_seconds)),
                            (
                                "Net",
                                format!("{} kamas", format_kamas(entry.net_kamas_per_run)),
                            ),
                        ],
                    );
                    ui.add_space(6.0);
                    render_metric_row(
                        ui,
                        compact_metrics,
                        vec![
                            (
                                "Loot",
                                format!("{} kamas", format_kamas(entry.loot_kamas_per_run)),
                            ),
                            (
                                "Capture",
                                format!("{} kamas", format_kamas(entry.full_soul_sale_price)),
                            ),
                            (
                                "Clefs",
                                format!(
                                    "{}x | {}",
                                    entry.keys_count,
                                    format_kamas(entry.total_key_cost)
                                ),
                            ),
                        ],
                    );

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        if secondary_button(ui, "Modifier").clicked() {
                            edit_clicked = true;
                        }

                        if self.duo_trio_delete_confirm == Some(index) {
                            ui.label(
                                egui::RichText::new("Confirmer la suppression ?")
                                    .size(13.0)
                                    .color(colors.warning),
                            );
                            if danger_button(ui, "Confirmer").clicked() {
                                confirm_delete = true;
                            }
                            if secondary_button(ui, "Annuler").clicked() {
                                cancel_delete = true;
                            }
                        } else if danger_button(ui, "Supprimer").clicked() {
                            request_delete = true;
                        }
                    });
                });

            if edit_clicked {
                pending_action = Some(DuoTrioListAction::StartEdit(index));
            } else if request_delete {
                pending_action = Some(DuoTrioListAction::RequestDelete(index));
            } else if cancel_delete {
                pending_action = Some(DuoTrioListAction::CancelDelete(index));
            } else if confirm_delete {
                pending_action = Some(DuoTrioListAction::ConfirmDelete(index));
            }

            ui.add_space(8.0);

            if pending_action.is_some() {
                break;
            }
        }

        if let Some(action) = pending_action {
            self.apply_duo_trio_list_action(action);
        }
    }

    fn render_arena_list(&mut self, ui: &mut egui::Ui) {
        list_toolbar(ui, &mut self.arena_search, "Rechercher une session");
        ui.add_space(10.0);

        let filtered: Vec<usize> = self
            .data
            .arenas
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches_search(&entry.name, &self.arena_search))
            .map(|(index, _)| index)
            .collect();

        if filtered.is_empty() {
            empty_state(
                ui,
                self.data.arenas.is_empty(),
                "Aucune session PL arène enregistrée.",
                "Aucune session ne correspond à cette recherche.",
            );
            return;
        }

        let colors = theme::palette();
        let mut pending_action: Option<ArenaListAction> = None;

        for (rank, index) in filtered.into_iter().enumerate() {
            if self.arena_edit.as_ref().map(|edit| edit.index) == Some(index) {
                pending_action = self.render_arena_edit_card(ui, rank);
                if pending_action.is_some() {
                    break;
                }
                ui.add_space(10.0);
                continue;
            }

            let Some(entry) = self.data.arenas.get(index).cloned() else {
                continue;
            };

            let recommended_class = recommended_arena_class(&self.data.arenas, &entry.name);
            let mut edit_clicked = false;
            let mut request_delete = false;
            let mut confirm_delete = false;
            let mut cancel_delete = false;

            egui::Frame::none()
                .fill(colors.surface_alt)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .rounding(14.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    let _compact_metrics = ui.available_width() < CARD_COMPACT_METRICS_BREAKPOINT;

                    render_ranked_card_header(
                        ui,
                        rank,
                        &entry.name,
                        "Session",
                        format!("{} kamas/h", format_kamas(entry.kamas_per_hour)),
                        profit_color(entry.kamas_per_hour),
                    );

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        class_badge(
                            ui,
                            "Classe",
                            entry.character_class,
                            PreviewTone::Neutral,
                            &self.class_textures,
                        );

                        if let Some(recommended_class) = recommended_class {
                            recommendation_badge(
                                ui,
                                recommended_class,
                                entry.character_class,
                                &self.class_textures,
                            );
                        }
                    });

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        metric_badge(ui, "Date + heure", recorded_at_label(entry.recorded_at));
                        metric_badge(
                            ui,
                            "Temps",
                            format_duration_hms(entry.round_time_minutes * 60.0),
                        );
                        metric_badge(
                            ui,
                            "Places",
                            format!(
                                "{} × {} kamas",
                                entry.seats_sold,
                                format_kamas(entry.seat_price)
                            ),
                        );
                        metric_badge(
                            ui,
                            "Captures",
                            format!(
                                "{} × {} kamas",
                                entry.captures_count,
                                format_kamas(entry.capture_price)
                            ),
                        );
                        metric_badge(
                            ui,
                            "Net",
                            format!("{} kamas", format_kamas(entry.net_profit)),
                        );
                    });

                    ui.add_space(8.0);
                    clamped_wrapped_row(ui, |ui| {
                        if secondary_button(ui, "Modifier").clicked() {
                            edit_clicked = true;
                        }

                        if self.arena_delete_confirm == Some(index) {
                            ui.label(
                                egui::RichText::new("Confirmer la suppression ?")
                                    .size(13.0)
                                    .color(colors.warning),
                            );
                            if danger_button(ui, "Confirmer").clicked() {
                                confirm_delete = true;
                            }
                            if secondary_button(ui, "Annuler").clicked() {
                                cancel_delete = true;
                            }
                        } else if danger_button(ui, "Supprimer").clicked() {
                            request_delete = true;
                        }
                    });
                });

            if edit_clicked {
                pending_action = Some(ArenaListAction::StartEdit(index));
            } else if request_delete {
                pending_action = Some(ArenaListAction::RequestDelete(index));
            } else if cancel_delete {
                pending_action = Some(ArenaListAction::CancelDelete(index));
            } else if confirm_delete {
                pending_action = Some(ArenaListAction::ConfirmDelete(index));
            }

            ui.add_space(8.0);

            if pending_action.is_some() {
                break;
            }
        }

        if let Some(action) = pending_action {
            self.apply_arena_list_action(action);
        }
    }

    fn render_zone_edit_card(&mut self, ui: &mut egui::Ui, rank: usize) -> Option<ZoneListAction> {
        let colors = theme::palette();
        let mut action = None;

        egui::Frame::none()
            .fill(colors.surface_alt)
            .stroke(egui::Stroke::new(1.0, colors.accent))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                rank_badge(ui, rank + 1);
                ui.add_space(6.0);

                if let Some(edit) = self.zone_edit.as_mut() {
                    ui.label(
                        egui::RichText::new("Modifier la zone")
                            .size(16.0)
                            .strong()
                            .color(colors.text_primary),
                    );

                    let mut has_focus = false;
                    let mut changed = false;

                    let response = form_field_row(
                        ui,
                        "Nom de la zone",
                        &mut edit.form.name,
                        "",
                        "Conservez un libellé court et reconnaissable.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    changed |= class_selector(
                        ui,
                        "Classe jouee",
                        &mut edit.form.character_class,
                        &self.class_textures,
                    );

                    let response = date_field_row(
                        ui,
                        "Date + heure de session",
                        &mut edit.form.recorded_at_input,
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = duration_input_row(
                        ui,
                        "zone-edit-session-time",
                        "Temps de la session",
                        &mut edit.form.session_time,
                    );
                    has_focus |= response.has_focus;
                    changed |= response.changed;

                    let response = form_field_row(
                        ui,
                        "Valeur totale mise en vente",
                        &mut edit.form.session_total_kamas,
                        "kamas",
                        "Valeur brute estimee de la session.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    if changed {
                        edit.error = None;
                    }

                    ui.add_space(8.0);
                    preview_card(ui, "Prévision mise à jour", preview_zone(&edit.form));

                    if let Some(error) = edit.error.as_deref() {
                        ui.add_space(8.0);
                        local_error_box(ui, error);
                    }

                    let submit_with_enter =
                        has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    ui.add_space(8.0);
                    if wide_primary_button(ui, "Enregistrer").clicked() || submit_with_enter {
                        action = Some(ZoneListAction::SaveEdit);
                    }
                    if action.is_none() && secondary_button(ui, "Annuler").clicked() {
                        action = Some(ZoneListAction::CancelEdit);
                    }
                }
            });

        action
    }
}

impl MyApp {
    fn render_dungeon_edit_card(
        &mut self,
        ui: &mut egui::Ui,
        rank: usize,
    ) -> Option<DungeonListAction> {
        let colors = theme::palette();
        let mut action = None;

        egui::Frame::none()
            .fill(colors.surface_alt)
            .stroke(egui::Stroke::new(1.0, colors.accent))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                rank_badge(ui, rank + 1);
                ui.add_space(6.0);

                if let Some(edit) = self.dungeon_edit.as_mut() {
                    ui.label(
                        egui::RichText::new("Modifier le donjon")
                            .size(16.0)
                            .strong()
                            .color(colors.text_primary),
                    );

                    let mut has_focus = false;
                    let mut changed = false;

                    let response = form_field_row(
                        ui,
                        "Nom du donjon",
                        &mut edit.form.name,
                        "",
                        "Libellé affiché dans le classement.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    changed |= class_selector(
                        ui,
                        "Classe jouee",
                        &mut edit.form.character_class,
                        &self.class_textures,
                    );

                    let response =
                        date_field_row(ui, "Date + heure du run", &mut edit.form.recorded_at_input);
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = duration_input_row(
                        ui,
                        "dungeon-edit-run-time",
                        "Temps moyen du run",
                        &mut edit.form.run_time,
                    );
                    has_focus |= response.has_focus;
                    changed |= response.changed;

                    let response = form_field_row(
                        ui,
                        "Gain brut par run",
                        &mut edit.form.gross_kamas_per_run,
                        "kamas",
                        "Revenu total avant coût de la clé.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Prix de la clé",
                        &mut edit.form.key_price,
                        "kamas",
                        "Coût à déduire de chaque run.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    if changed {
                        edit.error = None;
                    }

                    ui.add_space(8.0);
                    preview_card(ui, "Prévision mise à jour", preview_dungeon(&edit.form));

                    if let Some(error) = edit.error.as_deref() {
                        ui.add_space(8.0);
                        local_error_box(ui, error);
                    }

                    let submit_with_enter =
                        has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    ui.add_space(8.0);
                    if wide_primary_button(ui, "Enregistrer").clicked() || submit_with_enter {
                        action = Some(DungeonListAction::SaveEdit);
                    }
                    if action.is_none() && secondary_button(ui, "Annuler").clicked() {
                        action = Some(DungeonListAction::CancelEdit);
                    }
                }
            });

        action
    }

    fn render_duo_trio_edit_card(
        &mut self,
        ui: &mut egui::Ui,
        rank: usize,
    ) -> Option<DuoTrioListAction> {
        let colors = theme::palette();
        let mut action = None;

        egui::Frame::none()
            .fill(colors.surface_alt)
            .stroke(egui::Stroke::new(1.0, colors.accent))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                rank_badge(ui, rank + 1);
                ui.add_space(6.0);

                if let Some(edit) = self.duo_trio_edit.as_mut() {
                    ui.label(
                        egui::RichText::new("Modifier le run duo/trio")
                            .size(16.0)
                            .strong()
                            .color(colors.text_primary),
                    );

                    let mut has_focus = false;
                    let mut changed = false;

                    let response = form_field_row(
                        ui,
                        "Nom / reference",
                        &mut edit.form.name,
                        "",
                        "Nom du boss ou reference du run.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    changed |= class_selector(
                        ui,
                        "Classe jouee",
                        &mut edit.form.character_class,
                        &self.class_textures,
                    );

                    let response =
                        date_field_row(ui, "Date + heure du run", &mut edit.form.recorded_at_input);
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    changed |= party_mode_selector(ui, "Mode", &mut edit.form.party_mode);

                    let response = duration_input_row(
                        ui,
                        "duo-trio-edit-run-time",
                        "Temps du run",
                        &mut edit.form.run_time,
                    );
                    has_focus |= response.has_focus;
                    changed |= response.changed;

                    let response = form_field_row(
                        ui,
                        "Loot total du run",
                        &mut edit.form.loot_kamas_per_run,
                        "kamas",
                        "Valeur du loot hors capture pleine.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Prix unitaire d'une clef",
                        &mut edit.form.key_unit_price,
                        "kamas",
                        "Le total suit automatiquement le mode duo ou trio.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Prix de la pierre de capture",
                        &mut edit.form.capture_stone_price,
                        "kamas",
                        "Une pierre par run.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Prix de vente de la capture pleine",
                        &mut edit.form.full_soul_sale_price,
                        "kamas",
                        "Valeur de revente de la pierre pleine.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    if changed {
                        edit.error = None;
                    }

                    ui.add_space(8.0);
                    preview_card(ui, "Prevision mise a jour", preview_duo_trio(&edit.form));

                    if let Some(error) = edit.error.as_deref() {
                        ui.add_space(8.0);
                        local_error_box(ui, error);
                    }

                    let submit_with_enter =
                        has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    ui.add_space(8.0);
                    if wide_primary_button(ui, "Enregistrer").clicked() || submit_with_enter {
                        action = Some(DuoTrioListAction::SaveEdit);
                    }
                    if action.is_none() && secondary_button(ui, "Annuler").clicked() {
                        action = Some(DuoTrioListAction::CancelEdit);
                    }
                }
            });

        action
    }

    fn render_arena_edit_card(
        &mut self,
        ui: &mut egui::Ui,
        rank: usize,
    ) -> Option<ArenaListAction> {
        let colors = theme::palette();
        let mut action = None;

        egui::Frame::none()
            .fill(colors.surface_alt)
            .stroke(egui::Stroke::new(1.0, colors.accent))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                rank_badge(ui, rank + 1);
                ui.add_space(6.0);

                if let Some(edit) = self.arena_edit.as_mut() {
                    ui.label(
                        egui::RichText::new("Modifier la session PL arène")
                            .size(16.0)
                            .strong()
                            .color(colors.text_primary),
                    );

                    let mut has_focus = false;
                    let mut changed = false;

                    let response = form_field_row(
                        ui,
                        "Nom / référence",
                        &mut edit.form.name,
                        "",
                        "Nom du boss, session ou référence interne.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    changed |= class_selector(
                        ui,
                        "Classe jouee",
                        &mut edit.form.character_class,
                        &self.class_textures,
                    );

                    let response = date_field_row(
                        ui,
                        "Date + heure de session",
                        &mut edit.form.recorded_at_input,
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = duration_input_row(
                        ui,
                        "arena-edit-round-time",
                        "Temps de la ronde",
                        &mut edit.form.round_time,
                    );
                    has_focus |= response.has_focus;
                    changed |= response.changed;

                    let response = form_field_row(
                        ui,
                        "Prix d'une place",
                        &mut edit.form.seat_price,
                        "kamas",
                        "Montant vendu par siège.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Places vendues",
                        &mut edit.form.seats_sold,
                        "",
                        "Nombre réel de places vendues.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Prix d'une capture",
                        &mut edit.form.capture_price,
                        "kamas",
                        "Coût unitaire d'une capture.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    let response = form_field_row(
                        ui,
                        "Nombre de captures",
                        &mut edit.form.captures_count,
                        "",
                        "Quantité consommée sur la ronde.",
                    );
                    has_focus |= response.has_focus();
                    changed |= response.changed();

                    if changed {
                        edit.error = None;
                    }

                    ui.add_space(8.0);
                    preview_card(ui, "Prévision mise à jour", preview_arena(&edit.form));

                    if let Some(error) = edit.error.as_deref() {
                        ui.add_space(8.0);
                        local_error_box(ui, error);
                    }

                    let submit_with_enter =
                        has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    ui.add_space(8.0);
                    if wide_primary_button(ui, "Enregistrer").clicked() || submit_with_enter {
                        action = Some(ArenaListAction::SaveEdit);
                    }
                    if action.is_none() && secondary_button(ui, "Annuler").clicked() {
                        action = Some(ArenaListAction::CancelEdit);
                    }
                }
            });

        action
    }
}

fn section_frame(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    add_content: impl FnOnce(&mut egui::Ui),
) {
    let colors = theme::palette();

    egui::Frame::none()
        .fill(colors.surface)
        .stroke(egui::Stroke::new(1.0, colors.border))
        .rounding(14.0)
        .inner_margin(egui::Margin::same(14.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(18.0)
                    .strong()
                    .color(colors.text_primary),
            );
            if !subtitle.is_empty() {
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(11.0)
                        .color(colors.text_secondary),
                );
            }
            ui.add_space(8.0);
            clamped_content(ui, add_content);
        });
}

fn paint_background(ctx: &egui::Context, texture: Option<&egui::TextureHandle>) {
    let rect = ctx.screen_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());

    if let Some(texture) = texture {
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(8, 11, 14, 176),
    );
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(18, 22, 27, 88),
    );
}

fn render_brand_title(ui: &mut egui::Ui, title: &str) {
    let time = ui.input(|input| input.time) as f32;
    let border_color = with_alpha(brand_title_color(time, 0.18), 170);
    let fill_color = with_alpha(brand_title_color(time, 0.62), 28);

    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(33));

    egui::Frame::none()
        .fill(fill_color)
        .stroke(egui::Stroke::new(1.0, border_color))
        .rounding(16.0)
        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
        .show(ui, |ui| {
            let galley = ui
                .painter()
                .layout_job(build_brand_title_job(ui, title, time));
            let desired_size = galley.size() + egui::vec2(0.0, 8.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
            let text_pos = rect.left_top();
            let painter = ui.painter();

            painter.galley_with_override_text_color(
                text_pos + egui::vec2(0.0, 2.0),
                galley.clone(),
                egui::Color32::from_rgba_unmultiplied(6, 8, 12, 190),
            );
            painter.galley(text_pos, galley, egui::Color32::WHITE);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 2.0, rect.bottom() - 1.0),
                    egui::pos2(rect.right() - 2.0, rect.bottom() - 1.0),
                ],
                egui::Stroke::new(2.0, brand_title_color(time, 0.48)),
            );
        });
}

fn build_brand_title_job(ui: &egui::Ui, title: &str, time: f32) -> egui::text::LayoutJob {
    let mut font_id = egui::TextStyle::Heading.resolve(ui.style());
    font_id.size = HEADER_BRAND_TITLE_SIZE;

    let mut job = egui::text::LayoutJob::default();

    for (index, character) in title.chars().enumerate() {
        egui::RichText::new(character.to_string())
            .font(font_id.clone())
            .strong()
            .color(brand_title_color(
                time,
                index as f32 * HEADER_BRAND_CHARACTER_OFFSET,
            ))
            .append_to(
                &mut job,
                ui.style().as_ref(),
                egui::FontSelection::Default,
                egui::Align::Center,
            );
    }

    job
}

fn brand_title_color(time: f32, phase_offset: f32) -> egui::Color32 {
    let palette = [
        egui::Color32::from_rgb(89, 168, 255),
        egui::Color32::from_rgb(155, 102, 255),
        egui::Color32::from_rgb(255, 94, 107),
        egui::Color32::from_rgb(255, 213, 79),
    ];
    let palette_len = palette.len() as f32;
    let progress = (time * HEADER_BRAND_ANIMATION_SPEED + phase_offset).rem_euclid(palette_len);
    let current = progress.floor() as usize;
    let next = (current + 1) % palette.len();
    let blend = smoothstep(progress.fract());

    lerp_color(palette[current], palette[next], blend)
}

fn lerp_color(start: egui::Color32, end: egui::Color32, t: f32) -> egui::Color32 {
    let mix = |from: u8, to: u8| -> u8 {
        (from as f32 + (to as f32 - from as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };

    egui::Color32::from_rgba_unmultiplied(
        mix(start.r(), end.r()),
        mix(start.g(), end.g()),
        mix(start.b(), end.b()),
        mix(start.a(), end.a()),
    )
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn with_alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn themed_button_widget(
    label: &str,
    fill: egui::Color32,
    stroke: egui::Color32,
    text: egui::Color32,
) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(label).size(13.0).strong().color(text))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .rounding(10.0)
}

fn themed_button(
    ui: &mut egui::Ui,
    label: &str,
    fill: egui::Color32,
    stroke: egui::Color32,
    text: egui::Color32,
) -> egui::Response {
    ui.add(themed_button_widget(label, fill, stroke, text))
}

fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let colors = theme::palette();
    themed_button(ui, label, colors.accent_soft, colors.accent, colors.accent)
}

fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let colors = theme::palette();
    themed_button(
        ui,
        label,
        colors.surface_alt,
        colors.border,
        colors.text_primary,
    )
}

fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let colors = theme::palette();
    themed_button(ui, label, colors.danger_soft, colors.danger, colors.danger)
}

fn wide_primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let colors = theme::palette();
    ui.add_sized(
        [ui.available_width(), 34.0],
        themed_button_widget(label, colors.accent_soft, colors.accent, colors.accent),
    )
}

fn dialog_text_input(ui: &mut egui::Ui, value: &mut String, hint: &str) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), 32.0],
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .margin(egui::vec2(10.0, 7.0)),
    )
}

fn format_named_save_updated_at(updated_at: chrono::DateTime<chrono::Utc>) -> String {
    updated_at
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

fn status_banner(ui: &mut egui::Ui, status: &StatusBanner) -> bool {
    let colors = theme::palette();
    let (fill, stroke, text) = match status.kind {
        StatusKind::Success => (colors.positive_soft, colors.positive, colors.positive),
        StatusKind::Error => (colors.danger_soft, colors.danger, colors.danger),
        StatusKind::Info => (colors.accent_soft, colors.accent, colors.accent),
    };

    let mut close_clicked = false;

    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .rounding(10.0)
        .inner_margin(egui::Margin::symmetric(10.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&status.message).size(12.0).color(text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Fermer")
                                    .size(11.0)
                                    .color(colors.text_primary),
                            )
                            .fill(fill)
                            .stroke(egui::Stroke::new(1.0, stroke))
                            .rounding(8.0),
                        )
                        .clicked()
                    {
                        close_clicked = true;
                    }
                });
            });
        });

    close_clicked
}

fn list_toolbar(ui: &mut egui::Ui, search: &mut String, hint: &str) {
    let clear_width = if search.is_empty() { 0.0 } else { 82.0 };
    let input_width = (ui.available_width() - clear_width).max(140.0);

    ui.horizontal(|ui| {
        ui.add_sized(
            [input_width, 30.0],
            egui::TextEdit::singleline(search)
                .hint_text(hint)
                .margin(egui::vec2(10.0, 6.0)),
        );

        if !search.is_empty() {
            let response = secondary_button(ui, "Effacer");
            if response.clicked() {
                search.clear();
            }
        }
    });
}

fn rank_badge(ui: &mut egui::Ui, rank: usize) {
    let colors = theme::palette();

    egui::Frame::none()
        .fill(colors.accent_soft)
        .stroke(egui::Stroke::new(1.0, colors.accent))
        .rounding(999.0)
        .inner_margin(egui::Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("#{rank}"))
                    .size(11.0)
                    .strong()
                    .color(colors.accent),
            );
        });
}

fn render_ranked_card_header(
    ui: &mut egui::Ui,
    rank: usize,
    title: &str,
    subtitle: &str,
    value: String,
    value_color: egui::Color32,
) {
    let colors = theme::palette();
    let value_label = || {
        egui::Label::new(
            egui::RichText::new(value.clone())
                .monospace()
                .size(18.0)
                .strong()
                .color(value_color),
        )
        .wrap(true)
    };

    if ui.available_width() < CARD_HEADER_STACK_BREAKPOINT {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                rank_badge(ui, rank + 1);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(16.0)
                            .strong()
                            .color(colors.text_primary),
                    );
                    ui.label(
                        egui::RichText::new(subtitle)
                            .size(11.0)
                            .color(colors.text_secondary),
                    );
                });
            });
            ui.add_space(6.0);
            ui.add(value_label());
        });
        return;
    }

    ui.horizontal_top(|ui| {
        rank_badge(ui, rank + 1);
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(16.0)
                    .strong()
                    .color(colors.text_primary),
            );
            ui.label(
                egui::RichText::new(subtitle)
                    .size(11.0)
                    .color(colors.text_secondary),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            ui.add(value_label());
        });
    });
}

fn preview_card_sized(ui: &mut egui::Ui, title: &str, preview: &PreviewBlock, width: f32) {
    let colors = theme::palette();
    let (fill, stroke, text) = tone_colors(preview.tone);
    let width = width.min(ui.available_width());

    clamped_content(ui, |ui| {
        ui.set_width(width);
        ui.set_max_width(width);

        egui::Frame::none()
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, stroke))
            .rounding(12.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_max_width(ui.available_width());

                ui.label(
                    egui::RichText::new(title)
                        .size(11.0)
                        .color(colors.text_secondary),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(preview.value.clone())
                            .monospace()
                            .size(18.0)
                            .strong()
                            .color(text),
                    )
                    .wrap(true),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(preview.detail.clone())
                            .size(11.0)
                            .color(colors.text_secondary),
                    )
                    .wrap(true),
                )
                .on_hover_text(preview.detail.clone());
            });
    });
}

fn preview_card(ui: &mut egui::Ui, title: &str, preview: PreviewBlock) {
    let width = ui.available_width();
    preview_card_sized(ui, title, &preview, width);
}

fn local_error_box(ui: &mut egui::Ui, message: &str) {
    let colors = theme::palette();

    egui::Frame::none()
        .fill(colors.danger_soft)
        .stroke(egui::Stroke::new(1.0, colors.danger))
        .rounding(10.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(message).size(12.0).color(colors.danger));
        });
}

fn empty_state(ui: &mut egui::Ui, no_data: bool, empty_message: &str, search_message: &str) {
    let colors = theme::palette();
    let message = if no_data {
        empty_message
    } else {
        search_message
    };

    egui::Frame::none()
        .fill(colors.surface_alt)
        .stroke(egui::Stroke::new(1.0, colors.border))
        .rounding(12.0)
        .inner_margin(egui::Margin::same(14.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(message)
                    .size(13.0)
                    .color(colors.text_secondary),
            );
        });
}

fn table_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        egui::RichText::new(label)
            .size(11.0)
            .strong()
            .color(theme::palette().text_secondary),
    );
}

fn muted_text(ui: &mut egui::Ui, label: &str) {
    ui.label(
        egui::RichText::new(label)
            .size(12.0)
            .color(theme::palette().text_secondary),
    );
}

fn recorded_at_label(value: Option<NaiveDateTime>) -> String {
    let formatted = format_recorded_at(value);

    if formatted.is_empty() {
        "Horodatage inconnu".to_string()
    } else {
        formatted
    }
}

fn datetime_to_plot_x(value: NaiveDateTime) -> f64 {
    value.and_utc().timestamp() as f64
}

fn plot_x_to_datetime(value: f64) -> Option<NaiveDateTime> {
    chrono::DateTime::<chrono::Utc>::from_timestamp(value.round() as i64, 0)
        .map(|value| value.naive_utc())
}

fn build_cumulative_line_points(
    series: &CategorySeries,
    period_started_at: Option<NaiveDateTime>,
) -> Vec<[f64; 2]> {
    let mut line_points =
        Vec::with_capacity(series.points.len() + usize::from(period_started_at.is_some()));

    if let Some(start_at) = period_started_at {
        line_points.push([datetime_to_plot_x(start_at), 0.0]);
    }

    line_points.extend(series.points.iter().map(|point| {
        [
            datetime_to_plot_x(point.recorded_at),
            point.cumulative_value as f64,
        ]
    }));

    line_points
}

fn cumulative_chart_x_bounds(summary: &ReportSummary) -> Option<(f64, f64)> {
    let mut first_visible_at: Option<NaiveDateTime> = None;
    let mut last_visible_at: Option<NaiveDateTime> = None;

    for recorded_at in summary
        .chart_series
        .iter()
        .flat_map(|series| series.points.iter().map(|point| point.recorded_at))
    {
        first_visible_at =
            Some(first_visible_at.map_or(recorded_at, |current| current.min(recorded_at)));
        last_visible_at =
            Some(last_visible_at.map_or(recorded_at, |current| current.max(recorded_at)));
    }

    let first_visible_at = first_visible_at?;
    let min_at = summary.period_started_at.unwrap_or(first_visible_at);
    let mut max_at = last_visible_at?;

    if max_at <= min_at {
        max_at = min_at + chrono::Duration::seconds(60);
    }

    Some((datetime_to_plot_x(min_at), datetime_to_plot_x(max_at)))
}

fn format_plot_time_mark(value: f64, range: &std::ops::RangeInclusive<f64>) -> String {
    let Some(datetime) = plot_x_to_datetime(value) else {
        return String::new();
    };

    let span_hours = ((*range.end() - *range.start()).abs() / 3600.0).max(1.0);

    if span_hours <= 36.0 {
        datetime.format("%d/%m %H:%M").to_string()
    } else if span_hours <= 24.0 * 45.0 {
        datetime.format("%d/%m").to_string()
    } else {
        datetime.format("%m/%Y").to_string()
    }
}

fn format_plot_kamas_mark(value: f64) -> String {
    format_kamas(value as f32)
}

fn build_category_bar_chart(series: &CategoryBarSeries) -> BarChart {
    let (fill, stroke, _text) = activity_kind_colors(series.kind);
    let bars = series
        .segments
        .iter()
        .filter_map(|segment| build_bar_from_segment(segment, fill, stroke))
        .collect::<Vec<_>>();

    BarChart::new(bars)
        .name(series.kind.label())
        .id(egui::Id::new(("report_bar_chart", series.kind)))
        .element_formatter(Box::new(|_, _| String::new()))
}

fn build_bar_from_segment(
    segment: &SessionBarSegment,
    fill: egui::Color32,
    stroke_color: egui::Color32,
) -> Option<Bar> {
    let start_x = datetime_to_plot_x(segment.segment_start_at);
    let end_x = datetime_to_plot_x(segment.segment_end_at);
    let width = (end_x - start_x).abs();

    if width <= f64::EPSILON || segment.value.abs() <= f32::EPSILON {
        return None;
    }

    Some(
        Bar::new((start_x + end_x) / 2.0, segment.value as f64)
            .name(segment.name.clone())
            .width(width)
            .base_offset(segment.base_offset as f64)
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, stroke_color)),
    )
}

fn find_hovered_bar_segment<'a>(
    summary: &'a ReportSummary,
    transform: &egui_plot::PlotTransform,
    pointer_pos: egui::Pos2,
) -> Option<&'a SessionBarSegment> {
    let mut exact_match: Option<(f32, &'a SessionBarSegment)> = None;
    let mut nearby_match: Option<(f32, &'a SessionBarSegment)> = None;

    for segment in summary
        .chart_bars
        .iter()
        .flat_map(|series| series.segments.iter())
    {
        let rect = bar_segment_rect(transform, segment);
        let distance = rect.center().distance_sq(pointer_pos);

        if rect.contains(pointer_pos) {
            match exact_match {
                Some((best_distance, _)) if best_distance <= distance => {}
                _ => exact_match = Some((distance, segment)),
            }
        } else if rect.expand2(egui::vec2(4.0, 4.0)).contains(pointer_pos) {
            match nearby_match {
                Some((best_distance, _)) if best_distance <= distance => {}
                _ => nearby_match = Some((distance, segment)),
            }
        }
    }

    exact_match.or(nearby_match).map(|(_, segment)| segment)
}

fn bar_segment_rect(
    transform: &egui_plot::PlotTransform,
    segment: &SessionBarSegment,
) -> egui::Rect {
    let start_x = datetime_to_plot_x(segment.segment_start_at);
    let end_x = datetime_to_plot_x(segment.segment_end_at);
    let lower = segment.base_offset.min(segment.base_offset + segment.value) as f64;
    let upper = segment.base_offset.max(segment.base_offset + segment.value) as f64;

    transform.rect_from_values(
        &PlotPoint::new(start_x, lower),
        &PlotPoint::new(end_x, upper),
    )
}

fn render_tooltip_field(
    ui: &mut egui::Ui,
    label: &str,
    value: String,
    value_color: egui::Color32,
    monospace: bool,
) {
    let colors = theme::palette();

    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(12.0)
                .strong()
                .color(colors.text_secondary),
        );

        let mut value_text = egui::RichText::new(value).size(13.0).color(value_color);
        if monospace {
            value_text = value_text.monospace();
        }
        ui.label(value_text);
    });
}

fn render_activity_tooltip_content(
    ui: &mut egui::Ui,
    kind: ActivityKind,
    name: &str,
    duration_seconds: f32,
    delta_value: f32,
    kamas_per_hour: f32,
) {
    let colors = theme::palette();

    render_tooltip_field(
        ui,
        &format!("{} :", kind.singular_label()),
        name.to_string(),
        colors.text_primary,
        false,
    );
    render_tooltip_field(
        ui,
        "Temps :",
        format_duration_hms(duration_seconds),
        colors.text_primary,
        true,
    );
    render_tooltip_field(
        ui,
        "Kamas gagnes net :",
        format!("{} kamas", format_kamas(delta_value)),
        profit_color(delta_value),
        true,
    );
    render_tooltip_field(
        ui,
        "Ratio Kamas/heure :",
        format!("{} kamas/h", format_kamas(kamas_per_hour)),
        profit_color(kamas_per_hour),
        true,
    );
}

fn render_cumulative_point_tooltip(ui: &mut egui::Ui, point: &crate::reports::CumulativePoint) {
    ui.set_min_width(320.0);

    for (index, session) in point.sessions.iter().enumerate() {
        if index > 0 {
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
        }

        render_activity_tooltip_content(
            ui,
            session.kind,
            &session.name,
            session.duration_seconds,
            session.delta_value,
            session.kamas_per_hour,
        );
    }
}

fn render_bar_segment_tooltip(ui: &mut egui::Ui, segment: &SessionBarSegment) {
    ui.set_min_width(320.0);
    render_activity_tooltip_content(
        ui,
        segment.kind,
        &segment.name,
        segment.session_duration_seconds,
        segment.value,
        segment.kamas_per_hour,
    );
}

fn tone_colors(tone: PreviewTone) -> (egui::Color32, egui::Color32, egui::Color32) {
    let colors = theme::palette();

    match tone {
        PreviewTone::Accent => (colors.accent_soft, colors.accent, colors.accent),
        PreviewTone::Positive => (colors.positive_soft, colors.positive, colors.positive),
        PreviewTone::Danger => (colors.danger_soft, colors.danger, colors.danger),
        PreviewTone::Neutral => (
            colors.surface_alt,
            colors.border_strong,
            colors.text_primary,
        ),
    }
}

fn profit_color(value: f32) -> egui::Color32 {
    if value > 0.0 {
        theme::palette().positive
    } else {
        theme::palette().danger
    }
}

fn preview_zone(form: &ZoneForm) -> PreviewBlock {
    let Ok(session_time_seconds) = parse_duration_input(&form.session_time) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS et la valeur totale de session.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(session_total_kamas) = parse_f32(&form.session_total_kamas) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS et la valeur totale de session.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let kamas_per_hour = session_total_kamas * (3600.0 / session_time_seconds);

    PreviewBlock {
        value: format!("{} kamas/h", format_kamas(kamas_per_hour)),
        detail: format!(
            "{} de session | {} kamas de valeur",
            format_duration_hms(session_time_seconds),
            format_kamas(session_total_kamas)
        ),
        tone: if kamas_per_hour > 0.0 {
            PreviewTone::Positive
        } else {
            PreviewTone::Danger
        },
    }
}

fn preview_dungeon(form: &DungeonForm) -> PreviewBlock {
    let Ok(run_time_seconds) = parse_duration_input(&form.run_time) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le gain brut et le prix de la cle.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(gross_kamas_per_run) = parse_f32(&form.gross_kamas_per_run) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le gain brut et le prix de la cle.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(key_price) = parse_f32(&form.key_price) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le gain brut et le prix de la cle.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let run_time_minutes = seconds_to_minutes(run_time_seconds);
    let net_kamas_per_run = gross_kamas_per_run - key_price;
    let kamas_per_hour = net_kamas_per_run * (60.0 / run_time_minutes);

    PreviewBlock {
        value: format!("{} kamas/h", format_kamas(kamas_per_hour)),
        detail: format!(
            "{} | Net / run : {} kamas | Cout de cle : {} kamas",
            format_duration_hms(run_time_seconds),
            format_kamas(net_kamas_per_run),
            format_kamas(key_price)
        ),
        tone: if kamas_per_hour > 0.0 {
            PreviewTone::Positive
        } else {
            PreviewTone::Danger
        },
    }
}

fn preview_duo_trio(form: &DuoTrioForm) -> PreviewBlock {
    let Ok(run_time_seconds) = parse_duration_input(&form.run_time) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le loot, la capture pleine, la pierre et la clef."
                .to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(loot_kamas_per_run) = parse_f32(&form.loot_kamas_per_run) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le loot, la capture pleine, la pierre et la clef."
                .to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(capture_stone_price) = parse_f32(&form.capture_stone_price) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le loot, la capture pleine, la pierre et la clef."
                .to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(key_unit_price) = parse_f32(&form.key_unit_price) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le loot, la capture pleine, la pierre et la clef."
                .to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(full_soul_sale_price) = parse_f32(&form.full_soul_sale_price) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, le loot, la capture pleine, la pierre et la clef."
                .to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let keys_count = form.party_mode.player_count();
    let total_key_cost = key_unit_price * keys_count as f32;
    let gross_kamas_per_run = loot_kamas_per_run + full_soul_sale_price;
    let total_cost = capture_stone_price + total_key_cost;
    let net_kamas_per_run = gross_kamas_per_run - total_cost;
    let kamas_per_hour = net_kamas_per_run * (3600.0 / run_time_seconds);

    PreviewBlock {
        value: format!("{} kamas/h", format_kamas(kamas_per_hour)),
        detail: format!(
            "{} | Net / run : {} kamas | Cout total : {} kamas | Clefs : {} ({} kamas)",
            format_duration_hms(run_time_seconds),
            format_kamas(net_kamas_per_run),
            format_kamas(total_cost),
            keys_count,
            format_kamas(total_key_cost)
        ),
        tone: if kamas_per_hour > 0.0 {
            PreviewTone::Positive
        } else {
            PreviewTone::Danger
        },
    }
}

fn preview_arena(form: &ArenaForm) -> PreviewBlock {
    let Ok(round_time_seconds) = parse_duration_input(&form.round_time) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, les places vendues et le cout des captures.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(seat_price) = parse_f32(&form.seat_price) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, les places vendues et le cout des captures.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(seats_sold) = parse_u32(&form.seats_sold) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, les places vendues et le cout des captures.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(capture_price) = parse_f32(&form.capture_price) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, les places vendues et le cout des captures.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let Some(captures_count) = parse_u32(&form.captures_count) else {
        return PreviewBlock {
            value: "Calcul en attente".to_string(),
            detail: "Renseignez HH MM SS, les places vendues et le cout des captures.".to_string(),
            tone: PreviewTone::Neutral,
        };
    };

    let round_time_minutes = seconds_to_minutes(round_time_seconds);
    let gross_revenue = seat_price * seats_sold as f32;
    let total_capture_cost = capture_price * captures_count as f32;
    let net_profit = gross_revenue - total_capture_cost;
    let kamas_per_hour = net_profit * (60.0 / round_time_minutes);

    PreviewBlock {
        value: format!("{} kamas/h", format_kamas(kamas_per_hour)),
        detail: format!(
            "{} | Net : {} kamas | Places : {} | Captures : {}",
            format_duration_hms(round_time_seconds),
            format_kamas(net_profit),
            seats_sold,
            captures_count
        ),
        tone: if kamas_per_hour > 0.0 {
            PreviewTone::Positive
        } else {
            PreviewTone::Danger
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DofusClass, DurationInput};
    use crate::reports::{CategorySeries, CumulativePoint, ReportSummary};
    use crate::InlineEditState;
    use chrono::NaiveDateTime;

    fn dt(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").unwrap()
    }

    fn cumulative_point(recorded_at: &str, cumulative_value: f32) -> CumulativePoint {
        CumulativePoint {
            recorded_at: dt(recorded_at),
            delta_value: cumulative_value,
            cumulative_value,
            sessions: Vec::new(),
        }
    }

    fn summary_with_chart(
        period_started_at: Option<NaiveDateTime>,
        chart_series: Vec<CategorySeries>,
    ) -> ReportSummary {
        ReportSummary {
            session_count: chart_series.iter().map(|series| series.session_count).sum(),
            total_earned: chart_series
                .iter()
                .map(|series| series.total_in_period)
                .sum(),
            average_per_session: None,
            best_session: None,
            best_activity: None,
            top_activities: Vec::new(),
            class_summaries: Vec::new(),
            recent_sessions: Vec::new(),
            chart_series,
            chart_bars: Vec::new(),
            period_started_at,
        }
    }

    #[test]
    fn summarize_values_returns_best_average_and_count() {
        let summary = summarize_values([120_000.0, 240_000.0, 360_000.0].into_iter());

        assert_eq!(summary.count, 3);
        assert_eq!(summary.best, Some(360_000.0));
        assert_eq!(summary.average, Some(240_000.0));
    }

    #[test]
    fn search_is_case_and_accent_insensitive() {
        assert!(matches_search("Scarafeuilles", "scar"));
        assert!(matches_search("Scarafeuilles", "FEUIL"));
        assert!(matches_search("Cimetière des Torturés", "cimetiere"));
        assert!(matches_search("Qu'Tan", "qutan"));
        assert!(matches_search("Scarafeuilles", ""));
        assert!(!matches_search("Scarafeuilles", "dragon"));
    }

    #[test]
    fn duration_input_defaults_to_zero_selector_values() {
        assert_eq!(
            DurationInput::default(),
            DurationInput {
                hours: "00".to_string(),
                minutes: "00".to_string(),
                seconds: "00".to_string(),
            }
        );
    }

    #[test]
    fn normalize_duration_input_for_selector_clamps_invalid_values() {
        let mut input = DurationInput {
            hours: "29".to_string(),
            minutes: "70".to_string(),
            seconds: "abc".to_string(),
        };

        normalize_duration_input_for_selector(&mut input);

        assert_eq!(
            input,
            DurationInput {
                hours: "24".to_string(),
                minutes: "59".to_string(),
                seconds: "00".to_string(),
            }
        );
    }

    #[test]
    fn optional_duration_input_ignores_untouched_zero_filter() {
        assert_eq!(
            optional_duration_input_seconds(&DurationInput::default(), false),
            Ok(None)
        );
    }

    #[test]
    fn optional_duration_input_rejects_touched_zero_filter() {
        assert_eq!(
            optional_duration_input_seconds(&DurationInput::default(), true),
            Err("La duree doit etre superieure a 00:00:00.".to_string())
        );
    }

    #[test]
    fn activity_search_time_validation_waits_for_interaction() {
        let mut app = MyApp::default();

        assert_eq!(app.activity_search_available_time_seconds(), None);
        assert!(app.activity_search_time_error.is_none());

        app.activity_search_time_touched = true;

        assert_eq!(app.activity_search_available_time_seconds(), None);
        assert_eq!(
            app.activity_search_time_error,
            Some("La duree doit etre superieure a 00:00:00.".to_string())
        );

        app.activity_search_time = DurationInput {
            hours: "01".to_string(),
            minutes: "30".to_string(),
            seconds: "00".to_string(),
        };

        assert_eq!(app.activity_search_available_time_seconds(), Some(5_400.0));
        assert!(app.activity_search_time_error.is_none());
    }

    #[test]
    fn responsive_columns_reduces_column_count_when_width_is_tight() {
        assert_eq!(responsive_columns(1440.0, 260.0, SECTION_GAP, 3), 3);
        assert_eq!(responsive_columns(860.0, 260.0, SECTION_GAP, 3), 3);
        assert_eq!(responsive_columns(720.0, 260.0, SECTION_GAP, 3), 2);
        assert_eq!(responsive_columns(240.0, 260.0, SECTION_GAP, 3), 1);
    }

    #[test]
    fn grid_item_width_accounts_for_inter_item_gap() {
        assert_eq!(grid_item_width(900.0, 3, 12.0), 292.0);
        assert_eq!(grid_item_width(600.0, 2, 20.0), 290.0);
    }

    #[test]
    fn activity_result_columns_falls_back_to_single_column_earlier() {
        assert_eq!(activity_result_columns(1_300.0), 2);
        assert_eq!(activity_result_columns(1_100.0), 1);
        assert_eq!(activity_result_columns(620.0), 1);
    }

    #[test]
    fn recommended_class_prefers_highest_kamas_per_hour() {
        let entries = vec![
            ZoneEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 1_200.0,
                session_total_kamas: 100_000.0,
                kamas_per_hour: 300_000.0,
            },
            ZoneEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Sadida),
                recorded_at: None,
                session_time_seconds: 1_500.0,
                session_total_kamas: 110_000.0,
                kamas_per_hour: 264_000.0,
            },
        ];

        assert_eq!(
            recommended_zone_class(&entries, "Blop"),
            Some(DofusClass::Cra)
        );
    }

    #[test]
    fn recommended_class_uses_best_entry_per_class_and_ignores_unknown() {
        let entries = vec![
            ZoneEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 1_200.0,
                session_total_kamas: 100_000.0,
                kamas_per_hour: 300_000.0,
            },
            ZoneEntry {
                name: "blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                session_time_seconds: 1_100.0,
                session_total_kamas: 110_000.0,
                kamas_per_hour: 360_000.0,
            },
            ZoneEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Sadida),
                recorded_at: None,
                session_time_seconds: 1_250.0,
                session_total_kamas: 120_000.0,
                kamas_per_hour: 345_600.0,
            },
            ZoneEntry {
                name: "Blop".to_string(),
                character_class: None,
                recorded_at: None,
                session_time_seconds: 1_000.0,
                session_total_kamas: 500_000.0,
                kamas_per_hour: 1_800_000.0,
            },
        ];

        assert_eq!(
            recommended_zone_class(&entries, "  BLoP "),
            Some(DofusClass::Cra)
        );
    }

    #[test]
    fn recommended_class_matches_accentless_name() {
        let entries = vec![ZoneEntry {
            name: "Cimetière".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at: None,
            session_time_seconds: 3_600.0,
            session_total_kamas: 300_000.0,
            kamas_per_hour: 1.0,
        }];

        assert_eq!(
            recommended_zone_class(&entries, "Cimetiere"),
            Some(DofusClass::Cra)
        );
    }

    #[test]
    fn recommended_dungeon_class_recomputes_stale_derived_values() {
        let entries = vec![
            DungeonEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Cra),
                recorded_at: None,
                run_time_minutes: 20.0,
                gross_kamas_per_run: 120_000.0,
                key_price: 20_000.0,
                net_kamas_per_run: 999_999.0,
                kamas_per_hour: 999_999.0,
            },
            DungeonEntry {
                name: "Blop".to_string(),
                character_class: Some(DofusClass::Feca),
                recorded_at: None,
                run_time_minutes: 10.0,
                gross_kamas_per_run: 70_000.0,
                key_price: 10_000.0,
                net_kamas_per_run: -1.0,
                kamas_per_hour: -1.0,
            },
        ];

        assert_eq!(
            recommended_dungeon_class(&entries, "Blop"),
            Some(DofusClass::Feca)
        );
    }

    #[test]
    fn apply_zone_list_action_request_delete_clears_current_edit() {
        let mut app = MyApp {
            zone_edit: Some(InlineEditState {
                index: 1,
                form: ZoneForm::default(),
                error: Some("edit".to_string()),
            }),
            ..Default::default()
        };

        app.apply_zone_list_action(ZoneListAction::RequestDelete(2));

        assert_eq!(app.zone_delete_confirm, Some(2));
        assert!(app.zone_edit.is_none());
    }

    #[test]
    fn apply_zone_list_action_cancel_delete_only_clears_matching_index() {
        let mut app = MyApp {
            zone_delete_confirm: Some(1),
            ..Default::default()
        };

        app.apply_zone_list_action(ZoneListAction::CancelDelete(2));
        assert_eq!(app.zone_delete_confirm, Some(1));

        app.apply_zone_list_action(ZoneListAction::CancelDelete(1));
        assert!(app.zone_delete_confirm.is_none());
    }

    #[test]
    fn preview_zone_returns_projected_hourly_value() {
        let preview = preview_zone(&ZoneForm {
            name: "Plaine".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: String::new(),
            session_time: DurationInput {
                hours: "01".to_string(),
                minutes: "00".to_string(),
                seconds: "00".to_string(),
            },
            session_total_kamas: "240000".to_string(),
        });

        assert_eq!(preview.value, "240 000 kamas/h");
        assert!(preview.detail.contains("01:00:00"));
    }

    #[test]
    fn preview_dungeon_waits_for_complete_inputs() {
        let preview = preview_dungeon(&DungeonForm {
            name: "Blop".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at_input: String::new(),
            run_time: DurationInput {
                hours: "00".to_string(),
                minutes: "20".to_string(),
                seconds: "00".to_string(),
            },
            gross_kamas_per_run: "120000".to_string(),
            key_price: String::new(),
        });

        assert_eq!(preview.value, "Calcul en attente");
    }

    #[test]
    fn recommendation_is_scoped_per_tab() {
        let zones = vec![ZoneEntry {
            name: "Blop".to_string(),
            character_class: Some(DofusClass::Sadida),
            recorded_at: None,
            session_time_seconds: 1_800.0,
            session_total_kamas: 120_000.0,
            kamas_per_hour: 240_000.0,
        }];
        let dungeons = vec![DungeonEntry {
            name: "Blop".to_string(),
            character_class: Some(DofusClass::Cra),
            recorded_at: None,
            run_time_minutes: 20.0,
            gross_kamas_per_run: 120_000.0,
            key_price: 15_000.0,
            net_kamas_per_run: 105_000.0,
            kamas_per_hour: 315_000.0,
        }];

        assert_eq!(
            recommended_zone_class(&zones, "Blop"),
            Some(DofusClass::Sadida)
        );
        assert_eq!(
            recommended_dungeon_class(&dungeons, "Blop"),
            Some(DofusClass::Cra)
        );
    }

    #[test]
    fn cumulative_chart_all_time_single_point_uses_real_timestamp_bounds() {
        let summary = summary_with_chart(
            None,
            vec![CategorySeries {
                kind: ActivityKind::Zone,
                points: vec![cumulative_point("2026-03-08 10:00", 120_000.0)],
                total_in_period: 120_000.0,
                session_count: 1,
            }],
        );

        let (min_x, max_x) = cumulative_chart_x_bounds(&summary).unwrap();
        let expected_x = datetime_to_plot_x(dt("2026-03-08 10:00"));

        assert_eq!(min_x, expected_x);
        assert!(max_x > min_x);
        assert!(min_x > 1_000_000_000.0);
    }

    #[test]
    fn cumulative_chart_all_time_multiple_points_has_no_artificial_zero_prefix() {
        let series = CategorySeries {
            kind: ActivityKind::Zone,
            points: vec![
                cumulative_point("2026-03-08 10:00", 120_000.0),
                cumulative_point("2026-03-08 12:00", 200_000.0),
            ],
            total_in_period: 200_000.0,
            session_count: 2,
        };
        let summary = summary_with_chart(None, vec![series.clone()]);

        let (min_x, max_x) = cumulative_chart_x_bounds(&summary).unwrap();
        let line_points = build_cumulative_line_points(&series, summary.period_started_at);

        assert_eq!(min_x, datetime_to_plot_x(dt("2026-03-08 10:00")));
        assert_eq!(max_x, datetime_to_plot_x(dt("2026-03-08 12:00")));
        assert_eq!(line_points.len(), 2);
        assert_eq!(
            line_points[0],
            [datetime_to_plot_x(dt("2026-03-08 10:00")), 120_000.0]
        );
    }

    #[test]
    fn cumulative_chart_rolling_period_prefixes_zero_at_period_start() {
        let period_started_at = dt("2026-03-07 18:00");
        let series = CategorySeries {
            kind: ActivityKind::Zone,
            points: vec![cumulative_point("2026-03-08 10:00", 120_000.0)],
            total_in_period: 120_000.0,
            session_count: 1,
        };
        let summary = summary_with_chart(Some(period_started_at), vec![series.clone()]);

        let (min_x, max_x) = cumulative_chart_x_bounds(&summary).unwrap();
        let line_points = build_cumulative_line_points(&series, summary.period_started_at);

        assert_eq!(min_x, datetime_to_plot_x(period_started_at));
        assert_eq!(max_x, datetime_to_plot_x(dt("2026-03-08 10:00")));
        assert_eq!(line_points.len(), 2);
        assert_eq!(line_points[0], [datetime_to_plot_x(period_started_at), 0.0]);
        assert_eq!(
            line_points[1],
            [datetime_to_plot_x(dt("2026-03-08 10:00")), 120_000.0]
        );
    }

    #[test]
    fn cumulative_chart_bounds_return_none_when_all_series_are_empty() {
        let summary = summary_with_chart(
            None,
            vec![CategorySeries {
                kind: ActivityKind::Zone,
                points: Vec::new(),
                total_in_period: 0.0,
                session_count: 0,
            }],
        );

        assert_eq!(cumulative_chart_x_bounds(&summary), None);
    }
}
