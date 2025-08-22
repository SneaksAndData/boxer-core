use log::{set_boxed_logger, set_max_level, LevelFilter, Log, SetLoggerError};

pub struct ComposedLogger {
    loggers: Vec<Box<dyn Log>>,
    global_filter: Option<LevelFilter>,
}

impl ComposedLogger {
    pub fn new() -> Self {
        Self {
            loggers: Vec::new(),
            global_filter: None,
        }
    }

    pub fn with_logger(mut self, logger: Box<dyn Log>) -> Self {
        self.loggers.push(logger);
        self
    }

    pub fn with_global_level(mut self, filter: LevelFilter) -> Self {
        self.global_filter = Some(filter);
        self
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        let global_filter = self.global_filter.unwrap_or(LevelFilter::Info);
        set_boxed_logger(Box::new(self))?;
        set_max_level(global_filter);
        Ok(())
    }
}

impl Log for ComposedLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.loggers.iter().any(|logger| logger.enabled(metadata))
    }

    fn log(&self, record: &log::Record<'_>) {
        for logger in &self.loggers {
            if logger.enabled(record.metadata()) {
                logger.log(record);
            }
        }
    }

    fn flush(&self) {
        for logger in &self.loggers {
            logger.flush();
        }
    }
}
