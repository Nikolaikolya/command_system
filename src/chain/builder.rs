use super::{ChainExecutionMode, CommandChain};
use crate::logging::Logger;
use std::sync::Arc;

pub struct ChainBuilder {
    name: String,
    execution_mode: ChainExecutionMode,
    logger: Option<Arc<Box<dyn Logger>>>,
    rollback_on_error: bool,
}

impl ChainBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            execution_mode: ChainExecutionMode::Sequential,
            logger: None,
            rollback_on_error: false,
        }
    }

    pub fn execution_mode(mut self, mode: ChainExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    pub fn logger(mut self, logger: Arc<Box<dyn Logger>>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn rollback_on_error(mut self, rollback: bool) -> Self {
        self.rollback_on_error = rollback;
        self
    }

    pub fn build(self) -> CommandChain {
        CommandChain::new(
            &self.name,
            self.execution_mode,
            self.logger,
            self.rollback_on_error,
        )
    }
}
