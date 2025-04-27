use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::enums::{Inline, Kind};
use proc_macro2::TokenStream;
use syn::{Error, Expr, ExprLit, Ident, Lit, Meta, Type, Visibility};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct FuncProps {
    pub(crate) kind: Kind,
    // FuncProps must be unique (globally?) on `name`
    // Decision: Leave to user; a HashSet on FuncProps's derived `Hash` is good enough
    pub(crate) name: Ident,
    pub(crate) inline: Inline,
    pub(crate) vis: Visibility,
}

#[derive(Clone)]
pub(crate) struct OptFuncPropsWithKind {
    pub(crate) kind: Kind,
    pub(crate) optfuncprops: OptFuncProps,
}

impl OptFuncPropsWithKind {
    pub(crate) fn build(self, field: &Ident) -> FuncProps {
        self.optfuncprops.build(self.kind, field)
    }
}

impl Deref for OptFuncPropsWithKind {
    type Target = OptFuncProps;

    fn deref(&self) -> &Self::Target {
        &self.optfuncprops
    }
}

impl DerefMut for OptFuncPropsWithKind {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.optfuncprops
    }
}

#[derive(Clone)]
pub(crate) struct OptFuncProps {
    pub(crate) name: Option<Ident>,
    pub(crate) inline: Option<Inline>,
    pub(crate) vis: Option<Visibility>,
}

impl Default for OptFuncProps {
    fn default() -> Self {
        Self::new()
    }
}

impl OptFuncProps {
    pub(crate) fn new() -> Self {
        Self {
            vis: None,
            name: None,
            inline: None,
        }
    }

    pub(crate) fn or(self, other: OptFuncProps) -> Self {
        Self {
            vis: self.vis.or(other.vis),
            name: self.name.or(other.name),
            inline: self.inline.or(other.inline),
        }
    }

    pub(crate) fn build(self, kind: Kind, field: &Ident) -> FuncProps {
        FuncProps {
            kind, // Trivially copyable
            inline: self.inline.unwrap_or_default(),
            vis: self.vis.unwrap_or(Visibility::Inherited),
            name: self.name.unwrap_or(kind.into_ident(field)),
        }
    }
}

impl TryFrom<Meta> for OptFuncProps {
    type Error = Error;

    fn try_from(setting: Meta) -> Result<Self, Self::Error> {
        match setting {
            Meta::NameValue(mnv) if mnv.path.is_ident("rename") => {
                let rename = match mnv.value {
                    Expr::Lit(ExprLit {
                        attrs: _attrs,
                        lit: Lit::Str(str),
                    }) => Ok(Ident::new(str.value().as_str(), str.span())),
                    _ => Err(Error::new_spanned(
                        mnv.value,
                        "Valid gsflag setting is `rename = \"name\"`",
                    )),
                }?;

                Ok(OptFuncProps {
                    name: Some(rename),
                    ..Default::default()
                })
            }
            Meta::Path(path)
                if path.is_ident("inline")
                    || path.is_ident("inline_always")
                    || path.is_ident("inline_never") =>
            {
                let inline = if path.is_ident("inline") {
                    Inline::Sometimes
                } else if path.is_ident("inline_always") {
                    Inline::Always
                } else if path.is_ident("inline_never") {
                    Inline::Never
                } else {
                    unreachable!()
                };

                Ok(OptFuncProps {
                    inline: Some(inline),
                    ..Default::default()
                })
            }
            Meta::NameValue(mnv) if mnv.path.is_ident("vis") => {
                let vis = match mnv.value {
                    Expr::Lit(ExprLit {
                        attrs: _attrs,
                        lit: Lit::Str(str),
                    }) => Ok(syn::parse2::<Visibility>(str.parse::<TokenStream>()?)?),
                    _ => Err(Error::new_spanned(
                        mnv.value,
                        "Valid gsflag setting is `vis = \"pub(crate)\"`",
                    )),
                }?;

                Ok(OptFuncProps {
                    vis: Some(vis),
                    ..Default::default()
                })
            }
            _ => Err(Error::new_spanned(
                setting,
                "Invalid usage, see `README.md`",
            )),
        }
    }
}

pub(crate) struct FieldProps {
    pub(crate) ty: Type,
    pub(crate) all_skip: bool,
    pub(crate) props: HashSet<FuncProps>,
}
