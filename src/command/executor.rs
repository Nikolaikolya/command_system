use super::traits::{Command, CommandError};
use crate::chain::command_chain::ChainResult;
use crate::command::ExecutionMode;
use futures::future::try_join_all;

pub struct CommandExecutor {
    mode: ExecutionMode,
}

impl CommandExecutor {
    pub fn new(mode: ExecutionMode) -> Self {
        Self { mode }
    }

    pub async fn execute(
        &self,
        commands: &[Box<dyn Command>],
    ) -> Result<ChainResult, CommandError> {
        match self.mode {
            ExecutionMode::Sequential => self.execute_sequential(commands).await,
            ExecutionMode::Parallel => self.execute_parallel(commands).await,
        }
    }

    async fn execute_sequential(
        &self,
        commands: &[Box<dyn Command>],
    ) -> Result<ChainResult, CommandError> {
        let mut results = Vec::new();

        for command in commands {
            let result = command.execute().await?;
            results.push(result);
        }

        Ok(ChainResult {
            success: true,
            results,
            error: None,
        })
    }

    async fn execute_parallel(
        &self,
        commands: &[Box<dyn Command>],
    ) -> Result<ChainResult, CommandError> {
        let futures: Vec<_> = commands.iter().map(|cmd| cmd.execute()).collect();
        let results = try_join_all(futures).await?;

        Ok(ChainResult {
            success: true,
            results,
            error: None,
        })
    }
}
