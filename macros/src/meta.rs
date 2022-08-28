use std::convert::Into;
use proc_macro::TokenStream;
// use proc_macro2::Span;
use quote::quote;
// use quote::{quote, ToTokens};
use syn::{
    // Ident, ExprArray,
    // Result, 
    parse_macro_input,
    // punctuated::Punctuated, 
    Expr, 
    // Token, 
    // parse::{Parse, ParseStream}, 
    Error,
    DeriveInput,
};
// use convert_case::{Case, Casing};



pub fn derive_meta(input: TokenStream) -> TokenStream {

    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    // let struct_params = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        return Error::new_spanned(
            struct_name,
            format!("#[derive(Module)] supports only struct declarations")
        )
        .to_compile_error()
        .into();
    };

    let struct_name_string = quote!{ #struct_name}.to_string();//.to_lowercase();
    let path = syn::parse_str::<Expr>(&struct_name_string).expect("Unable to parse strut name as expression");
    
    meta_impl(
        path,
        // &struct_name_string,
        fields,
    ).into()

}

fn meta_impl(
    meta_struct : Expr,
    // module_name : &str,
    fields : &syn::FieldsNamed,
) -> TokenStream {


    let mut field_names = Vec::new();
    let mut get_field_names = Vec::new();
    let mut set_field_names = Vec::new();
    let mut field_types = Vec::new();
    for field in fields.named.iter() {
        // let field_name = field.ident.clone().unwrap().to_token_stream();//.to_string();//.to_string();
        //  let field_name = stringify!(#field_name);

        // field_names.push(quote!{#field_name}); // field.ident.clone().unwrap().to_string());
        let ident = field.ident.clone().unwrap();
        field_names.push(ident.clone());

        let get_field = format!("get_{}", ident);
        let get_field_name = syn::Ident::new(&get_field, ident.span());
        let set_field = format!("set_{}", ident);
        let set_field_name = syn::Ident::new(&set_field, ident.span());
        // let set_field_name = syn::Ident::new(&field, field.ident.span());

        // get_field_names.push(format!("{}",quote!{#field_name}.to_string())); // field.ident.clone().unwrap().to_string());
        get_field_names.push(get_field_name); // field.ident.clone().unwrap().to_string());
        set_field_names.push(set_field_name); // field.ident.clone().unwrap().to_string());
        // s /et_field_names.push(format!("set_{}",quote!{#field_name}.to_string())); // field.ident.clone().unwrap().to_string());
        // field_names.push(quote!{#field_name}.to_string()); // field.ident.clone().unwrap().to_string());
        // field_names.push(field.ident.clone().unwrap().to_string());
        // get_field_names.push(field.ident.clone().unwrap().to_string());
        // set_field_names.push(format!("set_{}",field.ident.clone().unwrap().to_string()));
        field_types.push(field.ty.clone());
    }

    // let module_name = module_name//.to_lowercase();
    //     .from_case(Case::Camel)
    //     .to_case(Case::Snake);


    (quote!{

        // impl workflow_ux::module::ModuleInterface for #module_struct {
        //     fn type_id(self : Arc<Self>) -> Option<std::any::TypeId> { Some(std::any::TypeId::of::<#module_struct>()) }
        // }

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

    }).into()
}

// pub fn identifier(input: TokenStream) -> TokenStream {
//     let string = input.to_string();
//     (quote!{ #string }).into()
// }
