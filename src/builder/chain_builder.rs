use crate::chain::{ChainExecutionMode, CommandChain};
use crate::command::Command;
use crate::logging::Logger;

/// Строитель для цепочки команд (паттерн Строитель)
pub struct ChainBuilder {
    /// Название цепочки
    name: String,

    /// Режим выполнения
    mode: ChainExecutionMode,

    /// Логгер для записи событий
    logger: Option<Box<dyn Logger>>,

    /// Откатывать ли выполненные команды в случае ошибки
    rollback_on_error: bool,
}

impl ChainBuilder {
    /// Создает новый строитель цепочки команд
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mode: ChainExecutionMode::Sequential,
            logger: None,
            rollback_on_error: true,
        }
    }

    /// Устанавливает режим выполнения
    pub fn execution_mode(mut self, mode: ChainExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Устанавливает логгер
    pub fn logger(mut self, logger: Box<dyn Logger>) -> Self {
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
        let mut chain = CommandChain::new(&self.name);

        chain
            .with_execution_mode(self.mode)
            .with_rollback_on_error(self.rollback_on_error);

        if let Some(logger) = self.logger {
            chain.with_logger(logger);
        }

        chain
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
pub fn sequential_chain(name: &str) -> CommandChain {
    let mut chain = CommandChain::new(name);
    chain.with_execution_mode(ChainExecutionMode::Sequential);
    chain
}

/// Создает параллельную цепочку команд
pub fn parallel_chain(name: &str) -> CommandChain {
    let mut chain = CommandChain::new(name);
    chain.with_execution_mode(ChainExecutionMode::Parallel);
    chain
}
