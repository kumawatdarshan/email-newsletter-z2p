pub mod health;
pub mod newsletters;
pub mod routes;
pub mod subscriptions;
pub mod subscriptions_confirm;

use axum::http::{StatusCode, Uri};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{self},
};
use tracing::warn;

// re-exports
pub use routes::get_router;

// Only for debugging. Should be removed in production to declutter the logs.
async fn handle_404(uri: Uri) -> StatusCode {
    warn!("Route not found: {}", uri);
    StatusCode::NOT_FOUND
}

#[derive(Serialize)]
pub struct ResponseMessage {
    message: String,
}

pub trait FormatterExt {
    fn write_error_chain(&mut self, e: &impl Error) -> fmt::Result;
}

impl FormatterExt for fmt::Formatter<'_> {
    fn write_error_chain(&mut self, e: &impl Error) -> fmt::Result {
        writeln!(self, "{e}")?;
        let mut cause = e.source();
        let mut depth = 1;

        while let Some(err) = cause {
            writeln!(self, "{:>width$}+ {err}", "", width = depth * 2)?;
            cause = err.source();
            depth += 1;
        }
        Ok(())
    }
}
