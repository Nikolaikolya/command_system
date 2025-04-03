/// Трейт посетителя для реализации паттерна Visitor
pub trait Visitor {
    /// Посещает shell команду
    fn visit_shell_command(&mut self, command: &crate::command::ShellCommand);

    /// Посещает составную команду
    fn visit_composite_command(&mut self, command: &crate::command::CompositeCommand);
}
