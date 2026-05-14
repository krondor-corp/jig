//! Op trait — typed command pattern for CLI operations
//!
//! Every CLI command implements `Op`: it declares a `Context` shape, builds
//! that context via `build_context`, then runs with `run(ctx)`. Formatting
//! lives in `Display` impls on the output types.

use std::error::Error;
use std::fmt::Display;

/// Trait for CLI operations.
pub trait Op {
    type Context;
    type Error: Error + Send + Sync + 'static;
    type Output: Display;

    fn build_context(&self) -> Result<Self::Context, Self::Error>;
    fn run(&self, ctx: Self::Context) -> Result<Self::Output, Self::Error>;
}

/// Unit output for commands that only produce stderr
#[derive(Debug, Default)]
pub struct NoOutput;

impl Display for NoOutput {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// Macro to generate command enums with Op implementations.
///
/// Two forms:
/// 1. Top-level: variants have distinct Output/Error types, the macro
///    generates wrapping `OpOutput`/`OpError` enums.
/// 2. Subcommand (`$enum: $output, $error`): variants share Output/Error,
///    no wrapping enums are generated — the impl returns them directly.
#[macro_export]
macro_rules! command_enum {
    // Subcommand form: all variants share the given Output/Error types.
    ($enum:ident: $output:ty, $error:ty {
        $($(#[$attr:meta])* ($variant:ident, $type:ty)),* $(,)?
    }) => {
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum $enum {
            $(
                $(#[$attr])*
                $variant($type),
            )*
        }

        impl $crate::cli::op::Op for $enum {
            type Context = ();
            type Output = $output;
            type Error = $error;

            fn build_context(&self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn run(&self, _ctx: ()) -> Result<Self::Output, Self::Error> {
                match self {
                    $(
                        $enum::$variant(op) => {
                            let ctx = op.build_context()?;
                            op.run(ctx)
                        },
                    )*
                }
            }
        }
    };

    // Top-level form: generates wrapping OpOutput/OpError enums.
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
            type Context = ();
            type Output = OpOutput;
            type Error = OpError;

            fn build_context(&self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn run(&self, _ctx: ()) -> Result<Self::Output, Self::Error> {
                match self {
                    $(
                        Command::$variant(op) => {
                            let ctx = op.build_context().map_err(OpError::$variant)?;
                            op.run(ctx)
                                .map(OpOutput::$variant)
                                .map_err(OpError::$variant)
                        },
                    )*
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
