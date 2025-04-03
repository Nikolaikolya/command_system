use super::traits::LoggingStrategy;
use super::{CompositeLogger, ConsoleLogger, FileLogger, LogLevel, Logger};
use std::sync::Arc;

#[derive(Clone)]
pub struct LoggerManager {
    logger: Arc<Box<dyn Logger>>,
}

impl LoggerManager {
    pub fn new() -> Self {
        let console_logger = Box::new(ConsoleLogger::new(LogLevel::Info));
        let file_logger = Box::new(FileLogger::new(LogLevel::Info, "logs/commands.log"));

        let mut composite_logger = CompositeLogger::new();
        composite_logger.add_logger(console_logger);
        composite_logger.add_logger(file_logger);

        Self {
            logger: Arc::new(Box::new(composite_logger)),
        }
    }

    pub fn with_level(console_level: LogLevel, file_level: LogLevel) -> Self {
        let console_logger = Box::new(ConsoleLogger::new(console_level));
        let file_logger = Box::new(FileLogger::new(file_level, "logs/commands.log"));

        let mut composite_logger = CompositeLogger::new();
        composite_logger.add_logger(console_logger);
        composite_logger.add_logger(file_logger);

        Self {
            logger: Arc::new(Box::new(composite_logger)),
        }
    }

    pub fn get_logger(&self) -> Arc<Box<dyn Logger>> {
        self.logger.clone()
    }
}
