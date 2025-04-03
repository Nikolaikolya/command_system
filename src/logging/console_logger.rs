use chrono::Local;
use colored::*;
use std::sync::Mutex;

use crate::logging::traits::{LogContext, LogLevel, Logger};

/// Структура для логирования в консоль с поддержкой цветов
pub struct ConsoleLogger {
    /// Минимальный уровень логирования
    min_level: LogLevel,

    /// Формат времени
    time_format: String,

    /// Мьютекс для синхронизации вывода
    output_mutex: Mutex<()>,
}

impl ConsoleLogger {
    /// Создает новый консольный логгер
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            time_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            output_mutex: Mutex::new(()),
        }
    }

    /// Устанавливает формат времени
    pub fn with_time_format(mut self, format: &str) -> Self {
        self.time_format = format.to_string();
        self
    }

    /// Возвращает цветной текст для уровня логирования
    fn get_colored_level(&self, level: LogLevel) -> ColoredString {
        match level {
            LogLevel::Debug => "DEBUG".cyan(),
            LogLevel::Info => "INFO".green(),
            LogLevel::Warning => "WARNING".yellow(),
            LogLevel::Error => "ERROR".red(),
            LogLevel::Critical => "CRITICAL".red().bold(),
        }
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // Проверяем, нужно ли логировать это сообщение
        if level as u8 >= self.min_level as u8 {
            // Блокируем мьютекс для избежания смешивания вывода
            let _lock = self.output_mutex.lock().unwrap_or_else(|e| e.into_inner());

            // Форматируем время
            let now = Local::now();
            let formatted_time = now.format(&self.time_format).to_string();

            // Выводим отформатированное сообщение
            println!(
                "{} [{}] {}",
                formatted_time,
                self.get_colored_level(level),
                message
            );
        }
    }

    fn log_with_context(&self, level: LogLevel, message: &str, context: &LogContext) {
        // Проверяем, нужно ли логировать это сообщение
        if level as u8 >= self.min_level as u8 {
            // Блокируем мьютекс для избежания смешивания вывода
            let _lock = self.output_mutex.lock().unwrap_or_else(|e| e.into_inner());

            // Форматируем время
            let now = Local::now();
            let formatted_time = now.format(&self.time_format).to_string();

            // Добавляем информацию о местоположении, если есть
            let location = if let (Some(file), Some(line)) = (&context.file, context.line) {
                format!(" ({}: {})", file, line)
            } else {
                String::new()
            };

            // Добавляем вызывающего, если есть
            let caller = if let Some(caller) = &context.caller {
                format!(" [{}]", caller)
            } else {
                String::new()
            };

            // Выводим отформатированное сообщение
            println!(
                "{} [{}]{}{} {}",
                formatted_time,
                self.get_colored_level(level),
                location,
                caller,
                message
            );
        }
    }
}
