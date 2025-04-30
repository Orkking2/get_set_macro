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
//! #[get_set(default(inline_always, vis = "pub"), get)]
//! struct Example {
//!     // This field will not recieve the default getter.
//!     #[gsflags(skip)]
//!     skipped: u8,
//!
//!     // Despite only having the `set` flag, this member will recieve both a getter and a setter,
//!     // each with pub visibility and each being #[inline(always)]
//!     #[gsflags(set)]
//!     name: String,
//!     
//!     // Since u32's are trivially copyable, 
//!     // there is no need to pass this value by reference and so instead we will pass it by value.
//!     // This flag has also inhereted the default `inline_always`, so this `get_copy` will be #[inline(always)].
//!     // If we wanted to override the defaults, adding `default(noinline, vis = "")` anywhere 
//!     // in the gsflags body would change the defaults to having no inline attribute and inherited (private) visibility.
//!     // If we didn't want this to override the default get(_ref), we could change 
//!     // `get_copy` to `get_copy(rename = "get_age_copy")`,
//!     // which would simply create a new method that returned a copy.
//!     #[gsflags(get_copy)]
//!     age: u32,
//! }
//! 
//! // Has functionality
//! fn main() {
//!     let mut example = Example {
//!         skipped: 8,
//!         name: "ExampleName".to_string(),
//!         age: 69,
//!     };
//! 
//!     // The following would error as there is no method named `get_skipped`
//!     // assert_eq!(8, example.get_skipped());
//! 
//!     // The getters and setters of `name`
//!     assert_eq!("ExampleName".to_string(), *example.get_name());
//!     example.set_name("NewName".to_string());
//!     assert_eq!("NewName".to_string(), *example.get_name());
//! 
//!     assert_eq!(69, example.get_age());
//! }
//! ```
//!
//! ## See Also
//!
//! - [Crate on crates.io](https://crates.io/crates/get_set_macro)
//! - [Source on GitHub](https://github.com/Orkking2/get_set_macro)
//! - [More examples](https://github.com/Orkking2/get_set_proc_macro/tree/main/tests/ui)

use proc_macro::TokenStream;
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
