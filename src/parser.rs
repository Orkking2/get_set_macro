use std::collections::HashSet;

use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, Expr, ExprLit, Fields, ItemStruct, Lit, Meta, Token};

pub fn expand_get_set(mut input: ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = input.ident.clone();
    let fields = if let Fields::Named(fields_named) = &mut input.fields {
        &mut fields_named.named
    } else {
        return Err(syn::Error::new_spanned(input, "Only named fields are supported"));
    };

    let mut implblk = vec![];

    for field in fields {
        let field_name = field.ident.clone().unwrap();
        let field_type = &field.ty;
        let mut found_gsflags = false;

        let mut remove_attr = None;

        for (i, attr) in field.attrs.iter_mut().enumerate() {
            match attr.meta {
                // gsflags(get, set, get_copy(rename = "draft"))
                syn::Meta::List(ref gsflags_ml) if gsflags_ml.path.is_ident("gsflags") => {
                    if found_gsflags {
                        return Err(syn::Error::new_spanned(
                            attr, 
                            "gsflags has already been defined for this field and cannot be defined again."
                        ))
                    } else {
                        found_gsflags = true;
                    }

                    remove_attr = Some(i);

                    // [get, set, get_copy(rename = "draft")]
                    let gsflags: Punctuated<Meta, Token![,]> =
                        gsflags_ml.parse_args_with(Punctuated::parse_terminated)?;

                    let mut seen_kinds: HashSet<GSKind> = HashSet::new();

                    // get, set, get_copy(rename = "draft")
                    for gsflag in gsflags {
                        let kind = if gsflag.path().is_ident("get") {
                            GSKind::GetrRef
                        } else if gsflag.path().is_ident("get_copy") {
                            GSKind::GetrCopy
                        } else if gsflag.path().is_ident("set") {
                            GSKind::Setr
                        } else {
                            return Err(syn::Error::new_spanned(
                                gsflag,
                                "Valid attributes are `get`, `get_copy`, and `set`",
                            ));
                        };

                        if !seen_kinds.insert(kind.clone()) {
                            return Err(syn::Error::new_spanned(
                                gsflag, 
                                format!("{:?} has been seen before for this field! Do not reuse field attributes.", kind)
                            ));
                        }

                        let mut rename = None::<syn::Ident>;

                        // get_copy(rename = "draft")
                        if let Meta::List(gsflag_settings_ml) = gsflag {
                            // [rename = "draft"]
                            let gsflag_settings: Punctuated<Meta, Token![,]> =
                                gsflag_settings_ml
                                    .parse_args_with(Punctuated::parse_terminated)?;

                            // rename = "draft"
                            for setting in gsflag_settings {
                                match setting {
                                    Meta::NameValue(mnv) if mnv.path.is_ident("rename") => {
                                        rename = Some(match mnv.value {
                                            Expr::Lit(ExprLit {
                                                attrs: _attrs,
                                                lit: Lit::Str(str),
                                            }) => syn::Ident::new(str.value().as_str(), str.span()),
                                            _ => {
                                                return Err(syn::Error::new_spanned(
                                                    mnv.value,
                                                    "Valid gsflag setting is `rename = \"name\"`",
                                                ));
                                            }
                                        });
                                    }
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            setting,
                                            "Invalid usage, see `README.md`",
                                        ));
                                    }
                                }
                            }
                        }

                        implblk.push(
                            match kind {
                                GSKind::GetrCopy | GSKind::GetrRef => {
                                    let name = rename.unwrap_or(format_ident!(
                                        "get_{}",
                                        field_name
                                    ));

                                    let amp = if kind == GSKind::GetrRef {
                                        quote! { & }
                                    } else {
                                        quote!{}
                                    };

                                    quote! {
                                        pub fn #name(&self) -> #amp #field_type {
                                            #amp self.#field_name
                                        }
                                    }
                                },
                                GSKind::Setr => {
                                    let name = rename.unwrap_or(format_ident!("set_{field_name}"));
                                    let new_val_name = format_ident!("new_{field_name}");

                                    quote! {
                                        pub fn #name(&mut self, #new_val_name: #field_type) {
                                            self.#field_name = #new_val_name;
                                        }
                                    }
                                }
                            }
                        );
                    }
                }
                _ if attr.path().is_ident("gsflags") => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "gsflags usage: `#[gsflags(get(rename = \"name\"), set, ...)]`",
                    ));
                }
                _ => {} // Not what we're looking for
            }
        }

        remove_attr.map(|i| field.attrs.remove(i));
    }



    let out = quote! {
        #input

        impl #struct_name {
            #(#implblk)*
        }
    };

    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum GSKind {
    GetrRef,
    GetrCopy,
    Setr,
}