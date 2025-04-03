use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shlex::split;
use std::collections::HashMap;
use tokio::process::Command as TokioCommand;

use crate::command::traits::{
    Command, CommandError, CommandExecution, CommandResult, ExecutionMode,
};
use crate::visitor::Visitor;

/// Структура для выполнения команд в оболочке
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommand {
    /// Название команды
    name: String,

    /// Командная строка для выполнения
    command: String,

    /// Рабочая директория для выполнения команды
    working_dir: Option<String>,

    /// Переменные окружения
    env_vars: HashMap<String, String>,

    /// Режим выполнения
    mode: ExecutionMode,

    /// Флаг, поддерживает ли команда откат
    supports_rollback: bool,

    /// Команда для отката
    rollback_command: Option<String>,

    /// Таймаут выполнения команды в секундах
    timeout_seconds: Option<u64>,
}

impl ShellCommand {
    /// Создает новую команду для оболочки
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            working_dir: None,
            env_vars: HashMap::new(),
            mode: ExecutionMode::Sequential,
            supports_rollback: false,
            rollback_command: None,
            timeout_seconds: None,
        }
    }

    /// Устанавливает рабочую директорию
    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// Добавляет переменную окружения
    pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Устанавливает режим выполнения
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Устанавливает команду отката
    pub fn with_rollback(mut self, rollback_command: &str) -> Self {
        self.supports_rollback = true;
        self.rollback_command = Some(rollback_command.to_string());
        self
    }

    /// Устанавливает таймаут выполнения
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Выполняет токио команду с таймаутом
    async fn execute_with_timeout(&self) -> Result<CommandResult, CommandError> {
        let args = match split(&self.command) {
            Some(args) => args,
            None => {
                return Err(CommandError::ExecutionError(format!(
                    "Не удалось разобрать команду: {}",
                    self.command
                )))
            }
        };

        if args.is_empty() {
            return Err(CommandError::ExecutionError("Пустая команда".to_string()));
        }

        let result = CommandResult::new(&self.name);

        let mut cmd = TokioCommand::new(&args[0]);

        if args.len() > 1 {
            cmd.args(&args[1..]);
        }

        // Устанавливаем рабочую директорию, если указана
        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        // Устанавливаем переменные окружения
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Запускаем команду и получаем результат
        let exec_future = cmd.output();

        // Применяем таймаут, если установлен
        let output = if let Some(timeout_secs) = self.timeout_seconds {
            match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), exec_future)
                .await
            {
                Ok(res) => res?,
                Err(_) => return Err(CommandError::TimeoutError),
            }
        } else {
            exec_future.await?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(result.success(stdout))
        } else {
            let error_msg = if stderr.is_empty() {
                format!(
                    "Команда завершилась с ошибкой: код {}",
                    output.status.code().unwrap_or(-1)
                )
            } else {
                stderr
            };

            Ok(result.failure(error_msg, output.status.code()))
        }
    }
}

#[async_trait]
impl CommandExecution for ShellCommand {
    async fn execute(&self) -> Result<CommandResult, CommandError> {
        self.execute_with_timeout().await
    }

    async fn rollback(&self) -> Result<CommandResult, CommandError> {
        if !self.supports_rollback {
            return Err(CommandError::RollbackError(
                "Команда не поддерживает откат".to_string(),
            ));
        }

        let rollback_cmd = match &self.rollback_command {
            Some(cmd) => cmd,
            None => {
                return Err(CommandError::RollbackError(
                    "Команда отката не задана".to_string(),
                ))
            }
        };

        let mut rollback = Self::new(&format!("{}_rollback", self.name), rollback_cmd);

        if let Some(dir) = &self.working_dir {
            rollback.working_dir = Some(dir.clone());
        }

        rollback.env_vars = self.env_vars.clone();
        rollback.mode = self.mode;

        rollback.execute().await
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn execution_mode(&self) -> ExecutionMode {
        self.mode
    }

    fn supports_rollback(&self) -> bool {
        self.supports_rollback
    }
}

#[async_trait]
impl Command for ShellCommand {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_shell_command(self);
    }
}
