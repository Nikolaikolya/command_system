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

/// Пример использования интерактивного ввода и подстановки переменных
pub async fn interactive_variables_example() {
    // Пример с интерактивным вводом
    let command = CommandBuilder::new(
        "greeting",
        "echo 'Привет, {name}! Добро пожаловать в {system}'",
    )
    .build();

    // При выполнении запросит значения для {name} и {system}
    let result = command.execute().await.unwrap();
    println!(
        "Результат команды с интерактивным вводом: {}",
        result.output
    );

    // Пример с использованием переменных окружения
    std::env::set_var("USER_NAME", "John");
    std::env::set_var("SYS_NAME", "Command System");

    let command = CommandBuilder::new(
        "env_greeting",
        "echo 'Привет, {$USER_NAME}! Добро пожаловать в {$SYS_NAME}'",
    )
    .build();

    // Использует переменные окружения USER_NAME и SYS_NAME
    let result = command.execute().await.unwrap();
    println!(
        "Результат команды с переменными окружения: {}",
        result.output
    );

    // Пример с использованием файла с переменными
    let file_path = "variables.json";
    let file_content = r#"
    {
        "USER_NAME": "Alice",
        "SYS_NAME": "Variable System"
    }
    "#;

    // Создаем временный файл с переменными
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    let mut file = File::create(file_path).await.unwrap();
    file.write_all(file_content.as_bytes()).await.unwrap();

    let command = CommandBuilder::new(
        "file_greeting",
        "echo 'Привет, {#USER_NAME}! Добро пожаловать в {#SYS_NAME}'",
    )
    .variables_file(file_path)
    .build();

    // Использует переменные из файла variables.json
    let result = command.execute().await.unwrap();
    println!(
        "Результат команды с переменными из файла: {}",
        result.output
    );

    // Удаляем временный файл
    tokio::fs::remove_file(file_path).await.unwrap();

    // Пример смешанного использования
    let command = CommandBuilder::new(
        "mixed_greeting",
        "echo 'Интерактивно: {interactive}, из окружения: {$USER_NAME}, из файла: {#SYS_NAME}'",
    )
    .variables_file(file_path)
    .build();

    // Переменная {interactive} будет запрошена интерактивно,
    // {$USER_NAME} будет взята из окружения,
    // для {#SYS_NAME} сначала будет попытка взять из файла, но так как файл уже удален,
    // то значение будет запрошено интерактивно
    let result = match command.execute().await {
        Ok(r) => {
            println!("Результат смешанной команды: {}", r.output);
        }
        Err(e) => {
            println!("Ошибка выполнения смешанной команды: {}", e);
        }
    };
}

/// Пример выполнения команд в Windows
pub async fn windows_command_example() {
    // Команда для Windows
    let command = CommandBuilder::new("windows_cmd", "dir").build();

    // При выполнении в Windows покажет содержимое текущей директории
    let result = command.execute().await.unwrap();
    println!("Результат команды dir в Windows: {}", result.success);

    // Команда с переменными для Windows
    let command = CommandBuilder::new("windows_echo", "echo Привет, {name}! Это Windows!").build();

    // При выполнении запросит значение для {name}
    let result = command.execute().await.unwrap();
    println!("Результат команды echo в Windows: {}", result.output);
}
