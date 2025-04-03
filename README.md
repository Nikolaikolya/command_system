# Command System

Библиотека для выполнения команд с использованием паттернов проектирования.

## Основные возможности

- **Паттерн Команда**: Выполнение shell-команд или пользовательских команд
- **Паттерн Цепочка обязанностей**: Последовательное или параллельное выполнение команд
- **Паттерн Строитель**: Удобное конструирование команд и цепочек
- **Паттерн Визитор**: Обход команд для логирования или других операций
- **Паттерн Стратегия**: Гибкое логирование в консоль, файл или обе цели

## Установка

Добавьте зависимость в ваш Cargo.toml:

```toml
[dependencies]
command_system = { git = "https://github.com/Nikolaikolya/command_system.git" }
```

## Примеры использования

### Выполнение простой команды

```rust
use command_system::{CommandBuilder, ExecutionMode};

#[tokio::main]
async fn main() {
    // Создание команды
    let command = CommandBuilder::new("list_files", "ls -la")
        .working_dir("/tmp")
        .build();
    
    // Выполнение команды
    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Вывод: {}", result.output);
        },
        Err(err) => {
            println!("Ошибка: {}", err);
        }
    }
}
```

### Выполнение цепочки команд

```rust
use command_system::{
    ChainBuilder,
    CommandBuilder,
    ChainExecutionMode,
    ConsoleLogger,
    LogLevel
};

#[tokio::main]
async fn main() {
    // Создание логгера
    let logger = Box::new(ConsoleLogger::new(LogLevel::Info));
    
    // Создание цепочки команд
    let mut chain = ChainBuilder::new("setup_chain")
        .execution_mode(ChainExecutionMode::Sequential)
        .logger(logger)
        .rollback_on_error(true)
        .build();
    
    // Добавление команд в цепочку
    chain.add_command(
        CommandBuilder::new("create_dir", "mkdir -p /tmp/app")
            .rollback("rm -rf /tmp/app")
            .build()
    );
    
    chain.add_command(
        CommandBuilder::new("copy_files", "cp -r ./src/* /tmp/app/")
            .build()
    );
    
    // Выполнение цепочки
    match chain.execute().await {
        Ok(result) => {
            if result.success {
                println!("Все команды выполнены успешно");
            } else {
                println!("Ошибка: {:?}", result.error);
            }
        },
        Err(err) => {
            println!("Критическая ошибка: {}", err);
        }
    }
}
```

### Параллельное выполнение команд

```rust
use command_system::{
    ChainBuilder,
    CommandBuilder,
    ChainExecutionMode,
    ExecutionMode
};

#[tokio::main]
async fn main() {
    let mut chain = ChainBuilder::new("parallel_jobs")
        .execution_mode(ChainExecutionMode::Parallel)
        .build();
    
    // Добавляем несколько параллельных задач
    for i in 1..=5 {
        chain.add_command(
            CommandBuilder::new(&format!("task_{}", i), &format!("echo 'Task {} started'; sleep {}; echo 'Task {} completed'", i, i, i))
                .execution_mode(ExecutionMode::Parallel)
                .timeout(10)
                .build()
        );
    }
    
    let result = chain.execute().await.unwrap();
    println!("Все задачи завершены: {}", result.success);
}
```

## Архитектура библиотеки

Библиотека состоит из следующих модулей:

- `command` :    Определяет интерфейс команд и базовые реализации
- `chain`   :    Реализует цепочку выполнения команд
- `logging` :    Стратегии логирования и реализации логгеров
- `builder` :    Строители для создания команд и цепочек
- `visitor` :    Интерфейс посетителя и конкретные реализации
- `examples`:    Интерфейс посетителя и конкретные реализации

## Лицензия

MIT 