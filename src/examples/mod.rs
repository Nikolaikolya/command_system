use crate::builder::chain_builder::ChainBuilder;
use crate::builder::CommandBuilder;
use crate::chain::ChainExecutionMode;
use crate::command::shell_command::ShellCommand;
use crate::command::traits::{Command, CommandExecution};
use crate::command::ExecutionMode;
use crate::logging::LoggerManager;
use crate::logging::{
    CompositeLogger, ConsoleLogger, FileLogger, LogLevel, Logger, LoggingStrategy,
};
use crate::visitor::LogVisitor;
use std::sync::Arc;

/// Простой пример использования ShellCommand
pub async fn simple_command_example() {
    // Создаем команду напрямую
    let command = CommandBuilder::new("list_files", "ls -la")
        .working_dir("/tmp")
        .build();

    // Выполняем команду
    match command.execute().await {
        Ok(result) => {
            println!("Команда выполнена успешно: {}", result.success);
            println!("Вывод: {}", result.output);
        }
        Err(err) => {
            println!("Ошибка выполнения команды: {}", err);
        }
    }
}

/// Пример использования цепочки команд
pub async fn command_chain_example() {
    // Создаем логгер для консоли и файла
    let console_logger = Box::new(ConsoleLogger::new(LogLevel::Info));
    let file_logger = Box::new(FileLogger::new(LogLevel::Debug, "logs/commands.log"));

    // Создаем композитный логгер (паттерн Стратегия)
    let mut logger = CompositeLogger::new();
    logger.add_logger(console_logger);
    logger.add_logger(file_logger);
    let boxed_logger: Box<dyn Logger> = Box::new(logger);
    let arc_logger = Arc::new(boxed_logger);

    // Создаем цепочку команд с помощью строителя
    let mut chain = ChainBuilder::new("backup_chain")
        .execution_mode(ChainExecutionMode::Sequential)
        .logger(arc_logger)
        .rollback_on_error(true)
        .build();

    // Добавляем команды в цепочку
    chain.add_command(
        CommandBuilder::new("create_backup_dir", "mkdir -p /tmp/backup")
            .rollback("rm -rf /tmp/backup")
            .build(),
    );

    chain.add_command(
        CommandBuilder::new("backup_files", "cp -r /tmp/src/* /tmp/backup/")
            .rollback("rm -rf /tmp/backup/*")
            .timeout(30)
            .build(),
    );

    chain.add_command(
        CommandBuilder::new(
            "create_archive",
            "tar -czf /tmp/backup.tar.gz -C /tmp/backup .",
        )
        .rollback("rm -f /tmp/backup.tar.gz")
        .build(),
    );

    // Выполняем цепочку команд
    match chain.execute().await {
        Ok(result) => {
            if result.success {
                println!("Цепочка команд выполнена успешно!");

                // Выводим результаты каждой команды
                for (i, cmd_result) in result.results.iter().enumerate() {
                    println!("Команда {}: {}", i + 1, cmd_result.command_name);
                    println!("Вывод: {}", cmd_result.output);
                }
            } else {
                println!("Ошибка выполнения цепочки команд: {:?}", result.error);
            }
        }
        Err(err) => {
            println!("Критическая ошибка выполнения цепочки команд: {}", err);
        }
    }
}

/// Пример выполнения команд параллельно
pub async fn parallel_commands_example() {
    // Создаем логгер только для консоли
    let boxed_logger: Box<dyn Logger> = Box::new(ConsoleLogger::new(LogLevel::Debug));
    let arc_logger = Arc::new(boxed_logger);

    // Создаем цепочку команд для параллельного выполнения
    let mut chain = ChainBuilder::new("parallel_tasks")
        .execution_mode(ChainExecutionMode::Parallel)
        .logger(arc_logger)
        .build();

    // Добавляем команды с разными таймаутами
    chain.add_command(
        CommandBuilder::new("slow_task1", "sleep 5 && echo 'Task 1 completed'")
            .execution_mode(ExecutionMode::Parallel)
            .timeout(10)
            .build(),
    );

    chain.add_command(
        CommandBuilder::new("fast_task", "echo 'Fast task completed'")
            .execution_mode(ExecutionMode::Parallel)
            .build(),
    );

    chain.add_command(
        CommandBuilder::new("slow_task2", "sleep 3 && echo 'Task 2 completed'")
            .execution_mode(ExecutionMode::Parallel)
            .timeout(10)
            .build(),
    );

    // Выполняем задачи параллельно
    match chain.execute().await {
        Ok(result) => {
            println!("Все задачи завершены. Успех: {}", result.success);

            // Выводим результаты каждой задачи
            for cmd_result in result.results {
                println!("[{}] Вывод: {}", cmd_result.command_name, cmd_result.output);
            }
        }
        Err(err) => {
            println!("Критическая ошибка выполнения задач: {}", err);
        }
    }
}

/// Пример создания и выполнения цепочки команд с логированием
pub async fn chain_with_logging_example() {
    // Создаем менеджер логгеров
    let logger_manager = LoggerManager::new();
    let logger = logger_manager.get_logger();

    // Создаем команды
    let _cmd1 = ShellCommand::new("first_command", "echo 'First command'");
    let _cmd2 = ShellCommand::new("second_command", "echo 'Second command'");

    // Создаем цепочку через строитель
    let chain = ChainBuilder::new("Example Chain")
        .execution_mode(ChainExecutionMode::Sequential)
        .logger(logger)
        .rollback_on_error(true)
        .build();

    // Выполняем цепочку
    match chain.execute().await {
        Ok(result) => {
            println!("Chain executed successfully: {:?}", result);
        }
        Err(err) => {
            println!("Chain execution failed: {}", err);
        }
    }
}

/// Пример создания параллельной цепочки команд
pub async fn parallel_chain_example() {
    // Создаем команды
    let cmd1 = ShellCommand::new("task1", "sleep 1 && echo 'Command 1'");
    let cmd2 = ShellCommand::new("task2", "sleep 2 && echo 'Command 2'");
    let cmd3 = ShellCommand::new("task3", "sleep 3 && echo 'Command 3'");

    // Создаем цепочку через строитель
    let chain = ChainBuilder::new("Parallel Chain")
        .execution_mode(ChainExecutionMode::Parallel)
        .rollback_on_error(false)
        .build_with_commands(vec![cmd1, cmd2, cmd3]);

    // Выполняем цепочку
    if let Ok(result) = chain.execute().await {
        println!("All commands executed in parallel: {:?}", result);
    }
}

/// Пример использования композитного логгера с разными уровнями
pub async fn custom_logger_example() {
    // Создаем логгер с разными уровнями для консоли и файла
    let logger_manager = LoggerManager::with_level(LogLevel::Debug, LogLevel::Info);
    let logger = logger_manager.get_logger();

    // Создаем команду
    let cmd = ShellCommand::new("test_logger", "echo 'Testing custom logger levels'");

    // Создаем и выполняем цепочку
    let chain = ChainBuilder::new("Logger Test")
        .logger(logger)
        .build_with_commands(vec![cmd]);

    chain.execute().await.expect("Failed to execute chain");
}

/// Пример обработки ошибок и отката
pub async fn error_handling_example() {
    let boxed_logger: Box<dyn Logger> = Box::new(ConsoleLogger::new(LogLevel::Debug));
    let arc_logger = Arc::new(boxed_logger);
    let mut visitor = LogVisitor::new(&arc_logger, LogLevel::Debug);

    let cmd1 = CommandBuilder::new("slow_task1", "sleep 5 && echo 'Task 1 completed'")
        .execution_mode(ExecutionMode::Parallel)
        .build();
    cmd1.accept(&mut visitor);

    let cmd2 = CommandBuilder::new("fast_task", "echo 'Fast task completed'")
        .execution_mode(ExecutionMode::Parallel)
        .build();
    cmd2.accept(&mut visitor);

    let cmd3 = CommandBuilder::new("slow_task2", "sleep 3 && echo 'Task 2 completed'")
        .execution_mode(ExecutionMode::Parallel)
        .build();
    cmd3.accept(&mut visitor);

    let mut chain = ChainBuilder::new("error_chain")
        .rollback_on_error(true)
        .logger(arc_logger)
        .build();

    chain.add_command(cmd1);
    chain.add_command(cmd2);
    chain.add_command(cmd3);

    let result = chain.execute().await;
    println!("Chain execution result: {:?}", result);
}

pub async fn logging_example() {
    let console_logger = Box::new(ConsoleLogger::new(LogLevel::Info));
    let file_logger = Box::new(FileLogger::new(LogLevel::Debug, "logs/commands.log"));

    let mut logger = CompositeLogger::new();
    logger.add_logger(console_logger);
    logger.add_logger(file_logger);
    let boxed_logger: Box<dyn Logger> = Box::new(logger);
    let arc_logger = Arc::new(boxed_logger);
    let mut visitor = LogVisitor::new(&arc_logger, LogLevel::Info);

    let cmd1 = CommandBuilder::new("create_backup_dir", "mkdir -p /tmp/backup").build();
    cmd1.accept(&mut visitor);

    let cmd2 = CommandBuilder::new("backup_files", "cp -r /tmp/src/* /tmp/backup/").build();
    cmd2.accept(&mut visitor);

    let cmd3 = CommandBuilder::new(
        "verify_backup",
        "[ $(ls -A /tmp/backup | wc -l) -gt 0 ] && echo 'Backup successful' || echo 'Backup failed'",
    )
    .build();
    cmd3.accept(&mut visitor);

    let mut chain = ChainBuilder::new("backup_chain")
        .logger(arc_logger.clone())
        .build();

    chain.add_command(cmd1);
    chain.add_command(cmd2);
    chain.add_command(cmd3);

    let result = chain.execute().await;
    println!("Chain execution result: {:?}", result);
}
