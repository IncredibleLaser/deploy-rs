use std::fmt;
use std::fmt::Debug;
use thiserror::Error;
use tokio::process::Command as TokioCommand;

pub trait HasCommandError {
    fn title() -> String;
}

#[derive(Error, Debug)]
pub enum CommandError<T: fmt::Debug + fmt::Display + HasCommandError> {
    RunError(std::io::Error),
    Exit(Option<i32>, String),
    OtherError(T),
}

impl<T: fmt::Debug + fmt::Display + HasCommandError> fmt::Display for CommandError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::RunError(err) => write!(
                f,
                "Failed to run {} command: {}",
                T::title(),
                err,
            ),
            CommandError::Exit(exit_code, cmd) => write!(
                f,
                "{} command resulted in a bad exit code: {:?}. The failed command is provided below:\n{}",
                T::title(),
                exit_code,
                cmd,
            ),
            CommandError::OtherError(err) => write!(f, "{}", err),
        }
    }
}

/// A wrapper over `tokio::process::Command` to provide the `run` method commonly used by `deploy`.
#[derive(Debug)]
pub struct Command {
    pub command: TokioCommand,
}

impl Command {
    pub fn new(command: TokioCommand) -> Command {
        Command { command }
    }

    pub async fn run<T: fmt::Debug + fmt::Display + HasCommandError>(
        &mut self,
    ) -> Result<std::process::Output, CommandError<T>> {
        let output = self
            .command
            .output()
            .await
            .map_err(CommandError::RunError)?;
        match output.status.code() {
            Some(0) => Ok(output),
            exit_code => Err(CommandError::Exit(exit_code, format!("{:?}", self.command))),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.command.fmt(f)
    }
}
