use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Error, Meta};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Inline {
    None,
    Never,
    Always,
    Sometimes,
}

impl Default for Inline {
    fn default() -> Self {
        Self::None
    }
}

impl Into<TokenStream> for &Inline {
    fn into(self) -> TokenStream {
        match self {
            Inline::None => quote! {},
            Inline::Never => quote! { #[inline(never)] },
            Inline::Always => quote! { #[inline(always)] },
            Inline::Sometimes => quote! { #[inline] },
        }
    }
}

impl Into<TokenStream> for Inline {
    fn into(self) -> TokenStream {
        (&self).into()
    }
}

impl ToTokens for Inline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(<&Self as Into<TokenStream>>::into(self));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Kind {
    Setr,
    GetrRef,
    GetrCopy,
}

impl Into<&'static str> for Kind {
    fn into(self) -> &'static str {
        match self {
            Kind::Setr => "set",
            Kind::GetrRef | Kind::GetrCopy => "get",
        }
    }
}

impl Kind {
    pub(crate) fn into_ident(self, field: &Ident) -> Ident {
        format_ident!("{}_{}", <Self as Into<&'static str>>::into(self), field)
    }
}

impl TryFrom<&Meta> for Kind {
    type Error = Error;

    fn try_from(gsflag: &Meta) -> Result<Self, Self::Error> {
        match gsflag {
            // idk why I did it this way, just looks cleaner.
            _ if gsflag.path().is_ident("set") => Ok(Kind::Setr),
            _ if gsflag.path().is_ident("get") => Ok(Kind::GetrRef),
            _ if gsflag.path().is_ident("get_copy") => Ok(Kind::GetrCopy),

            _ => Err(Error::new_spanned(
                gsflag,
                "Valid values are `skip`, `get`, `get_copy`, and `set`",
            )),
        }
    }
}

impl TryFrom<Meta> for Kind {
    type Error = Error;

    fn try_from(gsflag: Meta) -> Result<Self, Self::Error> {
        (&gsflag).try_into()
    }
}
