use std::collections::HashSet;

use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, Error, Expr, ExprLit, Fields, Ident, ItemStruct, Lit, Meta, Result,
    Token,
};

pub fn expand_get_set(mut input: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let fields = if let Fields::Named(fields_named) = &mut input.fields {
        &mut fields_named.named
    } else {
        return Err(Error::new_spanned(input, "Only named fields are supported"));
    };

    let mut implblk = vec![];

    for field in fields {
        let field_name = field.ident.clone().unwrap();
        let field_type = &field.ty;
        let mut found_gsflags = false;

        let mut remove_attr = None;

        for (i, attr) in field.attrs.iter_mut().enumerate() {
            match attr.meta {
                // gsflags(get, set, get_copy(rename = "draft", noinline, pub(crate)))
                syn::Meta::List(ref gsflags_ml) if gsflags_ml.path.is_ident("gsflags") => {
                    if found_gsflags {
                        return Err(Error::new_spanned(
                            attr,
                            "gsflags has already been defined for this field and cannot be defined again."
                        ));
                    } else {
                        found_gsflags = true;
                    }

                    remove_attr = Some(i);

                    // [get, set, get_copy(rename = "draft", noinline, pub(crate))]
                    let gsflags: Punctuated<Meta, Token![,]> =
                        gsflags_ml.parse_args_with(Punctuated::parse_terminated)?;

                    let mut seen_kinds: HashSet<GSKind> = HashSet::new();

                    // get, set, get_copy(rename = "draft", noinline, pub(crate))
                    for gsflag in gsflags {
                        let kind = match gsflag {
                            _ if gsflag.path().is_ident("set") => GSKind::Setr,
                            _ if gsflag.path().is_ident("get") => GSKind::GetrRef,
                            _ if gsflag.path().is_ident("get_copy") => GSKind::GetrCopy,
                            _ => {
                                return Err(Error::new_spanned(
                                    gsflag,
                                    "Valid attributes are `get`, `get_copy`, and `set`",
                                ))
                            }
                        };

                        if !seen_kinds.insert(kind.clone()) {
                            return Err(Error::new_spanned(
                                gsflag,
                                format!("{kind:?} has been seen before for this field! Do not reuse field attributes.")
                            ));
                        }

                        let mut rename = None;
                        let mut inline = None;
                        let mut vis = None;

                        // get_copy(rename = "draft", noinline, pub(crate))
                        if let Meta::List(gsflag_settings_ml) = gsflag {
                            // [rename = "draft", noinline]
                            let gsflag_settings: Punctuated<Meta, Token![,]> =
                                gsflag_settings_ml.parse_args_with(Punctuated::parse_terminated)?;

                            // rename = "draft", noinline, pub(crate)
                            for setting in gsflag_settings {
                                match setting {
                                    Meta::NameValue(mnv) if mnv.path.is_ident("rename") => {
                                        if rename.is_none() {
                                            rename = match mnv.value {
                                                Expr::Lit(ExprLit {
                                                    attrs: _attrs,
                                                    lit: Lit::Str(str),
                                                }) => Some(Ident::new(
                                                    str.value().as_str(),
                                                    str.span(),
                                                )),
                                                _ => {
                                                    return Err(Error::new_spanned(
                                                        mnv.value,
                                                        "Valid gsflag setting is `rename = \"name\"`",
                                                    ));
                                                }
                                            };
                                        } else {
                                            return Err(Error::new_spanned(
                                                mnv,
                                                "This gsflag has already been renamed and cannot be renamed again."
                                            ));
                                        }
                                    }
                                    Meta::Path(path)
                                        if path.is_ident("inline")
                                            || path.is_ident("inline_always")
                                            || path.is_ident("inline_never") =>
                                    {
                                        if inline.is_none() {
                                            inline = Some(if path.is_ident("inline") {
                                                quote! { #[inline] }
                                            } else if path.is_ident("inline_always") {
                                                quote! { #[inline(always)] }
                                            } else if path.is_ident("inline_never") {
                                                quote! { #[inline(never)] }
                                            } else {
                                                unreachable!()
                                            });
                                        } else {
                                            return Err(Error::new_spanned(
                                                path,
                                                "This gsflag has already had it's inline-ness defined and it cannot be defined again."
                                            ));
                                        }
                                    }
                                    Meta::NameValue(mnv) if mnv.path.is_ident("vis") => {
                                        if vis.is_none() {
                                            vis = match mnv.value {
                                                Expr::Lit(ExprLit {
                                                    attrs: _attrs,
                                                    lit: Lit::Str(str),
                                                }) => Some(str.value().parse()?),
                                                _ => {
                                                    return Err(Error::new_spanned(
                                                        mnv.value,
                                                        "Valid gsflag setting is `vis = \"pub(crate)\"`",
                                                    ));
                                                }
                                            };
                                        } else {
                                            return Err(Error::new_spanned(
                                                mnv,
                                                "This gsflag has already had its visibility defined and cannot have it be defined again."
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(Error::new_spanned(
                                            setting,
                                            "Invalid usage, see `README.md`",
                                        ));
                                    }
                                }
                            }
                        }

                        let inline = inline.unwrap_or(quote! {});

                        let vis = vis.unwrap_or(quote! { pub });

                        let (name, sig, body) = match kind {
                            GSKind::GetrCopy | GSKind::GetrRef => {
                                let name = rename.unwrap_or(format_ident!("get_{}", field_name));

                                let amp = if kind == GSKind::GetrRef
                                    && !matches!(field_type, syn::Type::Reference(_))
                                {
                                    quote! { & }
                                } else {
                                    quote! {}
                                };

                                let sig = quote! { (&self) -> #amp #field_type };

                                let body = quote! { #amp self.#field_name };

                                (name, sig, body)
                            }
                            GSKind::Setr => {
                                let name = rename.unwrap_or(format_ident!("set_{field_name}"));
                                let new_val_name = format_ident!("new_{field_name}");

                                let sig = quote! { (&mut self, #new_val_name: #field_type) };

                                let body = quote! { self.#field_name = #new_val_name; };

                                (name, sig, body)
                            }
                        };

                        let func = quote! {
                            #inline
                            #vis fn #name #sig {
                                #body
                            }
                        };

                        implblk.push(func);
                    }
                }
                _ if attr.path().is_ident("gsflags") => {
                    return Err(Error::new_spanned(
                        attr,
                        "gsflags usage: `#[gsflags(get(rename = \"name\"), set, ...)]`",
                    ));
                }
                _ => {} // Not what we're looking for
            }
        }

        remove_attr.map(|i| field.attrs.remove(i));
    }

    let implstmt = if !implblk.is_empty() {
        quote! {
            impl #struct_name {
                #(#implblk)*
            }
        }
    } else {
        quote! {}
    };

    let out = quote! {
        #input

        #implstmt
    };

    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum GSKind {
    GetrRef,
    GetrCopy,
    Setr,
}
