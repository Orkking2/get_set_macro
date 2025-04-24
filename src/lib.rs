use proc_macro::TokenStream;
#[cfg(test)]
use quote::quote;
use syn::parse_macro_input;

mod parser;

#[proc_macro_attribute]
pub fn get_set(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);
    parser::expand_get_set(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[test]
fn readme_debug() {
    let quote = quote! {
        // #[get_set] Removed by compiler
        struct Example {
            #[gsflags(get)]
            name: String,

            #[gsflags(get_copy)]
            age: u32,

            #[gsflags(get(noinline, rename = "city_ref"), set(rename = "set_city" /* same as default */))]
            city: String,
        }
    };

    let input = syn::parse2::<syn::ItemStruct>(quote).unwrap();

    let parsed = parser::expand_get_set(input).unwrap_or_else(syn::Error::into_compile_error);

    println!("parsed: {parsed}");
}
