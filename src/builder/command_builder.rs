use std::collections::HashMap;

use crate::command::{ExecutionMode, ShellCommand};

/// Строитель для команд (паттерн Строитель)
pub struct CommandBuilder {
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

    /// Команда для отката
    rollback_command: Option<String>,

    /// Таймаут выполнения команды в секундах
    timeout_seconds: Option<u64>,
}

impl CommandBuilder {
    /// Создает новый строитель команд
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            working_dir: None,
            env_vars: HashMap::new(),
            mode: ExecutionMode::Sequential,
            rollback_command: None,
            timeout_seconds: None,
        }
    }

    /// Устанавливает рабочую директорию
    pub fn working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// Добавляет переменную окружения
    pub fn env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Устанавливает режим выполнения
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Устанавливает команду отката
    pub fn rollback(mut self, rollback_command: &str) -> Self {
        self.rollback_command = Some(rollback_command.to_string());
        self
    }

    /// Устанавливает таймаут выполнения
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Строит команду
    pub fn build(self) -> ShellCommand {
        let mut command =
            ShellCommand::new(&self.name, &self.command).with_execution_mode(self.mode);

        if let Some(dir) = self.working_dir {
            command = command.with_working_dir(&dir);
        }

        for (key, value) in self.env_vars {
            command = command.with_env_var(&key, &value);
        }

        if let Some(rollback_cmd) = self.rollback_command {
            command = command.with_rollback(&rollback_cmd);
        }

        if let Some(timeout) = self.timeout_seconds {
            command = command.with_timeout(timeout);
        }

        command
    }
}

/// Создает команду быстро с минимальными параметрами
pub fn command(name: &str, command_str: &str) -> ShellCommand {
    ShellCommand::new(name, command_str)
}

/// Создает команду с откатом
pub fn command_with_rollback(name: &str, command_str: &str, rollback_str: &str) -> ShellCommand {
    ShellCommand::new(name, command_str).with_rollback(rollback_str)
}

/// Создает команду для параллельного выполнения
pub fn parallel_command(name: &str, command_str: &str) -> ShellCommand {
    ShellCommand::new(name, command_str).with_execution_mode(ExecutionMode::Parallel)
}
