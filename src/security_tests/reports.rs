use super::*;
use crate::models::ZoneEntry;

fn origin() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2026-09-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
}

fn session(index: usize, start: i64, duration: f32, value: f32) -> ReportSession {
    ReportSession {
        kind: ActivityKind::all()[index % 4],
        name: format!("Session {index}"),
        normalized_name: format!("session {index}"),
        class: None,
        recorded_at: Some(origin() + Duration::seconds(start)),
        duration_seconds: duration,
        accounting_value: value,
        kamas_per_hour: value * 3600.0 / duration,
    }
}

#[test]
fn overlapping_bars_share_one_detail_allocation() {
    let sessions = vec![session(0, 0, 3600.0, 10.0), session(1, 0, 3600.0, 20.0)];
    let (series, aggregated) =
        build_category_bar_series(&sessions, ReportCategoryFilter::default());
    assert!(!aggregated);
    let segments = series
        .iter()
        .flat_map(|series| &series.segments)
        .collect::<Vec<_>>();
    assert_eq!(segments.len(), 2);
    assert!(Arc::ptr_eq(
        &segments[0].simultaneous_sessions,
        &segments[1].simultaneous_sessions
    ));
    assert_eq!(segments[0].simultaneous_sessions.len(), 2);
    assert_eq!(segments[1].base_offset, 10.0);
}

#[test]
fn touching_sessions_do_not_overlap_at_the_shared_endpoint() {
    let sessions = vec![session(0, 0, 60.0, 10.0), session(1, 60, 60.0, 20.0)];
    let (series, aggregated) =
        build_category_bar_series(&sessions, ReportCategoryFilter::default());
    assert!(!aggregated);
    for bar in series.iter().flat_map(|series| &series.segments) {
        assert_eq!(bar.simultaneous_sessions.len(), 1);
        assert_eq!(bar.base_offset, 0.0);
    }
}

#[test]
fn ten_thousand_overlapping_sessions_have_bounded_geometry() {
    let sessions = (0..crate::limits::MAX_TOTAL_ENTRIES)
        .map(|index| session(index, index as i64 % 100, 3600.0, 100.0))
        .collect::<Vec<_>>();
    let (series, aggregated) =
        build_category_bar_series(&sessions, ReportCategoryFilter::default());
    assert!(aggregated);
    let bars = series
        .iter()
        .flat_map(|series| &series.segments)
        .collect::<Vec<_>>();
    assert!(bars.len() <= AGGREGATED_BAR_BUCKETS * 8);
    assert!(bars.iter().all(|bar| bar.simultaneous_sessions.is_empty()));
    let total: f64 = bars.iter().map(|bar| f64::from(bar.value)).sum();
    assert!((total - 1_000_000.0).abs() < 1.0);
}

#[test]
fn exact_mode_also_falls_back_when_its_segment_budget_is_exhausted() {
    let sessions = (0..MAX_EXACT_BAR_SESSIONS)
        .map(|index| session(index, index as i64, 3600.0, 100.0))
        .collect::<Vec<_>>();
    let (series, aggregated) =
        build_category_bar_series(&sessions, ReportCategoryFilter::default());
    assert!(aggregated);
    assert!(
        series
            .iter()
            .map(|series| series.segments.len())
            .sum::<usize>()
            <= MAX_BAR_SEGMENTS
    );
}

#[test]
fn aggregated_values_match_a_direct_overlap_reference() {
    let sessions = (0..257)
        .map(|index| {
            let value = if index % 3 == 0 { -120.0 } else { 250.0 };
            session(
                index,
                (index * 37 % 1000) as i64,
                (index * 29 % 1200 + 1) as f32,
                value,
            )
        })
        .collect::<Vec<_>>();
    let (series, aggregated) =
        build_category_bar_series(&sessions, ReportCategoryFilter::default());
    assert!(aggregated);
    for bar in series.iter().flat_map(|series| &series.segments) {
        let reference = sessions
            .iter()
            .filter(|session| {
                session.kind == bar.kind && (session.accounting_value < 0.0) == (bar.value < 0.0)
            })
            .map(|session| {
                let start = session.recorded_at.unwrap();
                let finish = start + Duration::seconds(session.duration_seconds.round() as i64);
                let overlap = (finish.min(bar.segment_end_at) - start.max(bar.segment_start_at))
                    .num_seconds()
                    .max(0) as f64;
                f64::from(session.accounting_value) * overlap
                    / (finish - start).num_seconds() as f64
            })
            .sum::<f64>();
        let tolerance = 1.0e-5 * reference.abs().max(1.0);
        assert!((f64::from(bar.value) - reference).abs() <= tolerance);
        assert_eq!(
            bar.base_offset >= 0.0,
            bar.value > 0.0 || bar.base_offset == 0.0
        );
    }
    let expected: f64 = sessions
        .iter()
        .map(|session| f64::from(session.accounting_value))
        .sum();
    let actual: f64 = series
        .iter()
        .flat_map(|series| &series.segments)
        .map(|bar| f64::from(bar.value))
        .sum();
    assert!((actual - expected).abs() <= expected.abs() * 1.0e-5);
}

#[test]
fn filtered_out_categories_do_not_affect_aggregation() {
    let sessions = (0..1024)
        .map(|index| session(index, 0, 60.0, 10.0))
        .collect::<Vec<_>>();
    let filter = ReportCategoryFilter {
        zones: true,
        dungeons: false,
        duo_trios: false,
        arenas: false,
    };
    let (series, aggregated) = build_category_bar_series(&sessions, filter);
    assert!(aggregated);
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].kind, ActivityKind::Zone);
    let total: f32 = series[0].segments.iter().map(|bar| bar.value).sum();
    assert!((total - 2560.0).abs() < 0.01);
}

#[test]
fn invalid_extreme_intervals_cannot_panic_in_the_bar_builder() {
    let mut sessions = vec![session(0, 0, 1.0e30, 10.0)];
    let mut last = session(1, 0, 3600.0, 10.0);
    last.recorded_at = Some(NaiveDateTime::MAX);
    sessions.push(last);
    let (series, _) = build_category_bar_series(&sessions, ReportCategoryFilter::default());
    assert!(series.iter().all(|series| series.segments.is_empty()));
}

#[test]
fn period_boundaries_at_chrono_minimum_do_not_panic() {
    assert_eq!(
        ReportPeriod::Last30Days.start_at(NaiveDateTime::MIN),
        Some(NaiveDateTime::MIN)
    );
}

#[test]
fn report_cache_reuses_values_and_invalidates_on_data_time_and_mode_changes() {
    let mut cache = None;
    let mut data = AppData {
        zones: vec![ZoneEntry {
            name: "Zone".to_string(),
            recorded_at: Some(origin()),
            session_time_seconds: 60.0,
            session_total_kamas: 10.0,
            ..ZoneEntry::default()
        }],
        ..AppData::default()
    };
    let categories = ReportCategoryFilter::default();
    let first = cached_report_summary(
        &mut cache,
        &data,
        ReportPeriod::AllTime,
        categories,
        origin(),
        false,
    );
    assert!(first.chart_bars.is_empty());
    let second = cached_report_summary(
        &mut cache,
        &data,
        ReportPeriod::AllTime,
        categories,
        origin(),
        false,
    );
    assert!(Arc::ptr_eq(&first, &second));
    data.zones[0].session_total_kamas = 20.0;
    let edited = cached_report_summary(
        &mut cache,
        &data,
        ReportPeriod::AllTime,
        categories,
        origin(),
        false,
    );
    assert!(!Arc::ptr_eq(&first, &edited));
    assert_eq!(edited.total_earned, 20.0);
    let bars = cached_report_summary(
        &mut cache,
        &data,
        ReportPeriod::AllTime,
        categories,
        origin(),
        true,
    );
    assert!(!bars.chart_bars.is_empty());
    let later = cached_report_summary(
        &mut cache,
        &data,
        ReportPeriod::AllTime,
        categories,
        origin() + Duration::seconds(1),
        true,
    );
    assert!(!Arc::ptr_eq(&bars, &later));
    let none = ReportCategoryFilter {
        zones: false,
        dungeons: false,
        duo_trios: false,
        arenas: false,
    };
    let filtered = cached_report_summary(
        &mut cache,
        &data,
        ReportPeriod::AllTime,
        none,
        origin(),
        true,
    );
    assert_eq!(filtered.session_count, 0);
}
