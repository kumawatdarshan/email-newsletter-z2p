pub use axum_error_helper::{DebugChain, IntoErrorResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    pub error: String,
}

pub fn write_error_chain(
    f: &mut core::fmt::Formatter,
    error: &dyn core::error::Error,
) -> core::fmt::Result {
    writeln!(f, "{error}")?;
    let mut cause = error.source();
    let mut depth = 1;

    while let Some(err) = cause {
        writeln!(f, "{:>width$}+ {err}", "", width = depth * 2)?;
        cause = err.source();
        depth += 1;
    }
    Ok(())
}
