
use std::convert::Into;
use proc_macro::{TokenStream};
use proc_macro2::{Span, Ident, Group};
 use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use workflow_macro_tools::attributes::*;

// use syn::parse::ParseBuffer;
use syn::{
    Error,
    // LitStr,
    Type,
    // Path,
    TypePath,
    Attribute,
    LitBool,
    Visibility,
    // LitInt,
    // ExprType,
    GenericArgument
};
use syn::{
    // Result,
    parse_macro_input,
    // ExprArray,
    PathArguments,
    // ExprLit,
    // spanned::Spanned,
    DeriveInput,
    punctuated::Punctuated,
    Expr,
    Token,
    parse::{Parse, ParseStream},
    // ExprPath,
    // PathSegment,
    // Lit,
    // FieldsNamed,
    // FieldsUnnamed,
};
// use darling::{FromDeriveInput, FromField};
use std::collections::HashMap;


#[derive(Debug)]
struct SegmentArgs {
//    reserve : Option<Expr>
    map : HashMap<Ident,Option<Value>>,
    // args : Vec<(Ident,Option<Group>)>,
}

fn get_segment_attrs(attr: &Attribute) -> syn::Result<SegmentArgs> {
    let meta: SegmentArgs = attr.parse_args().unwrap();
    Ok(meta)
}

impl SegmentArgs {
    pub fn get(&self, name: &str) -> Option<&Option<Value>> {
        let ident = Ident::new(name, Span::call_site());
        self.map.get(&ident)
    }
}

/*
#[segment(reserve(1024))]
#[segment(reserve = 1024)]
#[segment(reserve(size_of<T>*3))]
#[segment(reserve(MappedArray::size_with_records(3)))]
#[segment(resize, reserve(MappedArray::size_with_records(3)))]
#[segment(resize = true, reserve(MappedArray::size_with_records(3)))]
*/

const SEGMENT_ATTRIBUTES: &[&str] = &["fixed","reserve","flex"];

impl Parse for SegmentArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut map: HashMap<Ident,Option<Value>> = HashMap::new();
        while !input.is_empty() {
            let token : Item =  input.parse()?;
            match token {
                Item::Identifier(ident) => {
                    if input.peek(Token![,]) {
                        let _ : Token![,] = input.parse()?;
                        map.insert(ident, Some(Value::AssignmentValue(AssignmentValue::Boolean(LitBool::new(true,Span::call_site())))));
                    } else if input.peek(Token![=]) {
                        let _ : Token![=] = input.parse()?;
                        let rvalue : AssignmentValue = input.parse()?;
                        map.insert(ident, Some(Value::AssignmentValue(rvalue)));
                    } else {
                        let group : Group = input.parse()?;
                        map.insert(ident, Some(Value::EvaluationValue(EvaluationValue::Group(group))));
                    }

                    if input.peek(Token![,]) {
                        let _ : Token![,] = input.parse()?;
                    }

                },
                Item::Literal(lit) => {
                    let reserve = Ident::new("reserve", Span::call_site());
                    if map.get(&reserve).is_none() {
                        map.insert(reserve, Some(Value::EvaluationValue(EvaluationValue::Integer(lit))));
                    }
                },
                _ => {
                    return Err(Error::new_spanned(
                        input.parse::<Expr>()?,
                        format!("unsipported attributes")
                    ));
    
                }
            }
        }

        for (ident,_) in map.iter() {
            let name = ident.to_string();
            if !SEGMENT_ATTRIBUTES.contains(&name.as_str()) {
                return Err(Error::new_spanned(
                    ident,
                    format!("unsupported segment attribute: {}, supported attributes are {}", name, SEGMENT_ATTRIBUTES.join(", "))
                ));
                // .to_compile_error()
            //    .into()
            }
        }

        Ok(Self {
            map
        })
    }
}


// #[derive(Debug)]
// enum SegmentTypes {
//     Raw,
//     Struct,
//     Array,
//     Serialized,
//     Collection,
// }

// Span::call_site()

#[derive(Debug)]
struct Segment {
    // opts : Opts,
    args : Option<SegmentArgs>,
    flex : bool,
    field_name : syn::Ident,
    name : String,
    // literal_key_str : LitStr,
    type_name : Type,
    type_ident : Option<TypePath>,
    visibility: Visibility,

    // type_name_str : String,
    type_name_args : Option<String>,
}

impl Segment {
    pub fn is_meta(&self) -> bool { self.name == "meta"  || self.name == "_meta" }
    pub fn is_store(&self) -> bool { 
        match self.type_ident.as_ref() {
            None => false,
            Some(type_name_ident) => {
                if type_name_ident.path.is_ident("SegmentStore") {
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
struct ContainerAttributes {
    container_type : Expr,
    index_size_type : TokenStream2,
    // target_primitive_path : ExprPath,
    // primitive_dispatch_method : ExprPath,
    // client_struct_decl : TokenStream,
    // client_lifetimes : Option<String>,
}

impl Parse for ContainerAttributes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {

        let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(input).unwrap();
        if parsed.len() < 1 || parsed.len() > 2 {
            return Err(Error::new_spanned(
                parsed[2].clone(),
                format!("usage: #[container(<container type id>, <index size: u16 or u32>)]")
            ))

        }

        let mut iter = parsed.iter();
        let container_type = iter.next().clone().unwrap().clone();
        // let container_type_ref_expr = iter.next().clone().unwrap().clone();
        // let container_type_ref = match cype_ref_expr {
        //     Expr::Lit(lit) => lit,
        //     _ => panic!("the first argument should be the program_name)")
        // };


        let index_size_type : TokenStream2  = if parsed.len() > 1 {
            let index_size_type = iter.next().clone().unwrap().clone();
            let index_size_type_str = index_size_type.to_token_stream().to_string();
            match index_size_type_str.as_str() {
                "u32" | "u16" => { index_size_type.to_token_stream().into() }, 

                _ => {
                    return Err(Error::new_spanned(
                        index_size_type,
                        format!("the second argument should be a type u16 or u32)")
                    ));
                }
            }
        } else {
            (quote!{ u16 }).into()
        };

        // println!("================================================================================ {:#?}", index_size_type_expr);
        // let index_size_type = match index_size_type_expr {
        //     Expr::Type(expr_type) => expr_type,
        //     _ => panic!("the second argument should be a type u16 or u32)")
        // };

        Ok(ContainerAttributes {
            container_type, index_size_type : index_size_type.into()
        })

    }
}

// #[proc_macro_derive(Describe, attributes(segment))]
pub fn macro_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$ {:#?} $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$",attr);

    // let vvv : ExprArray = syn::parse(_attr).unwrap();

    // let attr_parse_buffer : ParseBuffer = _attr.into();

    // let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(_attr.into()).unwrap();
    // if parsed.len() != 2 {
    //     panic!("usage: #[container(<container type id>, <index size: u16 or u32>)]");
    // }
    
    // vvv.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated);
    let cattr = parse_macro_input!(attr as ContainerAttributes);
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$ {:#?} $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$",cattr);



    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    // println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    let struct_decl_src = item.clone();
    let ast = parse_macro_input!(struct_decl_src as DeriveInput);
    let struct_name = &ast.ident;
    let struct_params = &ast.generics;

    // let module_name_str = struct_name.to_string().to_lowercase();


    let mut generics_only = ast.generics.clone();
    generics_only.params = {
        let mut params : Punctuated<syn::GenericParam, Token![,]> = Punctuated::new();
        for param in generics_only.params.iter() {
            match param {
                syn::GenericParam::Type(_) => {
                    params.push(param.clone());
                },
                _ => {}
            }
        }
        params
    };
    let has_generics = generics_only.params.len() > 0;
    // generics_only.params = params;
    let where_clause = match generics_only.where_clause.clone() {
        Some(where_clause) => quote!{ #where_clause },
        None => quote!{}
    };

    // let vec : Vec<(String,String)> = std::env::vars().collect();
    // store_container_type(&format!("{:#?}",vec));
    // store_container_type(&format!("{:#?}",std::env::current_dir()));
 

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        return Error::new_spanned(
            ast,
            format!("#[container] macro only supports structs")
        )
        .to_compile_error()
        .into();
    };

    let mut flex_segments = 0;
    let mut segments : Vec<Segment> = Vec::new();
    for field in fields.named.iter() {
        let field_name: syn::Ident = field.ident.as_ref().unwrap().clone();
        let name: String = field_name.to_string();

        let mut attrs: Vec<_> =
            field.attrs.iter().filter(|attr| attr.path.is_ident("segment")).collect();
        if attrs.len() > 1 {
            return Error::new_spanned(
                attrs[1].clone(),
                format!("#[container]: more than one #[segment()] attributes while processing {}", name)
            )
            .to_compile_error()
            .into();
    
        }
        let args = if attrs.len() > 0 {
            let attr = attrs.remove(0);
            Some(get_segment_attrs(attr).unwrap())
        } else {
            None
        };

        // args.get()
        // let is_flex = args.get("flex").is_
        let flex = if let Some(args) = &args {
            args.get("flex").is_some()
        } else { false };
        if flex { flex_segments += 1; }

        if flex_segments > 1 {
            return Error::new_spanned(
                field_name.clone(),
                format!("multiple flex attributes are not supported")
            )
            .to_compile_error()
            .into();                
        }    

        let type_name = field.ty.clone();
        let visibility = field.vis.clone();
        let type_name_for_ident = type_name.clone();

        let (type_ident,type_name_args) = match type_name_for_ident {
            Type::Path(mut type_path) => {
                let target = type_path.path.segments.last_mut().unwrap();
                // println!("E");
                let type_name_args = match &target.arguments {
                    PathArguments::AngleBracketed(params) => {
                        // println!("F");

                        // params.args.iter().filter(|v|)
                        // GenericArgument::Type(arg_type) => {
                        let mut types : Vec<String> = Vec::new();
                        for arg in params.args.iter() {
                            match arg {
                                GenericArgument::Type(arg_type) => {
                                    types.push(arg_type.to_token_stream().to_string());
                                },
                                _ => {}
                            }
                        }

                        // let mut ts = proc_macro2::TokenStream::new();
                        // params.args.clone().to_tokens(&mut ts);
                        // let lifetimes = ts.to_string();
                        target.arguments = PathArguments::None;
                        // println!("G");
                        // Some(lifetimes)
                        Some(types.join(","))
                    },
                    _ => None
                };

                (Some(type_path), type_name_args)
            },
            _ => {
                (None,None)
            }
        };
// println!("!!!!!!!!!!!!!!!!!!!!!!  ========= > {:#?}", field_name);
        let seg = Segment {
            args,
            flex,
            visibility,
            field_name,
            name,
            // literal_key_str,
            type_name,
            // type_name_str,
            type_ident,
            type_name_args,
        };

        segments.push(seg);
    }


    let mut store_field_visibility = quote!{ };
    let mut store_field_name = quote!{ __store__ };
    let mut store_field_type = quote!{ workflow_allocator::container::segment::SegmentStore<'info,'refs> };
    // let mut idx : usize = 0;
    for segment in segments.iter() {
        match segment.type_ident.as_ref() {
            None => continue,
            Some(type_name_ident) => {
                if type_name_ident.path.is_ident("SegmentStore") {
                    let ts2 : TokenStream2 = segment.name.parse().unwrap();
                    store_field_name = quote!{ #ts2 };
                    let type_name = &segment.type_name;
                    store_field_type = quote!{ #type_name };
                    let visibility = &segment.visibility;
                    store_field_visibility = quote!{ #visibility };
                    break;
                }
            }
        }
        // if segment.type_name_ident.is_some() && segment.type_name_ident.unwrap().is_ident("SegmentStore") {
        // // if segment.name == "store" || segment.name == "_store" {
        //     let ts2 : TokenStream2 = segment.name.parse().unwrap();
        //     store_field_name = quote!{ #ts2 };
        //     break;
        // }
    }

    // filter our store field
    let segments = segments.into_iter().filter(|seg| !seg.is_store() ).collect::<Vec<_>>();


    // let seg = Segment {
    //     args : None,
    //     field_name : store_field_name.clone(),
    //     name : store_field_name.to_string(),
    //     // literal_key_str,
    //     type_name : store_type_name,
    //     // type_name_str,
    //     type_name_ident,
    //     // type_name_args,
    // };

    // segments.push(seg);

    // let store_field_name_ts2: TokenStream2 = match store_field_name_opt {
    //     Some(field_name) => {
    //         let ts2 : TokenStream2 = field_name.parse().unwrap();
    //         quote!{ #ts2 }
    //     },
    //     None => quote!{}
    // };


    let mut inits = Vec::new();
    let mut loads = Vec::new();

    let mut meta_type_path : Option<TypePath> = None;

    let mut flex : Option<usize> = None;
    let mut idx : usize = 0;
    for segment in segments.iter() {

        // println!("\n\n\n********************************");
        // println!("************* NAME {} SEGMENT TYPENAME: {:#?}", segment.name, segment.type_name);
        // println!("********************************\n\n\n");

        let field_name = &segment.field_name;
        // let field_type = segment.ty;

        if segment.name == "store" || segment.name == "_store" {
            // store_field_name = Some(segment.name.clone());
            continue;
        }

        if segment.name == "meta" || segment.name == "_meta" {
            let bind_meta = match &segment.type_name {
                Type::Reference(reference) => {
                    if reference.mutability.is_none() { 
                        return Error::new_spanned(
                            reference.clone(),
                            format!("meta must be &'info mut reference") 
                        )
                        .to_compile_error()
                        .into();                
                    }
                    if let Type::Path(type_path) = &*reference.elem {
                        meta_type_path = Some(type_path.clone());
                    }
                    let type_name = segment.type_name.clone();
                    let meta_name = Ident::new(&segment.name, Span::call_site());
                    quote!{
                        let #meta_name : #type_name = {
                            let mut data = #store_field_name.account.data.borrow_mut();
                            unsafe { std::mem::transmute(&mut data[container_meta_offset]) }
                        };
                    }
                },
                Type::Path(type_path) => {
                    let ref_cell = type_path.path.segments.first().unwrap();
                    if ref_cell.ident == "RefCell" {
                        match &ref_cell.arguments {
                            PathArguments::AngleBracketed(angle_bracketed) => {
                                match &angle_bracketed.args.first().unwrap() {
                                    GenericArgument::Type(arg_type) => {
                                        match arg_type {
                                            Type::Reference(reference) => {
                                                if reference.mutability.is_none() { 
                                                    return Error::new_spanned(
                                                        reference.clone(),
                                                        format!("meta must be &'info mut reference") 
                                                    )
                                                    .to_compile_error()
                                                    .into();
                                                }
                                                if let Type::Path(type_path) = &*reference.elem {
                                                    meta_type_path = Some(type_path.clone());
                                                }
                                                let type_name = segment.type_name.clone();
                                                let meta_name = Ident::new(&segment.name, Span::call_site());

                                                quote!{
                                                    let #meta_name : #type_name = {
                                                        let mut data = #store_field_name.account.data.borrow_mut();
                                                        let meta = unsafe { std::mem::transmute(&mut data[container_meta_offset]) };  // @meta
                                                        // let meta = unsafe { std::mem::transmute(&mut data[std::mem::size_of::<workflow_allocator::container::ContainerHeader>()]) };
                                                        RefCell::new(meta)
                                                    };
                                                }
                                            },
                                            _ => { 
                                                return Error::new_spanned(
                                                    ref_cell.clone(),
                                                    format!("RefCell generic arguments: expecting type or type reference") 
                                                )
                                                .to_compile_error()
                                                .into();
                                            }
                                        }
                                    },
                                    _ => {
                                        return Error::new_spanned(
                                            ref_cell.clone(),
                                            format!("RefCell generic arguments: expecting type or type reference")
                                        )
                                        .to_compile_error()
                                        .into();
                                    }
                                }
                            },
                            _ => {
                                return Error::new_spanned(
                                    ref_cell.clone(),
                                    format!("expecting AngleBracketed arguments for RefCell")
                                )
                                .to_compile_error()
                                .into();
                            }
                        }
                    }
                    else {
                        quote!{}
                    }
                },
                _ => {
                    quote!{}
                }
            };

            inits.push(quote!{
                #bind_meta
            });
            loads.push(quote!{
                #bind_meta
            });

            continue;
        } // if meta

        idx = idx+1;
        let type_name = &segment.type_name;
        let type_ident = &segment.type_ident;

        if segment.flex {
            flex = Some(idx);
        }

        inits.push(quote!{
            let segment = #store_field_name.try_get_segment_at(#idx)?;
            let #field_name : #type_name  = #type_ident::try_create_from_segment(segment)?;

            // ^  ket x : T<k> = T::new();
        });
        loads.push(quote!{
            let segment = #store_field_name.try_get_segment_at(#idx)?;
            let #field_name : #type_name  = #type_ident::try_load_from_segment(segment)?;
            // let #field_name = #type_ident::try_load_from_segment(segment)?;
        });
    }

    let field_ident_vec : Vec<Ident> = segments.iter().map(|f| { f.field_name.clone() }).collect();
    let field_idents = quote!{
        #store_field_name,
        #(#field_ident_vec),*
    };
    let field_type_vec : Vec<Type> = segments.iter().map(|f| { f.type_name.clone() }).collect();
    let field_visibility_vec : Vec<Visibility> = segments.iter().map(|f| { f.visibility.clone() }).collect();

    let inits_create = quote!{
        #struct_name {
            #field_idents
        }
    };

    let loads_create = quote!{
        #struct_name {
            #field_idents
        }
    };

    let mut inits_ts2 = TokenStream2::new();
    inits_ts2.extend(inits);
    let mut loads_ts2 = TokenStream2::new();
    loads_ts2.extend(loads);

    let struct_redef = quote!{
        // #[derive(Debug)]
        pub struct #struct_name #struct_params #where_clause {
            #store_field_visibility #store_field_name : #store_field_type,
            #( #field_visibility_vec #field_ident_vec : #field_type_vec ),*
        }
    };

    // gather sizes for fn sizes() -> &'static [usize]
    // let flex_segments: usize = 0;
    let mut sizes_ts = Vec::new();
    for segment in segments.iter() {
        if segment.type_ident.is_none() || segment.is_meta() || segment.is_store() {
            continue;
        }

        // if let Some(args) = &segment.args {
        //     let reserve = Ident::new("reserve", Span::call_site());
        //     match args.map.get(&reserve) {

        // }


        let ts_ = if let Some(args) = &segment.args {
            let reserve = Ident::new("reserve", Span::call_site());
            match args.map.get(&reserve) {
                Some(reserve) => {
                    match reserve {
                        Some(reserve) => {
                            let ts = reserve.to_token_stream();
                            // sizes_ts.push(ts);
                            //println!("#############reserve: {:?}", ts.to_string());
                            Some(ts.clone())
                        },
                        None => {
                            return Error::new_spanned(
                                // reserve.clone(),
                                // segment.args.clone(),
                                segment.field_name.clone(),
                                format!("#[segment()]: reserve attribute for segment '{}' must contain a value",segment.name)
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                },
                None => {
                    None
                }
            }
        } else { None };
        
        let ts = match ts_ {
            Some(ts) => {
                ts.clone()
            },
            None => {
                // let ty = segment.type_name.clone();//.unwrap();
                // let ts = quote!{ #ty::data_len_min()+1  };

                if segment.type_name_args.is_some() {
                    let ident = segment.type_ident.clone().unwrap();
                    let args = segment.type_name_args.clone().unwrap();
                    let args: proc_macro2::TokenStream = args.parse().unwrap();

                    let ts = quote!{ #ident::<#args>::data_len_min()  };
                    ts

                } else {
                    let ident = segment.type_name.clone();//.unwrap();
                    let ts = quote!{ #ident::data_len_min()  };
                    ts
                }
            }
        };

        sizes_ts.push(ts.clone());
    }

    let sizes_ts_len = sizes_ts.len();

    let init_offset = if let Some(type_path) = &meta_type_path {
        quote!{
            let container_meta_offset = std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
            let segment_store_offset = std::mem::size_of::<workflow_allocator::container::ContainerHeader>() + std::mem::size_of::<#type_path>();
        }
    } else {
        //            let meta_offset = std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
        quote!{
            let segment_store_offset = std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
        }
    };


    let container_type = cattr.container_type;
    let index_unit_size = cattr.index_size_type;


    // let meta_type = if let Some(type_path) = &meta_type_path {
    //     type_path.to_token_stream()
    // } else {
    //     quote!{}
    // };

    let struct_name_str = struct_name.to_string();
    // let struct_declaration = Ident::new(&struct_name_str.clone().to_uppercase(), struct_name.span());
    let module_declaration = Ident::new(&format!("init_{}",struct_name_str).to_lowercase(), struct_name.span());
    let container_declaration_register_ = Ident::new(&format!("container_declaration_register_{}",struct_name_str).to_lowercase(), struct_name.span());


    let try_create_with_meta = if let Some(_type_path) = &meta_type_path {
        quote!{}

        // quote!{

        //     pub fn try_create_with_meta(
        //         account : &'refs solana_program::account_info::AccountInfo<'info>, 
        //         meta_init : &#meta_type) 
        //     -> workflow_allocator::result::Result<Self> {
            
        //         #init_offset
        //         let container_type : u32 = #container_type as u32;
        //         let layout = Self::layout();
        //         let #store_field_name = workflow_allocator::container::segment::SegmentStore::try_create(
        //             &account, segment_store_offset, &layout,
        //         // ).unwrap();
        //         )?;

        //         #inits_ts2

        //         {
        //             let data = account.data.borrow_mut();
        //             let header = unsafe { std::mem::transmute::<_,&mut workflow_allocator::container::ContainerHeader>(
        //                 data.as_ptr()
        //             ) };
        //             header.container_type = container_type;
        //         }

        //         {
        //             let mut meta_dest = meta.borrow_mut();
        //             *(*meta_dest) = *meta_init;
        //         }

        //         Ok(#inits_create)
        //     }


        // }
    } else {
        quote!{}
    };

    let struct_path_with_generics = if has_generics {
        quote!{ #struct_name::#generics_only }
    } else {
        quote!{ #struct_name }
    };

    let init_layout = match flex {
        None => { 
            quote! { 
                let layout = Self::layout();
            }
        },
        Some(flex) => {
            let flex_value_for_layout = flex - 1;
            quote! {
                let layout = {
                    let account_data_len = account.data_len();
                    let mut layout = Self::layout();
                    let layout_len: usize = layout.data_len();
                    if account_data_len > layout_len {
                        let segment_len = layout.get_segment_size(#flex_value_for_layout);
                        let available_for_flex = account_data_len - layout_len - segment_len;
                        layout.set_segment_size(#flex_value_for_layout,available_for_flex);
                    }
                    layout
                };

            }
        }
    };

    // let try_create_with_meta = match 

    let init = quote!{

        #struct_redef

        impl #struct_params #struct_name #struct_params #where_clause {

            // pub fn try_allocate<'pid,'instr>(
            //     ctx: &std::rc::Rc<workflow_allocator::context::Context<'info,'refs,'pid,'instr>>,
            //     allocation_args : &workflow_allocator::context::AccountAllocationArgs<'info,'refs>
            // ) -> workflow_allocator::result::Result<Self> {

            //     let data_len = Self::initial_data_len();
            //     let account_info = ctx.create_pda(data_len,allocation_args)?;
            //     Ok(Self::try_create(account_info)?)
            // }
        
            pub fn try_allocate_default<'pid,'instr>(
                ctx: &std::rc::Rc<workflow_allocator::context::Context<'info,'refs,'pid,'instr>>,
                allocation_args : &workflow_allocator::context::AccountAllocationArgs<'info,'refs>,
            ) -> workflow_allocator::result::Result<Self> {
                
                Ok(Self::try_allocate(ctx,allocation_args,0)?)
            }
        
            pub fn try_allocate<'pid,'instr>(
                ctx: &std::rc::Rc<workflow_allocator::context::Context<'info,'refs,'pid,'instr>>,
                allocation_args : &workflow_allocator::context::AccountAllocationArgs<'info,'refs>,
                reserve_data_len : usize
            ) -> workflow_allocator::result::Result<Self> {

                let data_len = Self::initial_data_len() + reserve_data_len;
                let account_info = ctx.create_pda(data_len,allocation_args)?;
                Ok(Self::try_create(account_info)?)
            }
        

        // fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self> {
            pub fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<#struct_name #struct_params> {
        
                #init_offset
                let container_type : u32 = #container_type as u32;
                // let layout = Self::layout();
                #init_layout
                // println!("try_create##### offset:{},  11111:{:?}", offset, account);
                let #store_field_name = workflow_allocator::container::segment::SegmentStore::try_create(
                    &account, segment_store_offset, &layout,
                // ).unwrap();
                )?;
                // println!("try_create#####22222:{:?}", #store_field_name);
                #inits_ts2

                {
                    let data = account.data.borrow_mut();
                    let header = unsafe { std::mem::transmute::<_,&mut workflow_allocator::container::ContainerHeader>(
                        data.as_ptr()
                    ) };
                    header.container_type = container_type;
                }

                Ok(#inits_create)
            }

            pub fn try_create_with_layout(account : &'refs solana_program::account_info::AccountInfo<'info>, layout : &workflow_allocator::container::segment::Layout<#index_unit_size>) -> workflow_allocator::result::Result<#struct_name #struct_params> {
        
                #init_offset
                let container_type : u32 = #container_type as u32;
                // let layout = Self::layout();
                // println!("try_create##### offset:{},  11111:{:?}", offset, account);
                let #store_field_name = workflow_allocator::container::segment::SegmentStore::try_create(
                    &account, segment_store_offset, &layout,
                // ).unwrap();
                )?;
                // println!("try_create#####22222:{:?}", #store_field_name);
                #inits_ts2

                {
                    let data = account.data.borrow_mut();
                    let header = unsafe { std::mem::transmute::<_,&mut workflow_allocator::container::ContainerHeader>(
                        data.as_ptr()
                    ) };
                    header.container_type = container_type;
                }

                Ok(#inits_create)
            }

            // pub fn try_load_account_data<'a : 'info + 'refs> (data : &'a mut workflow_allocator::simulator::AccountData) -> workflow_allocator::result::Result<#struct_name #struct_params> {
            //     let account_info = data.into_account_info();
            //     Self::try_load(&account_info)
            // }

            // pub 
            // fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self> {
            pub fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<#struct_name #struct_params> {

                #init_offset
                let container_type : u32 = #container_type as u32;
                let layout = Self::layout();
                let #store_field_name = workflow_allocator::container::segment::SegmentStore::try_load(
                    &account, segment_store_offset,
                // ).unwrap();
                )?;

                {
                    let data = account.data.borrow_mut();
                    let header = unsafe { std::mem::transmute::<_,&mut workflow_allocator::container::ContainerHeader>(
                        data.as_ptr()
                    )};

                    if header.container_type != container_type {
                        return Err(
                            workflow_allocator::error::Error::new()
                                .with_program_code(workflow_allocator::error::ErrorCode::ContainerTypeMismatch as u32)
                                .with_source(file!(),line!())
                        );
                        // return Err(workflow_allocator::error::ErrorCode::ContainerTypeMismatch.into());
                    }
                }

                #loads_ts2

                Ok(#loads_create)
            }

            #try_create_with_meta


            #[inline]
            pub fn layout() -> workflow_allocator::container::segment::Layout<#index_unit_size> {
                workflow_allocator::container::segment::Layout::<#index_unit_size>::from(&#struct_path_with_generics::segments()) // @aspect
            }

            #[inline]
            pub fn initial_data_len() -> usize {
                #init_offset
                #struct_path_with_generics::layout().data_len() + segment_store_offset // @aspect
            }

            #[inline]
            pub fn sync_rent<'pid,'instr>(
                &self,
                ctx: &std::rc::Rc<workflow_allocator::context::Context<'info,'refs,'pid,'instr>>,
                rent_collector : &workflow_allocator::rent::RentCollector<'info,'refs>,
            ) -> workflow_allocator::result::Result<()> {
                // TODO: @alpha - transfer out excess rent
                ctx.sync_rent(self.account(),rent_collector)?;
                Ok(())
            }

            #[inline]
            pub fn purge<'pid,'instr>(
                &self,
                ctx: &std::rc::Rc<workflow_allocator::context::Context<'info,'refs,'pid,'instr>>,
                rent_collector : &workflow_allocator::rent::RentCollector<'info,'refs>,
            ) -> workflow_allocator::result::Result<()> {
                // TODO: move out lamports from the account
                ctx.purge(self.account(),rent_collector)?;
                Ok(())
            }

            //
            // if this function presents are returning &{unknown}, 
            // you forgot to include solana-program as a dependency
            // into Cargo.toml
            //
            #[inline] 
            pub fn account(&self) -> &'refs solana_program::account_info::AccountInfo<'info> {
                self.#store_field_name.account
            }

            #[inline]
            pub fn pubkey(&self) -> &solana_program::pubkey::Pubkey {
                self.#store_field_name.account.key//.clone()
            }

            // #[inline]
            // pub fn test(&self) -> bool { true }

            // // #[inline]
            // pub fn print(&self) -> workflow_allocator::result::Result<()> {

            //     Ok(())
            // }

        }

        impl #struct_params #struct_name #struct_params #where_clause {
            fn segments() -> [usize;#sizes_ts_len] {
                [
                    #(#sizes_ts), *
                ]
            }
        }

/* 
        impl #struct_params workflow_allocator::container::Container<'info,'refs> 
        for #struct_name #struct_params 
        #where_clause
        {

            // pub 
            // fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self> {
            fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<#struct_name #struct_params> {
                #struct_path_with_generics::try_create(account)
            }

            // pub 
            // fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self> {
            fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<#struct_name #struct_params> {
                #struct_path_with_generics::try_load(account)
            }
        }
*/

        // #[cfg(not(target_arch = "bpf"))]
        // lazy_static!{
            
        //     //    static ref XREGISTRY : RefCell<BTreeMap<u32, ContainerDeclaration>> = RefCell::new(BTreeMap::new());
        //     pub static ref #struct_declaration : workflow_allocator::container::registry::ContainerDeclaration 
        //         = workflow_allocator::container::registry::ContainerDeclaration::new(#container_type as u32, #struct_name_str);
        // }

        #[cfg(not(any(target_arch = "bpf",target_arch = "wasm32")))]
        inventory::submit! {
            workflow_allocator::container::registry::ContainerDeclaration::new(#container_type as u32, #struct_name_str, 
                // &#struct_path_with_generics::print
            )
        }

        #[cfg(target_arch = "wasm32")]
        #[macro_use]
        mod #module_declaration {
            use super::*;
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn #container_declaration_register_() -> workflow_allocator::result::Result<()> {
            // pub fn container_declaration_register_() {
                let container_declaration = workflow_allocator::container::registry::ContainerDeclaration::new(
                    #container_type as u32,
                    #struct_name_str, 
                    // &#struct_path_with_generics::print
                );
                workflow_allocator::container::registry::register_container_declaration(
                    container_declaration
                )?;
                Ok(())
            }
        }


//        static CONTAINER_DECLARATION_ABC : ContainerDeclaration = ContainerDeclaration::new(123,"hello");



    };

    init.into()
}


// // use std::borrow::Cow;
// // use std::io::prelude::*;
// use std::path::Path;
// use std::*;

// pub fn store_container_type(container_type : &str) {
//     let filename = format!("container_types.rs");
//     let path = Path::new(&filename);
//     let display = path.display();

//     // Open a file in write-only mode, returns `io::Result<File>`
//     match std::fs::File::create(&path) {
//         Err(why) => {
//             println!("couldn't create {}: {}", display, why);
//             //return;
//         },
//         Ok(mut file) => {
//             //let j = serde_json::to_string(&data).unwrap();
//             //if let Err(error) = file.write_all(j.as_ref()) {

//             // let data = account_info.data.borrow();
//             let container_type_string = container_type.to_string();
//             let data = container_type_string.as_bytes();
//             if let Err(error) = file.write_all(&data[..]) {
//                 println!("unable to write to {} error: {}",display,error);
//             }
//         },
//     };

// }


// ~













