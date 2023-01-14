use proc_macro::TokenStream;
use quote::quote;
use std::convert::Into;
use syn::{parse_macro_input, DeriveInput, Error, Expr};

pub fn derive_meta(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        return Error::new_spanned(
            struct_name,
            format!("#[derive(Module)] supports only struct declarations"),
        )
        .to_compile_error()
        .into();
    };

    let struct_name_string = quote! { #struct_name}.to_string(); //.to_lowercase();
    let path = syn::parse_str::<Expr>(&struct_name_string)
        .expect("Unable to parse strut name as expression");

    meta_impl(path, fields).into()
}

fn meta_impl(meta_struct: Expr, fields: &syn::FieldsNamed) -> TokenStream {
    let mut field_names = Vec::new();
    let mut get_field_names = Vec::new();
    let mut set_field_names = Vec::new();
    let mut field_types = Vec::new();
    for field in fields.named.iter() {
        let ident = field.ident.clone().unwrap();
        field_names.push(ident.clone());

        let get_field = format!("get_{}", ident);
        let get_field_name = syn::Ident::new(&get_field, ident.span());
        let set_field = format!("set_{}", ident);
        let set_field_name = syn::Ident::new(&set_field, ident.span());
        get_field_names.push(get_field_name);
        set_field_names.push(set_field_name);
        field_types.push(field.ty.clone());
    }

    (quote! {

        impl #meta_struct {

            #(
                #[inline(always)]
                pub fn #get_field_names(&self) -> #field_types {
                    let unaligned = std::ptr::addr_of!(self.#field_names);
                    unsafe { std::ptr::read_unaligned(unaligned) }
                }

                #[inline(always)]
                pub fn #set_field_names(&mut self, v : #field_types) {
                    let unaligned = std::ptr::addr_of_mut!(self.#field_names);
                    unsafe { std::ptr::write_unaligned(unaligned, v) };
                }

            )*

        }

    })
    .into()
}
