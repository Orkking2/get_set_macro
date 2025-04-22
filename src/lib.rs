use proc_macro::TokenStream;
use syn::parse_macro_input;

mod parser;

#[proc_macro_attribute]
pub fn get_set(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);
    parser::expand_get_set(input).into()
}