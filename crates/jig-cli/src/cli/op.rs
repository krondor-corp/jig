//! Op trait — typed command pattern for CLI operations
//!
//! Every CLI command implements `Op`: it declares a `Context` shape, builds
//! that context via `build_context`, then runs with `run(ctx)`. Formatting
//! lives in `Display` impls on the output types.

use std::error::Error;
use std::fmt::Display;

use crate::context::AppPaths;

/// Where a command's tracing output goes. Decided per invocation by
/// [`Op::log_sink`], before the command runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSink {
    /// `warn` and above to stderr: one-off commands, where a warning should
    /// be seen by the person who ran it. The default.
    Stderr,
    /// `info` and above to a log file under `state/logs/`: processes that
    /// own the terminal (`ps --watch`) or run without one (`daemon start`).
    File,
}

/// Trait for CLI operations.
pub trait Op {
    type Context;
    type Error: Error + Send + Sync + 'static;
    type Output: Display;

    /// Build this command's context. `paths` is where jig keeps its files,
    /// resolved from the environment once by `main`.
    fn build_context(&self, paths: &AppPaths) -> Result<Self::Context, Self::Error>;
    fn run(&self, ctx: Self::Context) -> Result<Self::Output, Self::Error>;

    /// Where this invocation's tracing output goes. Override only when
    /// stderr would be wrong — see [`LogSink::File`].
    fn log_sink(&self) -> LogSink {
        LogSink::Stderr
    }
}

/// Unit output for commands that only produce stderr
#[derive(Debug, Default)]
pub struct NoOutput;

impl Display for NoOutput {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// Macro to generate Command enum with Op implementation
#[macro_export]
macro_rules! command_enum {
    ($($(#[$attr:meta])* ($variant:ident, $type:ty)),* $(,)?) => {
        #[derive(clap::Subcommand, Debug, Clone)]
        #[allow(clippy::large_enum_variant)]
        pub enum Command {
            $(
                $(#[$attr])*
                $variant($type),
            )*
        }

        #[derive(Debug)]
        #[allow(clippy::large_enum_variant)]
        pub enum OpOutput {
            $($variant(<$type as $crate::cli::op::Op>::Output),)*
        }

        #[derive(Debug, thiserror::Error)]
        pub enum OpError {
            $(
                #[error(transparent)]
                $variant(<$type as $crate::cli::op::Op>::Error),
            )*
        }

        impl $crate::cli::op::Op for Command {
            type Context = $crate::context::AppPaths;
            type Output = OpOutput;
            type Error = OpError;

            fn build_context(
                &self,
                paths: &$crate::context::AppPaths,
            ) -> Result<$crate::context::AppPaths, Self::Error> {
                Ok(paths.clone())
            }

            fn run(&self, paths: $crate::context::AppPaths) -> Result<Self::Output, Self::Error> {
                match self {
                    $(
                        Command::$variant(op) => {
                            let ctx = op.build_context(&paths).map_err(OpError::$variant)?;
                            op.run(ctx)
                                .map(OpOutput::$variant)
                                .map_err(OpError::$variant)
                        },
                    )*
                }
            }

            fn log_sink(&self) -> $crate::cli::op::LogSink {
                match self {
                    $(Command::$variant(op) => op.log_sink(),)*
                }
            }
        }

        impl std::fmt::Display for OpOutput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        OpOutput::$variant(output) => write!(f, "{}", output),
                    )*
                }
            }
        }
    };
}
