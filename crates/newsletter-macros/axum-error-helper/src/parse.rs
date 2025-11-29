use deluxe::ExtractAttributes;
use syn::Expr;

/// Represents: #[status(StatusCode::XYZ)]
#[derive(ExtractAttributes, Debug)]
#[deluxe(attributes(status), default)]
pub struct StatusAttr(pub Expr);

impl Default for StatusAttr {
    fn default() -> Self {
        Self(syn::parse_quote! { ::axum::http::StatusCode::INTERNAL_SERVER_ERROR })
    }
}
