//! # get_set_macro
//!
//! A procedural macro to generate customizable getters and setters for your Rust structs.
//!
//! ## Installation
//!
//! Run `cargo add get_set_macro`
//!
//! ## Example Usage
//!
//! ```rust
//! use get_set_macro::get_set;
//!
//! // By default, each field in `Example` will recieve a getter with the #[inline(always)] attribute.
//! // Here the `default` is applied to the `get`, so the global get will be #[inline(always)].
//! #[get_set(default(inline_always), get)]
//! struct Example {
//!     // This field will not recieve the default getter.
//!     #[gsflags(skip)]
//!     skipped: u8,
//!
//!     // Despite not having any `gsflags`, this field will recieve an inline(always) getter named `get_name`.
//!     name: String,
//!     
//!     // Since u32's are trivially copyable, there is no need to pass this value by reference and so instead passes it by value.
//!     // This flag has also inhereted the default `inline_always`, so this `get_copy` will be inlined always.
//!     #[gsflags(get_copy)]
//!     age: u32,
//! }
//! ```
//!
//! ## See Also
//!
//! - [Crate on crates.io](https://crates.io/crates/get_set_macro)
//! - [Source on GitHub](https://github.com/Orkking2/get_set_macro)
//! - [More examples](https://github.com/Orkking2/get_set_proc_macro/tree/main/tests/ui)

use proc_macro::TokenStream;
#[cfg(test)]
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Meta, Token};

mod enums;
mod parser;
mod props;

#[proc_macro_attribute]
pub fn get_set(attr: TokenStream, item: TokenStream) -> TokenStream {
    let gs_attrs =
        syn::parse::Parser::parse(Punctuated::<Meta, Token![,]>::parse_terminated, attr).ok();

    let input = parse_macro_input!(item as syn::ItemStruct);

    parser::expand_get_set(gs_attrs, input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[test]
fn debug() {
    let quote = quote! {
        // #[get_set] Removed by compiler
        struct Example {
            #[gsflags(get)]
            name: String,

            #[gsflags(get_copy)]
            age: u32,

            #[gsflags(get(inline_always, vis = "pub(crate)", rename = "city_ref"), set(rename = "set_city" /* same as default */))]
            city: String,
        }
    };

    let input = syn::parse2::<syn::ItemStruct>(quote).unwrap();

    let parsed = parser::expand_get_set(None, input).unwrap_or_else(syn::Error::into_compile_error);

    println!("parsed: {parsed}");
}
