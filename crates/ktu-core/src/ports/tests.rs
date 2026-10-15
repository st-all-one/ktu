//! Testes das portas e dos seus *fakes* (determinismo, sem I/O real).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[cfg(feature = "instrument")]
use super::SeqClock;
use super::{
    Clock, Env, FixedClock, Fs, Level, LogRecord, Logger, MapEnv, MemFs, Metric, Metrics,
    RecordingLogger, RecordingMetrics, Rng, SeqRng, Timestamp,
};
use crate::events;

#[test]
fn fixed_clock_reports_its_instant() {
    let clock = FixedClock::new(Timestamp::from_millis(1_700_000_000_000));
    assert_eq!(clock.now().as_millis(), 1_700_000_000_000);
}

#[test]
fn seq_rng_cycles_deterministically() {
    let rng = SeqRng::new(vec![10, 20, 30]);
    let drawn = [
        rng.next_u64(),
        rng.next_u64(),
        rng.next_u64(),
        rng.next_u64(),
    ];
    assert_eq!(drawn, [10, 20, 30, 10]);
}

#[test]
fn seq_rng_empty_yields_zero() {
    let rng = SeqRng::new(Vec::new());
    assert_eq!(rng.next_u64(), 0);
}

#[test]
fn mem_fs_round_trips_a_file() {
    let fs = MemFs::new();
    let path = Path::new("/sala/nota.txt");
    assert!(!fs.exists(path));
    assert!(fs.write(path, b"ola").is_ok());
    assert!(fs.exists(path));
    assert_eq!(fs.read(path).ok(), Some(b"ola".to_vec()));
}

#[test]
fn mem_fs_with_file_preloads_content() {
    let fs = MemFs::new().with_file("/sala/nota.txt", b"pre");
    let read = fs.read(Path::new("/sala/nota.txt"));
    assert_eq!(read.ok(), Some(b"pre".to_vec()));
}

#[test]
fn mem_fs_missing_file_is_error() {
    let fs = MemFs::new();
    assert!(fs.read(Path::new("/ausente")).is_err());
}

#[test]
fn map_env_reads_injected_values() {
    let env = MapEnv::new().with("KTU_MODE", "test");
    assert_eq!(env.var("KTU_MODE").as_deref(), Some("test"));
    assert_eq!(env.var("AUSENTE"), None);
}

#[test]
fn recording_logger_captures_events() {
    let logger = RecordingLogger::new();
    logger.log(&LogRecord {
        level: Level::Info,
        event: events::SESSION_START,
        message: "inicio",
        fields: &[("agent", "ktu")],
    });
    let records = logger.snapshot();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records.first().map(|record| record.event.as_str()),
        Some(events::SESSION_START)
    );
    assert_eq!(
        records.first().map(|record| record.level),
        Some(Level::Info)
    );
}

#[test]
fn recording_metrics_snapshot_is_deterministic() {
    let metrics = RecordingMetrics::new();
    metrics.record(Metric::DurationMicros {
        name: "parse",
        micros: 5,
    });
    metrics.record(Metric::DurationMicros {
        name: "parse",
        micros: 7,
    });
    metrics.record(Metric::Counter {
        name: "tool.calls",
        delta: 2,
    });
    let snapshot = metrics.snapshot();
    assert_eq!(
        snapshot.durations_micros,
        BTreeMap::from([("parse".to_owned(), vec![5_u64, 7])])
    );
    assert_eq!(
        snapshot.counters,
        BTreeMap::from([("tool.calls".to_owned(), 2_u64)])
    );
}

#[test]
fn metrics_snapshot_serializes_to_json() {
    let metrics = RecordingMetrics::new();
    metrics.record(Metric::Counter {
        name: "steps",
        delta: 3,
    });
    let json = serde_json::to_string(&metrics.snapshot());
    assert_eq!(
        json.ok().as_deref(),
        Some(r#"{"durations_micros":{},"counters":{"steps":3}}"#)
    );
}

#[test]
fn event_ids_are_unique_and_namespaced() {
    let unique: BTreeSet<&str> = events::ALL.iter().copied().collect();
    assert_eq!(unique.len(), events::ALL.len());
    assert!(events::ALL.iter().all(|id| id.contains('.')));
}

#[cfg(feature = "instrument")]
#[test]
fn instrument_records_duration_when_enabled() {
    let metrics = RecordingMetrics::new();
    let clock = SeqClock::new(Timestamp::from_millis(0), vec![100, 350]);
    let value = crate::instrument!(&metrics, &clock, "sum", { 1_u64.saturating_add(2) });
    assert_eq!(value, 3);
    assert_eq!(
        metrics.snapshot().durations_micros,
        BTreeMap::from([("sum".to_owned(), vec![250_u64])])
    );
}
