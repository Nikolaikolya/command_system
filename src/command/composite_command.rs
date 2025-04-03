use async_trait::async_trait;
use futures::future;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::command::traits::{
    Command, CommandError, CommandExecution, CommandResult, ExecutionMode,
};
use crate::visitor::Visitor;

/// Структура для группировки и последовательного или параллельного выполнения команд
#[derive(Clone)]
pub struct CompositeCommand {
    /// Название составной команды
    name: String,

    /// Список команд для выполнения
    commands: Vec<Arc<dyn Command>>,

    /// Режим выполнения
    mode: ExecutionMode,
}

impl CompositeCommand {
    /// Создает новую составную команду
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            commands: Vec::new(),
            mode: ExecutionMode::Sequential,
        }
    }

    /// Добавляет команду в группу
    pub fn add_command<C: Command + 'static>(&mut self, command: C) -> &mut Self {
        self.commands.push(Arc::new(command));
        self
    }

    /// Устанавливает режим выполнения
    pub fn with_execution_mode(&mut self, mode: ExecutionMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Выполняет команды последовательно
    async fn execute_sequential(&self) -> Result<CommandResult, CommandError> {
        let mut result = CommandResult::new(&self.name);
        let mut all_output = String::new();

        for command in &self.commands {
            match command.execute().await {
                Ok(cmd_result) => {
                    if !cmd_result.success {
                        return Ok(result.failure(
                            format!(
                                "Подкоманда {} завершилась с ошибкой: {}",
                                command.name(),
                                cmd_result
                                    .error
                                    .unwrap_or_else(|| "Неизвестная ошибка".to_string())
                            ),
                            cmd_result.exit_code,
                        ));
                    }

                    all_output.push_str(&format!("{}:\n{}\n", command.name(), cmd_result.output));
                }
                Err(err) => {
                    return Ok(result.failure(
                        format!(
                            "Ошибка при выполнении подкоманды {}: {}",
                            command.name(),
                            err
                        ),
                        None,
                    ));
                }
            }
        }

        Ok(result.success(all_output))
    }

    /// Выполняет команды параллельно
    async fn execute_parallel(&self) -> Result<CommandResult, CommandError> {
        let result = CommandResult::new(&self.name);

        let futures = self
            .commands
            .iter()
            .map(|cmd| cmd.execute())
            .collect::<Vec<_>>();

        let results = future::join_all(futures).await;

        let mut all_output = String::new();
        let mut has_errors = false;
        let mut first_error = None;
        let mut first_exit_code = None;

        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(cmd_result) => {
                    if !cmd_result.success && !has_errors {
                        has_errors = true;
                        first_error = cmd_result.error.clone();
                        first_exit_code = cmd_result.exit_code;
                    }

                    all_output.push_str(&format!(
                        "{}:\n{}\n",
                        self.commands[i].name(),
                        if cmd_result.success {
                            cmd_result.output
                        } else {
                            cmd_result
                                .error
                                .unwrap_or_else(|| "Неизвестная ошибка".to_string())
                        }
                    ));
                }
                Err(err) => {
                    if !has_errors {
                        has_errors = true;
                        first_error = Some(err.to_string());
                    }

                    all_output.push_str(&format!("{}: Ошибка: {}\n", self.commands[i].name(), err));
                }
            }
        }

        if has_errors {
            Ok(result.failure(
                format!(
                    "Некоторые подкоманды завершились с ошибкой: {}",
                    first_error.unwrap_or_else(|| "Неизвестная ошибка".to_string())
                ),
                first_exit_code,
            ))
        } else {
            Ok(result.success(all_output))
        }
    }

    /// Выполняет откат команд в обратном порядке
    async fn rollback_commands(&self) -> Result<CommandResult, CommandError> {
        let result = CommandResult::new(&format!("{}_rollback", self.name));
        let mut all_output = String::new();

        // Откатываем команды в обратном порядке
        for command in self.commands.iter().rev() {
            if command.supports_rollback() {
                match command.rollback().await {
                    Ok(cmd_result) => {
                        all_output.push_str(&format!(
                            "Откат {}:\n{}\n",
                            command.name(),
                            if cmd_result.success {
                                cmd_result.output
                            } else {
                                cmd_result
                                    .error
                                    .unwrap_or_else(|| "Неизвестная ошибка".to_string())
                            }
                        ));
                    }
                    Err(err) => {
                        all_output.push_str(&format!(
                            "Ошибка отката {}: {}\n",
                            command.name(),
                            err
                        ));
                    }
                }
            } else {
                all_output.push_str(&format!(
                    "Команда {} не поддерживает откат\n",
                    command.name()
                ));
            }
        }

        Ok(result.success(all_output))
    }
}

impl std::fmt::Debug for CompositeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeCommand")
            .field("name", &self.name)
            .field("commands_count", &self.commands.len())
            .field("mode", &self.mode)
            .finish()
    }
}

#[async_trait]
impl CommandExecution for CompositeCommand {
    async fn execute(&self) -> Result<CommandResult, CommandError> {
        match self.mode {
            ExecutionMode::Sequential => self.execute_sequential().await,
            ExecutionMode::Parallel => self.execute_parallel().await,
        }
    }

    async fn rollback(&self) -> Result<CommandResult, CommandError> {
        self.rollback_commands().await
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn execution_mode(&self) -> ExecutionMode {
        self.mode
    }

    fn supports_rollback(&self) -> bool {
        self.commands.iter().any(|cmd| cmd.supports_rollback())
    }
}

#[async_trait]
impl Command for CompositeCommand {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_composite_command(self);

        // Вызываем visitor для всех вложенных команд
        for command in &self.commands {
            command.accept(visitor);
        }
    }
}
