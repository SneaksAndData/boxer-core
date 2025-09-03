use env_filter::{Builder, Filter};
use log::{Log, SetLoggerError, set_boxed_logger, set_max_level};

pub struct ComposedLogger {
    loggers: Vec<Box<dyn Log>>,
    global_filter: Option<Filter>,
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

    pub fn with_global_level(mut self, filter: Filter) -> Self {
        self.global_filter = Some(filter);
        self
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        let default_filter = Builder::default().parse("info").build();

        let global_filter = match &self.global_filter {
            Some(f) => f.filter().clone(),
            None => default_filter.filter(),
        };
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
            match &self.global_filter {
                Some(f) => {
                    if f.matches(record) {
                        logger.log(record)
                    }
                }
                None => {
                    if logger.enabled(record.metadata()) {
                        logger.log(record)
                    }
                }
            }
        }
    }

    fn flush(&self) {
        for logger in &self.loggers {
            logger.flush();
        }
    }
}
