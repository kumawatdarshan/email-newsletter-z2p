#![no_std]

use proc_macro::TokenStream;
use syn::ItemEnum;

mod generate;
mod parse;

#[proc_macro_derive(IntoErrorResponse, attributes(status))]
pub fn error_macro(item: TokenStream) -> TokenStream {
    let mut input: ItemEnum = syn::parse(item).expect("expected an enum");
    let expanded = crate::generate::impl_status_code(&mut input);
    expanded.into()
}

#[proc_macro_derive(DebugChain)]
pub fn debug_chain_macro(item: TokenStream) -> TokenStream {
    let mut input: ItemEnum = syn::parse(item).expect("expected an enum");
    let expanded = crate::generate::impl_debug_chain(&mut input);
    expanded.into()
}
