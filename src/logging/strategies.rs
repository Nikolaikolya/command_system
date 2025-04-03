use std::sync::{Arc, Mutex};

use crate::logging::traits::{LogContext, LogLevel, Logger, LoggingStrategy};

/// Композитный логгер, объединяющий несколько стратегий логирования
pub struct CompositeLogger {
    /// Логгеры, которые будут использоваться для логирования
    loggers: Mutex<Vec<Box<dyn Logger>>>,
}

impl CompositeLogger {
    /// Создает новый композитный логгер
    pub fn new() -> Self {
        Self {
            loggers: Mutex::new(Vec::new()),
        }
    }

    /// Создает композитный логгер с изначальным списком логгеров
    pub fn with_loggers(loggers: Vec<Box<dyn Logger>>) -> Self {
        Self {
            loggers: Mutex::new(loggers),
        }
    }
}

impl Logger for CompositeLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // Получаем блокировку логгеров
        if let Ok(loggers) = self.loggers.lock() {
            // Отправляем сообщение во все логгеры
            for logger in loggers.iter() {
                logger.log(level, message);
            }
        }
    }

    fn log_with_context(&self, level: LogLevel, message: &str, context: &LogContext) {
        // Получаем блокировку логгеров
        if let Ok(loggers) = self.loggers.lock() {
            // Отправляем сообщение с контекстом во все логгеры
            for logger in loggers.iter() {
                logger.log_with_context(level, message, context);
            }
        }
    }
}

impl LoggingStrategy for CompositeLogger {
    fn add_logger(&mut self, logger: Box<dyn Logger>) {
        if let Ok(mut loggers) = self.loggers.lock() {
            loggers.push(logger);
        }
    }
}

/// Создает комбинированный логгер с консольным и файловым логгерами
pub fn create_default_logger() -> impl LoggingStrategy {
    let console_logger = Box::new(crate::logging::ConsoleLogger::new(LogLevel::Info));

    // По умолчанию записываем логи в файл logs/app.log
    let file_path = std::env::var("LOG_FILE").unwrap_or_else(|_| "logs/app.log".to_string());
    let file_logger = Box::new(crate::logging::FileLogger::new(LogLevel::Debug, &file_path));

    CompositeLogger::with_loggers(vec![console_logger, file_logger])
}

/// Создает тестовый логгер, который выводит только в консоль
pub fn create_test_logger() -> impl LoggingStrategy {
    let console_logger = Box::new(crate::logging::ConsoleLogger::new(LogLevel::Debug));
    CompositeLogger::with_loggers(vec![console_logger])
}
