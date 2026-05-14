//! Which command - show path to jig executable

use clap::Args;
use std::path::PathBuf;

use crate::cli::op::Op;

/// Show path to jig executable
#[derive(Args, Debug, Clone)]
pub struct Which;

#[derive(Debug)]
pub struct WhichOutput(PathBuf);

impl std::fmt::Display for WhichOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WhichError {
    #[error("Failed to get executable path: {0}")]
    IoError(#[from] std::io::Error),
}

impl Op for Which {
    type Context = ();
    type Error = WhichError;
    type Output = WhichOutput;

    fn build_context(&self) -> Result<(), WhichError> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        let exe = std::env::current_exe()?;
        Ok(WhichOutput(exe))
    }
}
