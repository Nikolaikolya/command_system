use command_system::builder::CommandBuilder;
use command_system::CommandExecution;

#[tokio::main]
async fn main() {
    println!("=== Тестирование команд в Windows ===");

    // Проверка команды dir
    println!("\n=== Команда dir ===");
    let command = CommandBuilder::new("dir_test", "dir").build();

    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Первые 3 строки вывода:");
            let lines: Vec<&str> = result.output.lines().take(3).collect();
            for line in lines {
                println!("{}", line);
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }

    // Проверка команды echo
    println!("\n=== Команда echo ===");
    let command = CommandBuilder::new("echo_test", "echo Hello Windows!").build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Проверка команды с интерактивным вводом в Windows
    println!("\n=== Интерактивный ввод в Windows ===");
    let command =
        CommandBuilder::new("win_interactive", "echo Привет, {name}! Это Windows!").build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Проверка команды whoami
    println!("\n=== Команда whoami ===");
    let command = CommandBuilder::new("user_test", "whoami").build();

    match command.execute().await {
        Ok(result) => println!("Текущий пользователь: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Проверка команды с использованием переменных окружения
    println!("\n=== Переменные окружения в Windows ===");
    let command = CommandBuilder::new(
        "env_win_test",
        "echo Системная переменная PATH: %PATH%, интерактивная переменная: {win_var}",
    )
    .build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\n=== Тестирование завершено ===");
}
