use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, Error, Fields, Ident, ItemStruct, Meta, Result,
    Token,
};

use crate::enums::Kind;
use crate::props::{FieldProps, FuncProps, OptFuncProps, OptFuncPropsWithKind};

pub fn expand_get_set(
    gs_attrs: Option<Punctuated<Meta, Token![,]>>,
    mut input: ItemStruct,
) -> Result<TokenStream> {
    let mut all_func_props = Vec::new();

    if let Some(gs_attrs) = gs_attrs {
        for gs_flag in gs_attrs {
            all_func_props.push(OptFuncPropsWithKind {
                optfuncprops: extract_opt_func_props(&gs_flag)?,
                kind: gs_flag.try_into()?,
            })
        }
    }

    let struct_ident = &input.ident;
    let fields = if let Fields::Named(fields_named) = &mut input.fields {
        &mut fields_named.named
    } else {
        return Err(Error::new_spanned(input, "Only named fields are supported"));
    };

    let mut field_map: HashMap<Ident, FieldProps> = HashMap::new();

    for field in fields {
        let ref mut field_props =
            field_map
                .entry(field.ident.clone().unwrap())
                .or_insert(FieldProps {
                    ty: field.ty.clone(),
                    all_skip: false,
                    props: HashSet::new(),
                });

        let mut remove_attrs = vec![];

        for (i, attr) in field.attrs.iter_mut().enumerate() {
            match attr.meta {
                // gsflags(get, set, get_copy(rename = "draft", inline_never, vis = "pub(crate)"))
                syn::Meta::List(ref gs_flags) if gs_flags.path.is_ident("gsflags") => {
                    let field_ident = field.ident.clone().unwrap();

                    remove_attrs.push(i);

                    // [get, set, get_copy(rename = "draft", inline_never, vis = "pub(crate)")]
                    for gs_flag in
                        gs_flags.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                    {
                        if gs_flag.path().is_ident("skip") {
                            field_props.all_skip = true;
                            continue;
                        }

                        field_props.props.insert(
                            extract_opt_func_props(&gs_flag)?
                                .build(gs_flag.try_into()?, &field_ident),
                        );
                    }

                    if !field_props.all_skip {
                        field_props.props.extend(
                            all_func_props
                                .iter()
                                .map(|ofpwk| ofpwk.clone().build(&field_ident)),
                        );
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

        remove_attrs.into_iter().for_each(|i| {
            field.attrs.remove(i);
        });
    }

    let impl_contents = field_map.into_iter().fold(
        quote! {},
        |acc,
         (
            field_ident,
            FieldProps {
                ty,
                all_skip: _,
                props,
            },
        )| {
            props.into_iter().fold(
                acc,
                |acc,
                 FuncProps {
                     kind,
                     name,
                     inline,
                     vis,
                 }| {
                    let (sig, body) = match kind {
                        Kind::Setr => {
                            let new_val_name = format_ident!("new_{field_ident}");

                            let sig = quote! { (&mut self, #new_val_name: #ty) };
                            let body = quote! { self.#field_ident = #new_val_name; };

                            (sig, body)
                        }
                        Kind::GetrRef | Kind::GetrCopy => {
                            let amp = if kind == Kind::GetrRef
                                && !matches!(ty, syn::Type::Reference(_))
                            {
                                quote! { & }
                            } else {
                                quote! {}
                            };

                            let sig = quote! { (&self) -> #amp #ty };
                            let body = quote! { #amp self.#field_ident };

                            (sig, body)
                        }
                    };

                    quote! {
                        #acc
                        #inline
                        #vis fn #name #sig {
                            #body
                        }
                    }
                },
            )
        },
    );

    Ok(quote! {
        #input

        impl #struct_ident {
            #impl_contents
        }
    })
}

fn extract_opt_func_props(gs_flag: &Meta) -> Result<OptFuncProps> {
    let mut opt_func_props = OptFuncProps::new();

    if let Meta::List(gs_flag_settings) = gs_flag {
        // [rename = "draft", inline_never, vis = "pub(crate)"]
        for setting in
            gs_flag_settings.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
        {
            opt_func_props = opt_func_props.or(setting.try_into()?)
        }
    }

    Ok(opt_func_props)
}
