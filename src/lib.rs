pub mod builder;
pub mod chain;
pub mod command;
pub mod logging;
pub mod visitor;

// Реэкспорт основных компонентов для удобства использования
pub use builder::{ChainBuilder, CommandBuilder};
pub use chain::{ChainExecutionMode, CommandChain};
pub use command::{Command, CommandExecution, CommandResult, ExecutionMode};
pub use logging::{ConsoleLogger, FileLogger, LogLevel, Logger, LoggingStrategy};
pub use visitor::{LogVisitor, Visitor};
