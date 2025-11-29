use crate::parse::StatusAttr;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemEnum;

pub(crate) fn impl_status_code(original_enum: &mut ItemEnum) -> TokenStream2 {
    let enum_ident = &original_enum.ident;
    let arms = original_enum.variants.iter_mut().map(|variant| {
        let StatusAttr(status_code) =
            deluxe::extract_attributes(variant).expect("StatusCode extraction failed");
        let error_variant = &variant.ident;
        let fields = &variant.fields;
        match fields {
            syn::Fields::Named(_) => quote! {
                #enum_ident::#error_variant{ .. } => #status_code,
            },
            syn::Fields::Unnamed(_) => quote! {
                #enum_ident::#error_variant( .. ) => #status_code,
            },
            syn::Fields::Unit => quote! {
                #enum_ident::#error_variant => #status_code,
            },
        }
    });

    quote! {
        impl ::axum::response::IntoResponse for #enum_ident {
            fn into_response(self) -> ::axum::response::Response {
                let status_code = match &self {
                    #(#arms)*
                };
                let body = self.to_string();

                ::tracing::error!(exception.details = ?self, exception.message = %body);

                let body = ::axum::Json(::serde_json::json!({
                    "status_code": status_code.as_u16(),
                    "error": body
                }));

                ::axum::response::IntoResponse::into_response((status_code, body))
            }
        }
    }
}

pub(crate) fn impl_debug_chain(original_enum: &mut ItemEnum) -> TokenStream2 {
    let enum_ident = &original_enum.ident;

    quote! {
        impl ::std::fmt::Debug for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                ::newsletter_macros::write_error_chain(f, self)
            }
        }
    }
}
