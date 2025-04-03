use std::fmt;

use crate::command::{CompositeCommand, ShellCommand};
use crate::CommandExecution;
use crate::logging::{LogLevel, Logger};
use crate::visitor::Visitor;

/// Структура для логирования команд
pub struct LogVisitor {
    /// Логгер для записи событий
    logger: Box<dyn Logger>,

    /// Уровень логирования
    level: LogLevel,
}

impl LogVisitor {
    /// Создает новый экземпляр LogVisitor
    pub fn new(logger: Box<dyn Logger>, level: LogLevel) -> Self {
        Self { logger, level }
    }

    /// Устанавливает уровень логирования
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Устанавливает логгер
    pub fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = logger;
    }
}

impl Visitor for LogVisitor {
    fn visit_shell_command(&mut self, command: &ShellCommand) {
        let message = format!("Команда: {}", command.name());
        self.logger.log(self.level, &message);
    }

    fn visit_composite_command(&mut self, command: &CompositeCommand) {
        let message = format!(
            "Составная команда: {} (режим: {})",
            command.name(),
            match command.execution_mode() {
                crate::command::ExecutionMode::Sequential => "последовательный",
                crate::command::ExecutionMode::Parallel => "параллельный",
            }
        );

        self.logger.log(self.level, &message);
    }
}

impl fmt::Debug for LogVisitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogVisitor")
            .field("level", &self.level)
            .finish()
    }
}
