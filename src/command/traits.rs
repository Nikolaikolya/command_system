use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

use crate::visitor::Visitor;

/// Режим выполнения команды
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Последовательное выполнение
    Sequential,
    /// Параллельное выполнение
    Parallel,
}

/// Ошибки, возникающие при выполнении команд
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Ошибка выполнения: {0}")]
    ExecutionError(String),

    #[error("Ошибка отката: {0}")]
    RollbackError(String),

    #[error("Таймаут выполнения")]
    TimeoutError,

    #[error("Команда прервана: {0}")]
    Interrupted(String),

    #[error("Ошибка ввода/вывода: {0}")]
    IoError(#[from] std::io::Error),
}

/// Результат выполнения команды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Уникальный идентификатор результата
    pub id: String,

    /// Имя команды
    pub command_name: String,

    /// Успешность выполнения
    pub success: bool,

    /// Вывод команды
    pub output: String,

    /// Сообщение об ошибке (если есть)
    pub error: Option<String>,

    /// Код возврата (если применимо)
    pub exit_code: Option<i32>,

    /// Время начала выполнения
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// Время завершения выполнения
    pub end_time: chrono::DateTime<chrono::Utc>,

    /// Длительность выполнения в миллисекундах
    pub duration_ms: u64,
}

impl CommandResult {
    /// Создает новый результат выполнения команды
    pub fn new(command_name: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            command_name: command_name.to_string(),
            success: false,
            output: String::new(),
            error: None,
            exit_code: None,
            start_time: now,
            end_time: now,
            duration_ms: 0,
        }
    }

    /// Отмечает результат как успешный
    pub fn success(mut self, output: String) -> Self {
        self.success = true;
        self.output = output;
        self.end_time = chrono::Utc::now();
        self.duration_ms = (self.end_time - self.start_time).num_milliseconds() as u64;
        self
    }

    /// Отмечает результат как неудачный
    pub fn failure(mut self, error: String, exit_code: Option<i32>) -> Self {
        self.success = false;
        self.error = Some(error);
        self.exit_code = exit_code;
        self.end_time = chrono::Utc::now();
        self.duration_ms = (self.end_time - self.start_time).num_milliseconds() as u64;
        self
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.command_name,
            if self.success {
                "успех"
            } else {
                "ошибка"
            },
            if self.success {
                self.output.lines().next().unwrap_or("<нет вывода>")
            } else {
                self.error
                    .as_ref()
                    .unwrap_or(&String::from("<неизвестная ошибка>"))
            }
        )
    }
}

/// Трейт для выполнения команд
#[async_trait]
pub trait CommandExecution {
    /// Выполняет команду
    async fn execute(&self) -> Result<CommandResult, CommandError>;

    /// Выполняет откат команды, если это возможно
    async fn rollback(&self) -> Result<CommandResult, CommandError> {
        Err(CommandError::RollbackError(
            "Откат не реализован для этой команды".to_string(),
        ))
    }

    /// Возвращает имя команды
    fn name(&self) -> &str;

    /// Возвращает режим выполнения команды
    fn execution_mode(&self) -> ExecutionMode;

    /// Возвращает информацию, поддерживает ли команда откат
    fn supports_rollback(&self) -> bool {
        false
    }
}

/// Основной трейт команды
#[async_trait]
pub trait Command: CommandExecution + Send + Sync {
    /// Принимает визитор для реализации паттерна посетитель
    fn accept(&self, visitor: &mut dyn Visitor);
}
