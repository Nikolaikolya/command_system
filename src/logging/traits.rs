use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Уровни логирования
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LogLevel {
    /// Детальное логирование отладочной информации
    Debug,
    /// Информационные сообщения
    Info,
    /// Предупреждения
    Warning,
    /// Ошибки
    Error,
    /// Критические ошибки
    Critical,
}

impl LogLevel {
    /// Возвращает строковое представление уровня логирования
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Контекст для логирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    /// Имя вызывающего метода
    pub caller: Option<String>,

    /// Файл, в котором произошло событие
    pub file: Option<String>,

    /// Строка, в которой произошло событие
    pub line: Option<u32>,

    /// Дополнительные данные
    pub extra: Option<serde_json::Value>,
}

impl LogContext {
    /// Создает новый контекст логирования
    pub fn new() -> Self {
        Self {
            caller: None,
            file: None,
            line: None,
            extra: None,
        }
    }

    /// Устанавливает имя вызывающего метода
    pub fn with_caller(mut self, caller: &str) -> Self {
        self.caller = Some(caller.to_string());
        self
    }

    /// Устанавливает файл и строку
    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }

    /// Добавляет дополнительные данные
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Трейт для логирования
#[async_trait]
pub trait Logger: Send + Sync {
    /// Логирует сообщение с указанным уровнем
    fn log(&self, level: LogLevel, message: &str);

    /// Логирует сообщение с контекстом
    fn log_with_context(&self, level: LogLevel, message: &str, context: &LogContext);

    /// Логирует отладочное сообщение
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Логирует информационное сообщение
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Логирует предупреждение
    fn warning(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    /// Логирует ошибку
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    /// Логирует критическую ошибку
    fn critical(&self, message: &str) {
        self.log(LogLevel::Critical, message);
    }
}

/// Трейт стратегии логирования (паттерн Стратегия)
pub trait LoggingStrategy: Logger {
    /// Добавляет логгер в стратегию
    fn add_logger(&mut self, logger: Box<dyn Logger>);
}
