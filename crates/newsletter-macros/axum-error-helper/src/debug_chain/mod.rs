use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemEnum;

pub(super) fn impl_debug_chain(original_enum: &mut ItemEnum) -> TokenStream2 {
    let enum_ident = &original_enum.ident;

    quote! {
        impl ::std::fmt::Debug for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                ::newsletter_macros::write_error_chain(f, self)
            }
        }
    }
}
