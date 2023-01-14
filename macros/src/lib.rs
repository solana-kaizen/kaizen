#[allow(non_snake_case)]
extern crate proc_macro;

mod client;
mod container;
mod interface;
mod meta;
mod program;

use proc_macro::TokenStream;

#[proc_macro]
pub fn declare_program(input: TokenStream) -> TokenStream {
    program::declare_program(input)
}

#[proc_macro]
pub fn declare_handlers(input: TokenStream) -> TokenStream {
    interface::declare_interface(input)
}

#[proc_macro]
pub fn declare_interface(input: TokenStream) -> TokenStream {
    interface::declare_interface(input)
}

#[proc_macro]
pub fn declare_client(input: TokenStream) -> TokenStream {
    client::declare_client(input)
}

#[proc_macro_attribute]
pub fn container(attr: TokenStream, item: TokenStream) -> TokenStream {
    container::container_attribute_handler(attr, item)
}

#[proc_macro_derive(Meta)]
pub fn meta(input: TokenStream) -> TokenStream {
    meta::derive_meta(input)
}
