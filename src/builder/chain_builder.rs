use crate::chain::{ChainExecutionMode, CommandChain};
use crate::command::Command;
use crate::logging::Logger;
use std::sync::Arc;

/// Строитель для цепочки команд (паттерн Строитель)
pub struct ChainBuilder {
    /// Название цепочки
    name: String,

    /// Режим выполнения
    execution_mode: ChainExecutionMode,

    /// Логгер для записи событий
    logger: Option<Arc<Box<dyn Logger>>>,

    /// Откатывать ли выполненные команды в случае ошибки
    rollback_on_error: bool,
}

impl ChainBuilder {
    /// Создает нового строителя
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            execution_mode: ChainExecutionMode::Sequential,
            logger: None,
            rollback_on_error: true,
        }
    }

    /// Устанавливает режим выполнения
    pub fn execution_mode(mut self, mode: ChainExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Устанавливает логгер
    pub fn logger(mut self, logger: Arc<Box<dyn Logger>>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Устанавливает флаг отката при ошибке
    pub fn rollback_on_error(mut self, rollback: bool) -> Self {
        self.rollback_on_error = rollback;
        self
    }

    /// Строит цепочку команд
    pub fn build(self) -> CommandChain {
        CommandChain::new(
            &self.name,
            self.execution_mode,
            self.logger,
            self.rollback_on_error,
        )
    }

    /// Строит цепочку команд с набором начальных команд
    pub fn build_with_commands<C>(self, commands: Vec<C>) -> CommandChain
    where
        C: Command + 'static,
    {
        let mut chain = self.build();

        for command in commands {
            chain.add_command(command);
        }

        chain
    }
}

/// Создает последовательную цепочку команд
pub fn build_sequential_chain(name: &str, commands: Vec<Box<dyn Command>>) -> CommandChain {
    let mut chain = CommandChain::new(name, ChainExecutionMode::Sequential, None, true);

    for command in commands {
        chain.add_boxed_command(command);
    }

    chain
}

/// Создает параллельную цепочку команд
pub fn build_parallel_chain(name: &str, commands: Vec<Box<dyn Command>>) -> CommandChain {
    let mut chain = CommandChain::new(name, ChainExecutionMode::Parallel, None, true);

    for command in commands {
        chain.add_boxed_command(command);
    }

    chain
}
