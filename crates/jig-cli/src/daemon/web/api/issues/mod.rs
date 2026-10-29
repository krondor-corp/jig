//! `/api/v0/issues/*`: a machine's issues and spawning from them. Each
//! request names its machine; [`Backend`] forwards it there.
//!
//! [`Backend`]: crate::daemon::web::Backend

pub mod get;
pub mod refresh;
pub mod spawn;
