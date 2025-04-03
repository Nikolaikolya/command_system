use futures::future;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::command::{Command, CommandExecution, CommandResult, ExecutionMode};
use crate::command::traits::CommandError;
use crate::logging::{LogLevel, Logger};
use crate::visitor::{LogVisitor, Visitor};

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
    commands: Vec<Arc<dyn Command>>,

    /// Режим выполнения цепочки
    mode: ChainExecutionMode,

    /// Логгер для записи событий
    logger: Option<Box<dyn Logger>>,

    /// Откатывать ли выполненные команды в случае ошибки
    rollback_on_error: bool,
}

impl CommandChain {
    /// Создает новую цепочку команд
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            commands: Vec::new(),
            mode: ChainExecutionMode::Sequential,
            logger: None,
            rollback_on_error: true,
        }
    }

    /// Добавляет команду в цепочку
    pub fn add_command<C: Command + 'static>(&mut self, command: C) -> &mut Self {
        // Логируем добавление команды, если логгер установлен
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Добавлена команда '{}' в цепочку '{}'",
                command.name(),
                self.name
            ));
        }

        // Создаем визитор для логирования, если логгер установлен
        if let Some(logger) = &self.logger {
            let mut visitor = LogVisitor::new(Box::new(logger.clone()), LogLevel::Debug);

            // Применяем визитор к команде
            command.accept(&mut visitor);
        }

        // Добавляем команду в список
        self.commands.push(Arc::new(command));
        self
    }

    /// Устанавливает режим выполнения цепочки
    pub fn with_execution_mode(&mut self, mode: ChainExecutionMode) -> &mut Self {
        self.mode = mode;

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
    pub fn with_logger(&mut self, logger: Box<dyn Logger>) -> &mut Self {
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
        // Выбираем режим выполнения
        let execution_mode = match self.mode {
            ChainExecutionMode::Sequential => ExecutionMode::Sequential,
            ChainExecutionMode::Parallel => ExecutionMode::Parallel,
            ChainExecutionMode::Auto => {
                // Если хотя бы одна команда последовательная, то выполняем последовательно
                if self
                    .commands
                    .iter()
                    .any(|cmd| cmd.execution_mode() == ExecutionMode::Sequential)
                {
                    ExecutionMode::Sequential
                } else {
                    ExecutionMode::Parallel
                }
            }
        };

        // Логируем начало выполнения
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Начало выполнения цепочки '{}' в режиме {:?}",
                self.name, execution_mode
            ));
        }

        let result = match execution_mode {
            ExecutionMode::Sequential => self.execute_sequential().await,
            ExecutionMode::Parallel => self.execute_parallel().await,
        };

        // Логируем результат выполнения
        if let Some(logger) = &self.logger {
            match &result {
                Ok(chain_result) => {
                    if chain_result.success {
                        logger.info(&format!(
                            "Цепочка '{}' успешно выполнена ({} команд)",
                            self.name,
                            chain_result.results.len()
                        ));
                    } else {
                        logger.error(&format!(
                            "Ошибка выполнения цепочки '{}': {}",
                            self.name,
                            chain_result
                                .error
                                .as_ref()
                                .unwrap_or(&"<неизвестная ошибка>".to_string())
                        ));
                    }
                }
                Err(err) => {
                    logger.error(&format!(
                        "Критическая ошибка выполнения цепочки '{}': {}",
                        self.name, err
                    ));
                }
            }
        }

        result
    }

    /// Выполняет команды последовательно
    async fn execute_sequential(&self) -> Result<ChainResult, CommandError> {
        let mut results = Vec::with_capacity(self.commands.len());
        let mut executed_commands = Vec::new();

        for command in &self.commands {
            // Логируем выполнение команды
            if let Some(logger) = &self.logger {
                logger.info(&format!(
                    "Выполнение команды '{}' в цепочке '{}'",
                    command.name(),
                    self.name
                ));
            }

            match command.execute().await {
                Ok(result) => {
                    // Сохраняем команду как выполненную
                    executed_commands.push(Arc::clone(command));

                    if result.success {
                        // Логируем успешное выполнение
                        if let Some(logger) = &self.logger {
                            logger.info(&format!("Команда '{}' успешно выполнена", command.name()));
                        }

                        results.push(result);
                    } else {
                        // Команда выполнилась с ошибкой
                        if let Some(logger) = &self.logger {
                            logger.error(&format!(
                                "Ошибка выполнения команды '{}': {}",
                                command.name(),
                                result
                                    .error
                                    .as_ref()
                                    .unwrap_or(&String::from("<неизвестная ошибка>"))
                            ));
                        }

                        results.push(result.clone());

                        // Выполняем откат, если нужно
                        if self.rollback_on_error {
                            self.rollback_commands(&executed_commands).await;
                        }

                        return Ok(ChainResult {
                            results,
                            success: false,
                            error: result.error,
                        });
                    }
                }
                Err(err) => {
                    // Логируем ошибку
                    if let Some(logger) = &self.logger {
                        logger.error(&format!(
                            "Критическая ошибка выполнения команды '{}': {}",
                            command.name(),
                            err
                        ));
                    }

                    // Выполняем откат, если нужно
                    if self.rollback_on_error {
                        self.rollback_commands(&executed_commands).await;
                    }

                    return Err(err);
                }
            }
        }

        Ok(ChainResult {
            results,
            success: true,
            error: None,
        })
    }

    /// Выполняет команды параллельно
    async fn execute_parallel(&self) -> Result<ChainResult, CommandError> {
        if self.commands.is_empty() {
            return Ok(ChainResult {
                results: Vec::new(),
                success: true,
                error: None,
            });
        }

        // Логируем параллельное выполнение
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Параллельное выполнение {} команд в цепочке '{}'",
                self.commands.len(),
                self.name
            ));
        }

        // Выполняем команды параллельно
        let futures = self
            .commands
            .iter()
            .map(|cmd| async move {
                // Логируем выполнение команды
                if let Some(logger) = &self.logger {
                    logger.info(&format!(
                        "Выполнение команды '{}' в цепочке '{}'",
                        cmd.name(),
                        self.name
                    ));
                }

                let result = cmd.execute().await;

                if let Ok(ref cmd_result) = result {
                    if cmd_result.success {
                        // Логируем успешное выполнение
                        if let Some(logger) = &self.logger {
                            logger.info(&format!("Команда '{}' успешно выполнена", cmd.name()));
                        }
                    } else {
                        // Логируем ошибку
                        if let Some(logger) = &self.logger {
                            logger.error(&format!(
                                "Ошибка выполнения команды '{}': {}",
                                cmd.name(),
                                cmd_result
                                    .error
                                    .as_ref()
                                    .unwrap_or(&String::from("<неизвестная ошибка>"))
                            ));
                        }
                    }
                } else if let Err(ref err) = result {
                    // Логируем критическую ошибку
                    if let Some(logger) = &self.logger {
                        logger.error(&format!(
                            "Критическая ошибка выполнения команды '{}': {}",
                            cmd.name(),
                            err
                        ));
                    }
                }

                (cmd.clone(), result)
            })
            .collect::<Vec<_>>();

        // Ждем завершения всех команд
        let command_results = future::join_all(futures).await;

        // Обрабатываем результаты
        let mut results = Vec::new();
        let mut has_errors = false;
        let mut first_error = None;
        let mut executed_commands = Vec::new();

        for (command, result) in command_results {
            match result {
                Ok(cmd_result) => {
                    executed_commands.push(command);
                    results.push(cmd_result.clone());

                    if !cmd_result.success && !has_errors {
                        has_errors = true;
                        first_error = cmd_result.error.clone();
                    }
                }
                Err(err) => {
                    if !has_errors {
                        has_errors = true;
                        first_error = Some(err.to_string());
                    }
                }
            }
        }

        // Выполняем откат, если есть ошибки и установлен флаг отката
        if has_errors && self.rollback_on_error {
            self.rollback_commands(&executed_commands).await;
        }

        Ok(ChainResult {
            results,
            success: !has_errors,
            error: first_error,
        })
    }

    /// Выполняет откат команд
    async fn rollback_commands(&self, commands: &[Arc<dyn Command>]) {
        if let Some(logger) = &self.logger {
            logger.warning(&format!("Выполнение отката для цепочки '{}'", self.name));
        }

        // Откатываем команды в обратном порядке
        for command in commands.iter().rev() {
            if command.supports_rollback() {
                if let Some(logger) = &self.logger {
                    logger.info(&format!("Откат команды '{}'", command.name()));
                }

                match command.rollback().await {
                    Ok(result) => {
                        if result.success {
                            if let Some(logger) = &self.logger {
                                logger
                                    .info(&format!("Успешный откат команды '{}'", command.name()));
                            }
                        } else {
                            if let Some(logger) = &self.logger {
                                logger.error(&format!(
                                    "Ошибка отката команды '{}': {}",
                                    command.name(),
                                    result
                                        .error
                                        .unwrap_or_else(|| "<неизвестная ошибка>".to_string())
                                ));
                            }
                        }
                    }
                    Err(err) => {
                        if let Some(logger) = &self.logger {
                            logger.error(&format!(
                                "Критическая ошибка отката команды '{}': {}",
                                command.name(),
                                err
                            ));
                        }
                    }
                }
            } else {
                if let Some(logger) = &self.logger {
                    logger.warning(&format!(
                        "Команда '{}' не поддерживает откат",
                        command.name()
                    ));
                }
            }
        }
    }
}
