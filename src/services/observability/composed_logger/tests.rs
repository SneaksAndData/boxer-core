use super::ComposedLogger;
use env_filter::Builder;
use log::{Level, Log, Metadata, Record};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct CapturingLogger {
    min_level: Level,
    captured: Arc<Mutex<Vec<Level>>>,
}

impl CapturingLogger {
    fn new(min_level: Level) -> Self {
        Self {
            min_level,
            captured: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn levels(&self) -> Vec<Level> {
        self.captured.lock().unwrap().clone()
    }
}

impl Log for CapturingLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() >= self.min_level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            self.captured.lock().unwrap().push(record.level());
        }
    }

    fn flush(&self) {}
}

fn make_record(level: Level) -> Record<'static> {
    Record::builder()
        .args(format_args!("test"))
        .level(level)
        .target("test_target")
        .file(Some("test_file.rs"))
        .line(Some(1))
        .build()
}

#[test]
fn dispatch_without_global_filter_respects_individual_enabled() {
    let logger = CapturingLogger::new(Level::Info);
    let composed = ComposedLogger::new().with_logger(Box::new(logger.clone()));

    // Debug below Info -> not captured
    composed.log(&make_record(Level::Debug));
    // Info at threshold -> captured
    composed.log(&make_record(Level::Info));
    // Error above threshold -> captured
    composed.log(&make_record(Level::Error));

    assert_eq!(logger.levels(), vec![Level::Info, Level::Error]);
}

#[test]
fn dispatch_with_global_filter_blocks_lower_levels_globally() {
    let logger = CapturingLogger::new(Level::Debug); // would accept all >= Debug
    let global_filter = Builder::default().parse("warn").build();
    let composed = ComposedLogger::new()
        .with_logger(Box::new(logger.clone()))
        .with_global_level(global_filter);

    // Info below global warn -> filtered out
    composed.log(&make_record(Level::Info));
    // Warn -> allowed
    composed.log(&make_record(Level::Warn));
    // Error -> allowed
    composed.log(&make_record(Level::Error));

    assert_eq!(logger.levels(), vec![Level::Warn, Level::Error]);
}

#[test]
fn enabled_reflects_any_underlying_logger_enabled() {
    let logger_info = CapturingLogger::new(Level::Info);
    let logger_error = CapturingLogger::new(Level::Error);

    let composed = ComposedLogger::new()
        .with_logger(Box::new(logger_info))
        .with_logger(Box::new(logger_error));

    // Debug: neither logger enabled
    let debug_meta = make_record(Level::Debug).metadata().clone();
    assert!(!composed.enabled(&debug_meta));

    // Info: first logger enabled
    let info_meta = make_record(Level::Info).metadata().clone();
    assert!(composed.enabled(&info_meta));

    // Error: both enabled
    let error_meta = make_record(Level::Error).metadata().clone();
    assert!(composed.enabled(&error_meta));
}

#[test]
fn multiple_loggers_receive_matching_records() {
    let logger_a = CapturingLogger::new(Level::Info);
    let logger_b = CapturingLogger::new(Level::Info);

    let composed = ComposedLogger::new()
        .with_logger(Box::new(logger_a.clone()))
        .with_logger(Box::new(logger_b.clone()));

    composed.log(&make_record(Level::Info));

    assert_eq!(logger_a.levels(), vec![Level::Info]);
    assert_eq!(logger_b.levels(), vec![Level::Info]);
}
