//! OS command `Action`.
//!
//! # `cmd` attribute command wrapper.
//!
//! Specify the command to be executed in the `cmd` attribute of the `yaml` configuration
//! file, and the `Yaml` parser will package the command as a `CommandAction`, which implements
//! the `Action` trait and defines the specific logic of executing the command.

use std::{process::Command, sync::Arc};

use crate::{log, utils::EnvVar};

use super::{Action, CmdExecuteError, Input, Output, RunningError};

/// Can be used to run a command.
pub struct CommandAction {
    command: String,
}

impl CommandAction {
    pub fn new(cmd: &str) -> Self {
        Self {
            command: cmd.to_owned(),
        }
    }
}

impl Action for CommandAction {
    fn run(&self, input: Input, _env: Arc<EnvVar>) -> Result<Output, RunningError> {
        let mut args = Vec::new();
        let mut cmd = if cfg!(target_os = "windows") {
            args.push("/c");
            Command::new("cmd")
        } else {
            args.push("-c");
            Command::new("sh")
        };
        args.push(&self.command);
        input.get_iter().for_each(|input| {
            if let Some(inp) = input.get::<String>() {
                args.push(inp)
            }
        });
        let out = cmd.args(args).output().unwrap();
        if !out.stderr.is_empty() {
            let err_msg = String::from_utf8(out.stderr).unwrap();
            log::error(err_msg.clone());
            Err(CmdExecuteError::new(err_msg).into())
        } else {
            Ok(Output::new(String::from_utf8(out.stdout).unwrap()))
        }
    }
}
