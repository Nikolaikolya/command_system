use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::command::traits::CommandError;
use crate::command::{Command, CommandResult};
use crate::logging::Logger;

/// Режим выполнения цепочки команд
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainExecutionMode {
    /// Последовательное выполнение команд
    Sequential,
    /// Параллельное выполнение команд
    Parallel,
    /// Автоматический выбор режима на основе флагов команд
    Auto,
}

/// Результат выполнения цепочки команд
#[derive(Debug)]
pub struct ChainResult {
    /// Результаты отдельных команд
    pub results: Vec<CommandResult>,

    /// Общий результат (успех/неудача)
    pub success: bool,

    /// Сообщение об ошибке (если есть)
    pub error: Option<String>,
}

/// Цепочка команд (паттерн Цепочка Обязанностей)
pub struct CommandChain {
    /// Название цепочки
    name: String,

    /// Список команд для выполнения
    commands: Vec<Box<dyn Command>>,

    /// Режим выполнения цепочки
    execution_mode: ChainExecutionMode,

    /// Логгер для записи событий
    logger: Option<Arc<Box<dyn Logger>>>,

    /// Откатывать ли выполненные команды в случае ошибки
    rollback_on_error: bool,
}

impl CommandChain {
    /// Создает новую цепочку команд
    pub fn new(
        name: &str,
        execution_mode: ChainExecutionMode,
        logger: Option<Arc<Box<dyn Logger>>>,
        rollback_on_error: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            commands: Vec::new(),
            execution_mode,
            logger,
            rollback_on_error,
        }
    }

    /// Добавляет команду в цепочку
    pub fn add_command<C: Command + 'static>(&mut self, command: C) -> &mut Self {
        self.commands.push(Box::new(command));
        self
    }

    /// Добавляет готовую команду в цепочку
    pub fn add_boxed_command(&mut self, command: Box<dyn Command>) -> &mut Self {
        self.commands.push(command);
        self
    }

    /// Устанавливает режим выполнения цепочки
    pub fn with_execution_mode(&mut self, mode: ChainExecutionMode) -> &mut Self {
        self.execution_mode = mode;

        // Логируем изменение режима, если логгер установлен
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Установлен режим выполнения цепочки '{}': {:?}",
                self.name, mode
            ));
        }

        self
    }

    /// Устанавливает логгер для цепочки команд
    pub fn with_logger(&mut self, logger: Arc<Box<dyn Logger>>) -> &mut Self {
        self.logger = Some(logger);
        self
    }

    /// Устанавливает флаг отката при ошибке
    pub fn with_rollback_on_error(&mut self, rollback: bool) -> &mut Self {
        self.rollback_on_error = rollback;

        // Логируем изменение флага отката, если логгер установлен
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Установлен флаг отката при ошибке для цепочки '{}': {}",
                self.name, rollback
            ));
        }

        self
    }

    /// Выполняет цепочку команд
    pub async fn execute(&self) -> Result<ChainResult, CommandError> {
        let mut executed_commands = Vec::new();

        for command in &self.commands {
            executed_commands.push(command);
            let result = command.execute().await?;

            if !result.success && self.rollback_on_error {
                self.rollback_commands(&executed_commands).await;
                return Ok(ChainResult {
                    success: false,
                    results: vec![result],
                    error: Some("Command execution failed".to_string()),
                });
            }
        }

        Ok(ChainResult {
            success: true,
            results: vec![],
            error: None,
        })
    }

    /// Откатывает выполненные команды
    async fn rollback_commands(&self, commands: &[&Box<dyn Command>]) {
        for command in commands.iter().rev() {
            if let Err(err) = command.rollback().await {
                if let Some(logger) = &self.logger {
                    logger.error(&format!("Failed to rollback command: {}", err));
                }
            }
        }
    }
}
