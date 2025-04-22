use quote::{quote, format_ident};
use syn::{ItemStruct, Fields, Meta, MetaList, MetaNameValue, Lit, Attribute};

pub fn expand_get_set(input: ItemStruct) -> proc_macro2::TokenStream {
    let struct_name = input.ident.clone();
    let fields = if let Fields::Named(fields_named) = input.fields {
        fields_named.named
    } else {
        return syn::Error::new_spanned(input, "Only named fields are supported")
            .to_compile_error();
    };

    let mut getters = vec![];
    let mut setters = vec![];

    for field in fields.iter() {
        let field_name = field.ident.clone().unwrap();
        let field_type = &field.ty;
        let mut getter_kind = GetterKind::None;
        let mut custom_getter_name = None;
        let mut custom_setter_name = None;
        let mut has_setr = false;

        for attr in &field.attrs {
            if attr.path().is_ident("getr") {
                getter_kind = GetterKind::Ref;
                custom_getter_name = parse_custom_name(attr);
            }
            if attr.path().is_ident("get_copy") {
                getter_kind = GetterKind::Copy;
                custom_getter_name = parse_custom_name(attr);
            }
            if attr.path().is_ident("setr") {
                has_setr = true;
                custom_setter_name = parse_custom_name(attr);
            }
        }

        if let GetterKind::Ref | GetterKind::Copy = getter_kind {
            let getter_name = custom_getter_name.unwrap_or_else(|| format_ident!("get_{}", field_name));
            let return_type = match getter_kind {
                GetterKind::Ref => quote! { &#field_type },
                GetterKind::Copy => quote! { #field_type },
                GetterKind::None => unreachable!(),
            };
            let return_expr = match getter_kind {
                GetterKind::Ref => quote! { &self.#field_name },
                GetterKind::Copy => quote! { self.#field_name },
                GetterKind::None => unreachable!(),
            };

            getters.push(quote! {
                pub fn #getter_name(&self) -> #return_type {
                    #return_expr
                }
            });
        }

        if has_setr {
            let setter_name = custom_setter_name.unwrap_or_else(|| format_ident!("set_{}", field_name));
            setters.push(quote! {
                pub fn #setter_name(&mut self, new_val: #field_type) {
                    self.#field_name = new_val;
                }
            });
        }
    }

    quote! {
        #input

        impl #struct_name {
            #(#getters)*
            #(#setters)*
        }
    }
}

enum GetterKind {
    None,
    Ref,
    Copy,
}

fn parse_custom_name(attr: &Attribute) -> Option<proc_macro2::Ident> {
    attr.parse_args_with(|input: syn::parse::ParseStream| {
        let ident: syn::Ident = input.parse()?;
        if ident != "name" {
            return Err(syn::Error::new(ident.span(), "Expected `name`"));
        }

        input.parse::<syn::Token![=]>()?;
        let lit: syn::LitStr = input.parse()?;

        Ok(format_ident!("{}", lit.value()))
    }).ok()
}