use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shlex::split;
use std::collections::HashMap;
use std::env;
use std::io::{self as stdio, BufRead};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::{self, AsyncWriteExt};
use tokio::process::Command as TokioCommand;

use crate::command::traits::{
    Command, CommandError, CommandExecution, CommandResult, ExecutionMode,
};
use crate::visitor::Visitor;

lazy_static! {
    static ref VAR_PATTERN: Regex = Regex::new(r"\{([^{}]+)\}").unwrap();
    static ref ENV_VAR_PATTERN: Regex = Regex::new(r"\{\$([^{}]+)\}").unwrap();
    static ref FILE_VAR_PATTERN: Regex = Regex::new(r"\{#([^{}]+)\}").unwrap();
    static ref INTERACTIVE_VAR_PATTERN: Regex = Regex::new(r"\{([^$#{}][^{}]*)\}").unwrap();
}

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

    /// Путь к файлу с переменными
    variables_file: Option<String>,
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
            variables_file: None,
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

    /// Устанавливает файл с переменными
    pub fn with_variables_file(mut self, file_path: &str) -> Self {
        self.variables_file = Some(file_path.to_string());
        self
    }

    /// Интерактивный ввод значения переменной
    async fn prompt_for_variable(var_name: &str) -> Result<String, CommandError> {
        let mut stdout = io::stdout();
        stdout
            .write_all(format!("Введите значение для {}: ", var_name).as_bytes())
            .await
            .map_err(|e| CommandError::IoError(e))?;
        stdout.flush().await.map_err(|e| CommandError::IoError(e))?;

        let mut buffer = String::new();
        stdio::stdin()
            .lock()
            .read_line(&mut buffer)
            .map_err(|e| CommandError::IoError(e))?;

        Ok(buffer.trim().to_string())
    }

    /// Загружает переменные из файла
    async fn load_variables_from_file(
        file_path: &str,
    ) -> Result<HashMap<String, String>, CommandError> {
        let mut file = File::open(file_path).await.map_err(|e| {
            CommandError::ExecutionError(format!("Не удалось открыть файл с переменными: {}", e))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).await.map_err(|e| {
            CommandError::ExecutionError(format!("Не удалось прочитать файл с переменными: {}", e))
        })?;

        let json: Value = serde_json::from_str(&contents).map_err(|e| {
            CommandError::ExecutionError(format!("Не удалось разобрать JSON: {}", e))
        })?;

        let mut vars = HashMap::new();
        if let Value::Object(map) = json {
            for (key, value) in map {
                if let Value::String(val) = value {
                    vars.insert(key, val);
                } else {
                    vars.insert(key, value.to_string());
                }
            }
        }

        Ok(vars)
    }

    /// Заменяет переменные в командной строке
    async fn process_variables(&self, cmd: &str) -> Result<String, CommandError> {
        let mut processed_cmd = cmd.to_string();
        let mut file_vars = HashMap::new();

        // Загружаем переменные из файла, если указан
        if let Some(file_path) = &self.variables_file {
            file_vars = Self::load_variables_from_file(file_path).await?;
        }

        // Обрабатываем переменные из файла {#var}
        for cap in FILE_VAR_PATTERN.captures_iter(&cmd.to_string()) {
            let var_name = &cap[1];
            if let Some(_) = &self.variables_file {
                if let Some(value) = file_vars.get(var_name) {
                    processed_cmd = processed_cmd.replace(&cap[0], value);
                } else {
                    // Если переменной нет в файле, запрашиваем интерактивно
                    let value = Self::prompt_for_variable(var_name).await?;
                    processed_cmd = processed_cmd.replace(&cap[0], &value);
                }
            } else {
                // Файл не указан, запрашиваем интерактивно
                let value = Self::prompt_for_variable(var_name).await?;
                processed_cmd = processed_cmd.replace(&cap[0], &value);
            }
        }

        // Обрабатываем переменные окружения {$var}
        for cap in ENV_VAR_PATTERN.captures_iter(&processed_cmd.clone()) {
            let var_name = &cap[1];
            if let Ok(value) = env::var(var_name) {
                processed_cmd = processed_cmd.replace(&cap[0], &value);
            } else {
                // Если переменной нет в окружении, запрашиваем интерактивно
                let value = Self::prompt_for_variable(var_name).await?;
                processed_cmd = processed_cmd.replace(&cap[0], &value);
            }
        }

        // Обрабатываем интерактивные переменные {var}
        for cap in INTERACTIVE_VAR_PATTERN.captures_iter(&processed_cmd.clone()) {
            let var_name = &cap[1];
            let value = Self::prompt_for_variable(var_name).await?;
            processed_cmd = processed_cmd.replace(&cap[0], &value);
        }

        Ok(processed_cmd)
    }

    /// Выполняет токио команду с таймаутом
    async fn execute_with_timeout(&self) -> Result<CommandResult, CommandError> {
        // Обрабатываем переменные в команде
        let processed_command = self.process_variables(&self.command).await?;

        let args = match split(&processed_command) {
            Some(args) => args,
            None => {
                return Err(CommandError::ExecutionError(format!(
                    "Не удалось разобрать команду: {}",
                    processed_command
                )))
            }
        };

        if args.is_empty() {
            return Err(CommandError::ExecutionError("Пустая команда".to_string()));
        }

        let result = CommandResult::new(&self.name);

        #[cfg(target_family = "unix")]
        let program = "sh";
        #[cfg(target_family = "unix")]
        let args = ["-c", &processed_command];

        #[cfg(target_family = "windows")]
        let program = "cmd.exe";
        #[cfg(target_family = "windows")]
        let args = ["/C", &processed_command];

        let mut cmd = TokioCommand::new(program);
        cmd.args(&args);

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

        // Передаем файл с переменными в команду отката
        if let Some(vars_file) = &self.variables_file {
            rollback.variables_file = Some(vars_file.clone());
        }

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
