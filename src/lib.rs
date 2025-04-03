pub mod builder;
pub mod chain;
pub mod command;
pub mod examples;
pub mod logging;
pub mod visitor;

// Реэкспорт основных компонентов для удобства использования
pub use builder::chain_builder::ChainBuilder;
pub use chain::{ChainExecutionMode, CommandChain};
pub use command::{Command, CommandResult, ExecutionMode, ShellCommand};
pub use logging::{ConsoleLogger, FileLogger, LogLevel, Logger, LoggerManager};
pub use visitor::{LogVisitor, Visitor};
