# Command System

Библиотека для выполнения команд с использованием паттернов проектирования.

## Основные возможности

- **Паттерн Команда**: Выполнение shell-команд или пользовательских команд
- **Паттерн Цепочка обязанностей**: Последовательное или параллельное выполнение команд
- **Паттерн Строитель**: Удобное конструирование команд и цепочек
- **Паттерн Визитор**: Обход команд для логирования или других операций
- **Паттерн Стратегия**: Гибкое логирование в консоль, файл или обе цели
- **Кроссплатформенность**: Поддержка как Linux, так и Windows
- **Интерактивный ввод**: Возможность запрашивать переменные у пользователя или брать их из окружения или файла

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

### Использование интерактивного ввода и переменных

Библиотека поддерживает три варианта подстановки переменных:

1. **Интерактивный ввод** - `{name}`: Запрашивает значение у пользователя
2. **Переменные окружения** - `{$NAME}`: Берет значение из переменной окружения
3. **Переменные из файла** - `{#NAME}`: Берет значение из JSON файла

Если переменной нет в окружении или в файле, значение будет запрошено интерактивно.

```rust
use command_system::CommandBuilder;

#[tokio::main]
async fn main() {
    // Интерактивный ввод
    let command = CommandBuilder::new(
        "greeting", 
        "echo 'Привет, {name}! Добро пожаловать в {system}'"
    ).build();
    
    // При выполнении запросит значения для {name} и {system}
    let result = command.execute().await.unwrap();
    
    // Использование переменных окружения
    let command = CommandBuilder::new(
        "env_greeting", 
        "echo 'Привет, {$USER}! Добро пожаловать в {$SYSTEM}'"
    ).build();
    
    // Использование переменных из файла
    let command = CommandBuilder::new(
        "file_greeting", 
        "echo 'Привет, {#USER}! Добро пожаловать в {#SYSTEM}'"
    )
    .variables_file("variables.json")
    .build();
}
```

Пример файла переменных (variables.json):
```json
{
  "USER": "Alice",
  "SYSTEM": "Command System",
  "VERSION": "1.0"
}
```

Вы можете комбинировать все типы подстановок в одной команде.

## Кроссплатформенность

Библиотека автоматически определяет операционную систему и использует соответствующий интерпретатор команд:

- **Linux/macOS**: `/bin/sh -c "command"`
- **Windows**: `cmd.exe /C "command"`

## Архитектура библиотеки

Библиотека состоит из следующих модулей:

- `command` :    Определяет интерфейс команд и базовые реализации
- `chain`   :    Реализует цепочку выполнения команд
- `logging` :    Стратегии логирования и реализации логгеров
- `builder` :    Строители для создания команд и цепочек
- `visitor` :    Интерфейс посетителя и конкретные реализации
- `examples`:    Примеры использования

## Лицензия

MIT 