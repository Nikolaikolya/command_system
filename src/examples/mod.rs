use crate::{
    builder::{ChainBuilder, CommandBuilder},
    chain::ChainExecutionMode,
    command::ExecutionMode,
    logging::{CompositeLogger, ConsoleLogger, FileLogger, LogLevel},
};

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

    // Создаем цепочку команд с помощью строителя
    let mut chain = ChainBuilder::new("backup_chain")
        .execution_mode(ChainExecutionMode::Sequential)
        .logger(Box::new(logger))
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
    let logger = Box::new(ConsoleLogger::new(LogLevel::Debug));

    // Создаем цепочку команд для параллельного выполнения
    let mut chain = ChainBuilder::new("parallel_tasks")
        .execution_mode(ChainExecutionMode::Parallel)
        .logger(logger)
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
