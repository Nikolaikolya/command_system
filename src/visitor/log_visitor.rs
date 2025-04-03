use std::fmt;
use std::sync::Arc;

use super::Visitor;
use crate::command::traits::CommandExecution;
use crate::command::{CompositeCommand, ShellCommand};
use crate::logging::{LogLevel, Logger};

/// Структура для логирования команд
pub struct LogVisitor<'a> {
    /// Логгер для записи событий
    logger: &'a Arc<Box<dyn Logger>>,

    /// Уровень логирования
    level: LogLevel,
}

impl<'a> LogVisitor<'a> {
    /// Создает новый экземпляр LogVisitor
    pub fn new(logger: &'a Arc<Box<dyn Logger>>, level: LogLevel) -> Self {
        Self { logger, level }
    }

    /// Устанавливает уровень логирования
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Устанавливает логгер
    pub fn set_logger(&mut self, logger: &'a Arc<Box<dyn Logger>>) {
        self.logger = logger;
    }
}

impl<'a> Visitor for LogVisitor<'a> {
    fn visit_shell_command(&mut self, command: &ShellCommand) {
        let message = format!("Команда: {}", command.name());
        match self.level {
            LogLevel::Debug => self.logger.debug(&message),
            LogLevel::Info => self.logger.info(&message),
            LogLevel::Warning => self.logger.warning(&message),
            LogLevel::Error => self.logger.error(&message),
            LogLevel::Critical => self.logger.error(&message),
        }
    }

    fn visit_composite_command(&mut self, command: &CompositeCommand) {
        let message = format!(
            "Составная команда '{}' с режимом выполнения {:?}",
            command.name(),
            command.execution_mode()
        );

        match self.level {
            LogLevel::Debug => self.logger.debug(&message),
            LogLevel::Info => self.logger.info(&message),
            LogLevel::Warning => self.logger.warning(&message),
            LogLevel::Error => self.logger.error(&message),
            LogLevel::Critical => self.logger.error(&message),
        }
    }
}

impl<'a> fmt::Debug for LogVisitor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogVisitor")
            .field("level", &self.level)
            .finish()
    }
}
