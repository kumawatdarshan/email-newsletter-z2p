use deluxe::{ExtractAttributes, ParseMetaItem};
use proc_macro2::Span;
use syn::{Expr, Fields, Ident, ItemEnum, LitStr, Path, Token, Variant, parse::ParseStream};

/// Represents:
/// ```rs
/// #[status(StatusCode::XYZ)]
/// ```
#[derive(ExtractAttributes, Debug)]
#[deluxe(attributes(status), default)]
pub struct StatusAttr(pub Expr);

impl Default for StatusAttr {
    fn default() -> Self {
        Self(syn::parse_quote! { ::axum::http::StatusCode::INTERNAL_SERVER_ERROR })
    }
}

///  Represents:
/// ```rs
/// #[headers([
///     "WWW-Authenticate" = "Basic realm=\"newsletter\"",
///     "X-Reason" = reason
/// ]
/// )]
/// ```
#[derive(ExtractAttributes, Debug, Default)]
#[deluxe(attributes(headers), default)]
pub struct HeadersAttr(pub Vec<HeaderPair>);

#[derive(Debug)]
pub struct HeaderPair {
    pub name: HeaderKey,
    pub value: Expr,
}

#[derive(Debug)]
pub enum HeaderKey {
    Constant(Path),
    Literal(LitStr),
}

impl ParseMetaItem for HeaderPair {
    fn parse_meta_item(input: ParseStream, _mode: deluxe::ParseMode) -> deluxe::Result<Self> {
        let name = if input.peek(syn::LitStr) {
            HeaderKey::Literal(input.parse()?)
        } else if input.peek(syn::Ident) || input.peek(Token![::]) {
            HeaderKey::Constant(input.parse()?)
        } else {
            return Err(input.error("Header key must be a constant (Path) or string literal"));
        };

        input.parse::<Token![=]>()?;

        let value = input.parse()?;

        Ok(HeaderPair { name, value })
    }
}

#[derive(Debug)]
pub struct VariantMeta {
    pub ident: Ident,
    pub fields: Fields,
    pub status: Expr,
    pub headers: Vec<HeaderPair>,
}

pub fn parse_enum(original_enum: &mut ItemEnum) -> syn::Result<Vec<VariantMeta>> {
    original_enum
        .variants
        .iter_mut()
        .map(parse_variant)
        .collect()
}

fn parse_variant(variant: &mut Variant) -> syn::Result<VariantMeta> {
    let StatusAttr(status) = deluxe::extract_attributes(variant)
        .map_err(to_syn_error("StatusCode extraction failed"))?;

    let HeadersAttr(headers) =
        deluxe::extract_attributes(variant).map_err(to_syn_error("Headers extraction failed"))?;

    Ok(VariantMeta {
        ident: variant.ident.clone(),
        fields: variant.fields.clone(),
        status,
        headers, // Vec<(Expr, Expr)> if that’s what HeadersAttr gives
    })
}

fn to_syn_error(msg: &'static str) -> impl Fn(deluxe::Error) -> syn::Error {
    move |e| syn::Error::new(Span::call_site(), format!("{msg}: {e}"))
}
