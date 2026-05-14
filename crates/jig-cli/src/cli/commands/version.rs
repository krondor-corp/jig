//! Version command - show version information

use std::convert::Infallible;

use clap::Args;

use crate::cli::op::{NoOutput, Op};
use crate::cli::ui;

/// Show version information
#[derive(Args, Debug, Clone)]
pub struct Version;

impl Op for Version {
    type Context = ();
    type Error = Infallible;
    type Output = NoOutput;

    fn build_context(&self) -> Result<(), Infallible> {
        Ok(())
    }

    fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
        eprintln!(
            "{} {}",
            ui::bold("jig"),
            ui::highlight(env!("CARGO_PKG_VERSION"))
        );
        Ok(NoOutput)
    }
}
