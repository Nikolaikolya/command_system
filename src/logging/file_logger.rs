use chrono::{DateTime, Local, Utc};
use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use crate::logging::traits::{LogContext, LogLevel, Logger};

/// Структура для логирования в файл в формате JSON
pub struct FileLogger {
    /// Минимальный уровень логирования
    min_level: LogLevel,

    /// Путь к файлу логов
    file_path: String,

    /// Мьютекс для синхронизации записи в файл
    file_mutex: Mutex<()>,
}

impl FileLogger {
    /// Создает новый файловый логгер
    pub fn new(min_level: LogLevel, file_path: &str) -> Self {
        // Создаем директорию для логов, если ее нет
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        Self {
            min_level,
            file_path: file_path.to_string(),
            file_mutex: Mutex::new(()),
        }
    }

    /// Открывает файл для записи (создает, если не существует)
    fn open_log_file(&self) -> std::io::Result<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
    }

    /// Записывает JSON-сообщение в файл
    fn write_json_log(&self, log_entry: serde_json::Value) -> std::io::Result<()> {
        // Блокируем мьютекс для синхронизации записи
        let _lock = self.file_mutex.lock().unwrap_or_else(|e| e.into_inner());

        // Открываем файл логов
        let mut file = self.open_log_file()?;

        // Сериализуем JSON и записываем в файл
        let log_json = serde_json::to_string(&log_entry)?;
        writeln!(file, "{}", log_json)?;

        Ok(())
    }
}

impl Logger for FileLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // Проверяем, нужно ли логировать это сообщение
        if level as u8 >= self.min_level as u8 {
            // Текущее время в разных форматах
            let now: DateTime<Utc> = Utc::now();
            let local_time = Local::now();

            // Создаем JSON запись
            let log_entry = json!({
                "timestamp": now.to_rfc3339(),
                "local_time": local_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "level": level.as_str(),
                "message": message,
            });

            // Пишем в файл
            if let Err(err) = self.write_json_log(log_entry) {
                eprintln!("Ошибка записи в файл логов: {}", err);
            }
        }
    }

    fn log_with_context(&self, level: LogLevel, message: &str, context: &LogContext) {
        // Проверяем, нужно ли логировать это сообщение
        if level as u8 >= self.min_level as u8 {
            // Текущее время в разных форматах
            let now: DateTime<Utc> = Utc::now();
            let local_time = Local::now();

            // Создаем JSON запись с контекстом
            let mut log_entry = json!({
                "timestamp": now.to_rfc3339(),
                "local_time": local_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "level": level.as_str(),
                "message": message,
            });

            // Добавляем контекст, если информация доступна
            if let Some(caller) = &context.caller {
                log_entry["caller"] = json!(caller);
            }

            if let Some(file) = &context.file {
                log_entry["file"] = json!(file);
            }

            if let Some(line) = context.line {
                log_entry["line"] = json!(line);
            }

            if let Some(extra) = &context.extra {
                log_entry["extra"] = extra.clone();
            }

            // Пишем в файл
            if let Err(err) = self.write_json_log(log_entry) {
                eprintln!("Ошибка записи в файл логов с контекстом: {}", err);
            }
        }
    }
}
