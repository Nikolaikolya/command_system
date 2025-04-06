use command_system::builder::CommandBuilder;
use command_system::CommandExecution;

#[tokio::main]
async fn main() {
    println!("=== Тестирование интерактивного ввода и переменных ===");

    // Устанавливаем переменные окружения для теста
    std::env::set_var("TEST_NAME", "Test User");
    std::env::set_var("TEST_SYSTEM", "Command System");

    // Создаем файл с переменными
    let vars_content = r#"
    {
        "FILE_NAME": "File User",
        "FILE_SYSTEM": "File System"
    }
    "#;

    tokio::fs::write("test_vars.json", vars_content)
        .await
        .unwrap();

    // Пример с интерактивным вводом
    println!("\n=== Интерактивный ввод ===");
    println!("Вам будет предложено ввести имя и название системы");

    let command = CommandBuilder::new(
        "interactive_test",
        "echo 'Привет, {name}! Добро пожаловать в {system}'",
    )
    .build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пример с переменными окружения
    println!("\n=== Переменные окружения ===");
    let command = CommandBuilder::new(
        "env_test",
        "echo 'Привет, {$TEST_NAME}! Добро пожаловать в {$TEST_SYSTEM}'",
    )
    .build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пример с переменными из файла
    println!("\n=== Переменные из файла ===");
    let command = CommandBuilder::new(
        "file_test",
        "echo 'Привет, {#FILE_NAME}! Добро пожаловать в {#FILE_SYSTEM}'",
    )
    .variables_file("test_vars.json")
    .build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Смешанный пример
    println!("\n=== Смешанный вариант ===");
    let command = CommandBuilder::new(
        "mixed_test",
        "echo 'Интерактивно: {interactive_var}, из env: {$TEST_NAME}, из файла: {#FILE_SYSTEM}'",
    )
    .variables_file("test_vars.json")
    .build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Проверка отсутствующих переменных (запросит интерактивно)
    println!("\n=== Отсутствующие переменные ===");
    let command = CommandBuilder::new(
        "missing_test",
        "echo 'Отсутствует в env: {$MISSING_VAR}, отсутствует в файле: {#MISSING_VAR}'",
    )
    .variables_file("test_vars.json")
    .build();

    match command.execute().await {
        Ok(result) => println!("Результат: {}", result.output),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Удаляем файл с переменными
    tokio::fs::remove_file("test_vars.json").await.unwrap();
    println!("\n=== Тестирование завершено ===");
}
