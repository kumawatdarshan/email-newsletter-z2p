use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Fields, Ident, ItemEnum};

use crate::into_error_response::parser::{HeaderKey, HeaderPair, parse_enum};

// TODO: For now we ignore all params, but in future, we should expose
fn variant_pat(enum_ident: &Ident, ident: &Ident, fields: &Fields) -> TokenStream2 {
    match fields {
        syn::Fields::Named(_) => quote! { #enum_ident::#ident { .. } },
        syn::Fields::Unnamed(_) => quote! { #enum_ident::#ident ( .. ) },
        syn::Fields::Unit => quote! { #enum_ident::#ident },
    }
}

fn insert_header(pair: &HeaderPair) -> TokenStream2 {
    let value = &pair.value;

    let expansion = match &pair.name {
        // CASE A: User provided a constant (e.g., header::CONTENT_TYPE)
        HeaderKey::Constant(path) => quote! { #path },

        // CASE B: User provided a string (e.g., "X-My-Header")
        HeaderKey::Literal(lit) => quote! {
            ::axum::http::header::HeaderName::from_static(#lit)
        },
    };

    quote! {
        headers.insert(
            #expansion,
            ::axum::http::header::HeaderValue::from_str(&#value.to_string())
                .expect("Header value must be valid ASCII")
        );
    }
}

pub(crate) fn impl_status_code(original_enum: &mut ItemEnum) -> TokenStream2 {
    let enum_ident = original_enum.ident.clone();
    let variants =
        parse_enum(original_enum).expect("failed to parse enum attributes for IntoResponse");

    let match_arms = variants.iter().map(|v| {
        let pat = variant_pat(&enum_ident, &v.ident, &v.fields);

        let status = &v.status;
        let header_inserts = v.headers.iter().map(insert_header);

        quote! {
            #pat => {
                let mut headers = ::axum::http::HeaderMap::new();
                #(#header_inserts)*
                (headers, #status)
            }
        }
    });

    quote! {
        impl ::axum::response::IntoResponse for #enum_ident {
            fn into_response(self) -> ::axum::response::Response {
                let (mut headers, status_code) = match &self {
                    #(#match_arms)*
                };

                let body = self.to_string();

                ::tracing::error!(exception.details = ?self, exception.message = %body);

                let body = ::axum::Json(::newsletter_macros::Response {
                    error: body
                });

                ::axum::response::IntoResponse::into_response((status_code, headers, body))
            }
        }
    }
}
