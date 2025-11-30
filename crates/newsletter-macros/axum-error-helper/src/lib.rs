use proc_macro::TokenStream;
use syn::ItemEnum;

mod debug_chain;
mod into_error_response;

/// # Implements `axum::response::IntoResponse`
///
/// Also integrates tracing and outputs json with the following schema
/// ```rust
/// struct ResponseBody {
///   status_code: u16,
///   error: String,
/// }
/// ```
///
/// # Usage
///
/// ```rs
/// #[derive(IntoErrorResponse)]
/// pub enum Error {
///     #[status(StatusCode::UNPROCESSABLE_ENTITY)]
///     ValidationError(String),
///     #[status(StatusCode::INTERNAL_SERVER_ERROR)]
///     UnexpectedError(#[from] anyhow::Error),
/// }
#[proc_macro_derive(IntoErrorResponse, attributes(status, headers))]
pub fn error_macro(item: TokenStream) -> TokenStream {
    let mut input: ItemEnum = syn::parse(item).expect("expected an enum");
    let expanded = crate::into_error_response::impl_status_code(&mut input);
    expanded.into()
}

/// # Implements `std::fmt::Debug`
///
/// A custom Debug implementation for `Error` enums, currently you cannot override what impl to use.
/// But I have some Ideas on how you can do so.
///
/// # Usage
///
/// For this error
/// ```rs
/// MyError::Network(io::Error::new(ErrorKind::TimedOut, "read timeout"))
/// ```
/// Default Debug impl might look something like:
/// ```rs
/// Network(
///     Custom {
///         kind: TimedOut,
///         error: "read timeout",
///     },
/// )
/// ```
/// While the DebugChain might look like:
/// ```Error
/// network failure
///   + read timeout
/// ```
#[proc_macro_derive(DebugChain)]
pub fn debug_chain_macro(item: TokenStream) -> TokenStream {
    let mut input: ItemEnum = syn::parse(item).expect("expected an enum");
    let expanded = crate::debug_chain::impl_debug_chain(&mut input);
    expanded.into()
}
