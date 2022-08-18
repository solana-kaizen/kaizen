#[allow(non_snake_case)]

extern crate proc_macro;

mod container;
mod program;
mod interface;
mod client;

use proc_macro::TokenStream;

#[proc_macro]
pub fn declare_program(input: TokenStream) -> TokenStream {
    program::declare_program(input)
}

#[proc_macro]
pub fn declare_handlers(input: TokenStream) -> TokenStream {
    interface::declare_handlers(input)
}

#[proc_macro]
pub fn declare_client(input: TokenStream) -> TokenStream {
    client::declare_client(input)
}

#[proc_macro_attribute]
pub fn container(attr: TokenStream, item: TokenStream) -> TokenStream {
    container::macro_handler(attr, item)
}

