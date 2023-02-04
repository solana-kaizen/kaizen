use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use std::convert::Into;
// use proc_macro2::{Span, Ident, Group};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use workflow_macro_tools::attributes::*;
use workflow_macro_tools::parse_error;

use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Expr, PathArguments, Token,
};
use syn::{Error, GenericArgument, Type, TypePath, Visibility};

/*
#[segment(reserve(1024))]
#[segment(reserve = 1024)]
#[segment(reserve(size_of<T>*3))]
#[segment(reserve(MappedArray::size_with_records(3)))]
#[segment(resize, reserve(MappedArray::size_with_records(3)))]
#[segment(resize = true, reserve(MappedArray::size_with_records(3)))]
*/

const SEGMENT_ATTRIBUTES: &[&str] = &["fixed", "reserve", "flex"];
const COLLECTION_ATTRIBUTES: &[&str] = &["seed", "container", "container_type"];

#[derive(Debug)]
pub struct SegmentArgs {
    pub segment: Option<Args>,
    pub collection: Option<Args>,
}

#[derive(Debug, Clone)]
pub struct CollectionArgs {
    pub seed: Option<Value>,
    pub container: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct CollectionRefs {
    pub seed_const: Ident,
    pub container_type_const: Ident,
}

#[derive(Debug)]
struct Segment {
    args: SegmentArgs,
    flex: bool,
    collection: Option<CollectionArgs>,
    field_name: syn::Ident,
    name: String,
    type_name: Type,
    type_ident: Option<TypePath>,
    visibility: Visibility,
    type_name_args: Option<String>,
}

impl Segment {
    pub fn is_meta(&self) -> bool {
        self.name == "meta" || self.name == "_meta"
    }
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
    container_type: Expr,
    index_size_type: TokenStream2,
}

impl Parse for ContainerAttributes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(input).unwrap();
        if parsed.len() < 1 || parsed.len() > 2 {
            return Err(Error::new_spanned(
                parsed[2].clone(),
                format!("usage: #[container(<container type id>, <index size: u16 or u32>)]"),
            ));
        }

        let mut iter = parsed.iter();
        let container_type = iter.next().clone().unwrap().clone();
        let index_size_type: TokenStream2 = if parsed.len() > 1 {
            let index_size_type = iter.next().clone().unwrap().clone();
            let index_size_type_str = index_size_type.to_token_stream().to_string();
            match index_size_type_str.as_str() {
                "u32" | "u16" => index_size_type.to_token_stream().into(),

                _ => {
                    return Err(Error::new_spanned(
                        index_size_type,
                        format!("the second argument should be a type u16 or u32)"),
                    ));
                }
            }
        } else {
            (quote! { u16 }).into()
        };

        Ok(ContainerAttributes {
            container_type,
            index_size_type: index_size_type.into(),
        })
    }
}

pub fn container_attribute_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cattr = parse_macro_input!(attr as ContainerAttributes);
    let struct_decl_src = item.clone();
    let ast = parse_macro_input!(struct_decl_src as DeriveInput);
    let struct_name = &ast.ident;
    let struct_params = &ast.generics;

    let mut generics_only = ast.generics.clone();
    generics_only.params = {
        let mut params: Punctuated<syn::GenericParam, Token![,]> = Punctuated::new();
        for param in generics_only.params.iter() {
            match param {
                syn::GenericParam::Type(_) => {
                    params.push(param.clone());
                }
                _ => {}
            }
        }
        params
    };
    let has_generics = generics_only.params.len() > 0;
    let where_clause = match generics_only.where_clause.clone() {
        Some(where_clause) => quote! { #where_clause },
        None => quote! {},
    };

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        return parse_error(ast, "#[container] macro only supports structs").into();
    };

    let mut flex_segments = 0;
    let mut segments: Vec<Segment> = Vec::new();
    for field in fields.named.iter() {
        let field_name: syn::Ident = field.ident.as_ref().unwrap().clone();
        let name: String = field_name.to_string();

        let mut args = SegmentArgs {
            segment: None,
            collection: None,
        };

        for attr in field.attrs.iter() {
            if attr.path.is_ident("segment") {
                if args.segment.is_some() {
                    return parse_error(
                        attr.clone(),
                        &format!("#[container]: more than one #[segment()] attributes while processing {}", name)
                    ).into();
                }

                match get_attributes(attr) {
                    Some(attrs) => {
                        if let Err(err) = attrs.allow(SEGMENT_ATTRIBUTES) {
                            return err.to_compile_error().into();
                        }
                        args.segment = Some(attrs);
                    }
                    None => {}
                }
            } else if attr.path.is_ident("collection") {
                if args.collection.is_some() {
                    return parse_error(
                        attr.clone(),
                        &format!("#[container]: more than one #[collection()] attributes while processing {}", name)
                    ).into();
                }

                match get_attributes(attr) {
                    Some(attrs) => {
                        if let Err(err) = attrs.allow(COLLECTION_ATTRIBUTES) {
                            return err.to_compile_error().into();
                        }
                        args.collection = Some(attrs);
                    }
                    None => {}
                }
            }
        }

        let flex = if let Some(args) = &args.segment {
            args.get("flex").is_some()
        } else {
            false
        };
        if flex {
            flex_segments += 1;
        }

        if flex_segments > 1 {
            return Error::new_spanned(
                field_name.clone(),
                format!("multiple flex attributes are not supported"),
            )
            .to_compile_error()
            .into();
        }

        let collection_seed = if let Some(args) = &args.collection {
            match args.get_value_or("seed", field_name.clone(), "missing seed value") {
                Ok(value) => value,
                Err(err) => return err.into(),
            }
        } else {
            None
        };

        let collection_container = if let Some(args) = &args.collection {
            match args.get_value_or("container", field_name.clone(), "missing container value") {
                Ok(value) => value,
                Err(err) => return err.into(),
            }
        } else {
            None
        };

        let type_name = field.ty.clone();
        let visibility = field.vis.clone();
        let type_name_for_ident = type_name.clone();

        let (type_ident, type_name_args) = match type_name_for_ident {
            Type::Path(mut type_path) => {
                let target = type_path.path.segments.last_mut().unwrap();
                let type_name_args = match &target.arguments {
                    PathArguments::AngleBracketed(params) => {
                        let mut types: Vec<String> = Vec::new();
                        for arg in params.args.iter() {
                            match arg {
                                GenericArgument::Type(arg_type) => {
                                    types.push(arg_type.to_token_stream().to_string());
                                }
                                _ => {}
                            }
                        }

                        target.arguments = PathArguments::None;
                        Some(types.join(","))
                    }
                    _ => None,
                };

                (Some(type_path), type_name_args)
            }
            _ => (None, None),
        };

        let collection = if collection_seed.is_some() || collection_container.is_some() {
            Some(CollectionArgs {
                seed: collection_seed,
                container: collection_container,
            })
        } else {
            if args.collection.is_some() {
                return parse_error(
                    field_name.clone(),
                    "collection attribute requires seed field: #[collection(seed(b\"...\"))]",
                )
                .into();
            }

            None
        };

        let seg = Segment {
            args,
            flex,
            collection,
            visibility,
            field_name,
            name,
            type_name,
            type_ident,
            type_name_args,
        };

        segments.push(seg);
    }

    let mut store_field_visibility = quote! {};
    let mut store_field_name = quote! { __store__ };
    let mut store_field_type = quote! { kaizen::container::segment::SegmentStore<'info,'refs> };
    // let mut idx : usize = 0;
    for segment in segments.iter() {
        match segment.type_ident.as_ref() {
            None => continue,
            Some(type_name_ident) => {
                if type_name_ident.path.is_ident("SegmentStore") {
                    let ts2: TokenStream2 = segment.name.parse().unwrap();
                    store_field_name = quote! { #ts2 };
                    let type_name = &segment.type_name;
                    store_field_type = quote! { #type_name };
                    let visibility = &segment.visibility;
                    store_field_visibility = quote! { #visibility };
                    break;
                }
            }
        }
    }

    // filter our store field
    let segments = segments
        .into_iter()
        .filter(|seg| !seg.is_store())
        .collect::<Vec<_>>();

    let mut collection_refs = HashMap::<String, CollectionRefs>::new();
    let mut collection_inits = Vec::new();
    for segment in segments.iter() {
        if let Some(collection) = &segment.collection {
            let collection_seed_const = Ident::new(
                &format!("COLLECTION_SEED_{}", &segment.name.to_uppercase()),
                Span::call_site(),
            );
            match &collection.seed {
                Some(seed) => {
                    let seed = seed.to_token_stream();
                    collection_inits
                        .push(quote! { const #collection_seed_const: &'static [u8] = #seed; });
                }
                None => {
                    return parse_error(
                        segment.field_name.clone(),
                        "missing seed attribute parameter",
                    )
                    .into();
                }
            }
            let collection_container_type_const = Ident::new(
                &format!("COLLECTION_CONTAINER_{}", &segment.name.to_uppercase()),
                Span::call_site(),
            );
            match &collection.container {
                Some(container) => {
                    let container = container.to_token_stream();
                    collection_inits.push(quote!{ const #collection_container_type_const: Option<u32> = Some(#container :: CONTAINER_TYPE); });
                }
                None => {
                    collection_inits.push(
                        quote! { const #collection_container_type_const: Option<u32> = None; },
                    );
                }
            }

            collection_refs.insert(
                segment.name.clone(),
                CollectionRefs {
                    seed_const: collection_seed_const,
                    container_type_const: collection_container_type_const,
                },
            );
        }
    }

    let mut inits = Vec::new();
    let mut loads = Vec::new();

    let mut meta_type_path: Option<TypePath> = None;

    let mut flex: Option<usize> = None;
    let mut idx: usize = 0;
    for segment in segments.iter() {
        let field_name = &segment.field_name;
        if segment.name == "store" || segment.name == "_store" {
            continue;
        }

        if segment.name == "meta" || segment.name == "_meta" {
            let bind_meta = match &segment.type_name {
                Type::Reference(reference) => {
                    if reference.mutability.is_none() {
                        return Error::new_spanned(
                            reference.clone(),
                            format!("meta must be &'info mut reference"),
                        )
                        .to_compile_error()
                        .into();
                    }
                    if let Type::Path(type_path) = &*reference.elem {
                        meta_type_path = Some(type_path.clone());
                    }
                    let type_name = segment.type_name.clone();
                    let meta_name = Ident::new(&segment.name, Span::call_site());
                    quote! {
                        let #meta_name : #type_name = {
                            let mut data = #store_field_name.account.data.borrow_mut();
                            unsafe { &mut *data.as_mut_ptr().add(container_meta_offset).cast::<_>() }
                            // unsafe { std::mem::transmute(&mut data[container_meta_offset]) }
                        };
                    }
                }
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
                                                        format!(
                                                            "meta must be &'info mut reference"
                                                        ),
                                                    )
                                                    .to_compile_error()
                                                    .into();
                                                }
                                                if let Type::Path(type_path) = &*reference.elem {
                                                    meta_type_path = Some(type_path.clone());
                                                }
                                                let type_name = segment.type_name.clone();
                                                let meta_name =
                                                    Ident::new(&segment.name, Span::call_site());

                                                quote! {
                                                    let #meta_name : #type_name = {
                                                        let mut data = #store_field_name.account.data.borrow_mut();
                                                        // let meta = unsafe { std::mem::transmute(&mut data[container_meta_offset]) };  // @meta
                                                        let meta = unsafe { &mut *data.as_mut_ptr().add(container_meta_offset).cast::<_>() };  // @meta
                                                        RefCell::new(meta)
                                                    };
                                                }
                                            }
                                            _ => {
                                                return Error::new_spanned(
                                                    ref_cell.clone(),
                                                    format!("RefCell generic arguments: expecting type or type reference") 
                                                )
                                                .to_compile_error()
                                                .into();
                                            }
                                        }
                                    }
                                    _ => {
                                        return Error::new_spanned(
                                            ref_cell.clone(),
                                            format!("RefCell generic arguments: expecting type or type reference")
                                        )
                                        .to_compile_error()
                                        .into();
                                    }
                                }
                            }
                            _ => {
                                return Error::new_spanned(
                                    ref_cell.clone(),
                                    format!("expecting AngleBracketed arguments for RefCell"),
                                )
                                .to_compile_error()
                                .into();
                            }
                        }
                    } else {
                        quote! {}
                    }
                }
                _ => {
                    quote! {}
                }
            };

            inits.push(quote! {
                #bind_meta
            });
            loads.push(quote! {
                #bind_meta
            });

            continue;
        } // if meta

        idx = idx + 1;
        let type_name = &segment.type_name;
        let type_ident = &segment.type_ident;

        if segment.flex {
            flex = Some(idx);
        }

        if let Some(collection) = &collection_refs.get(&segment.name) {
            let seed_const = collection.seed_const.clone();
            let container_type_const = collection.container_type_const.clone();
            inits.push(quote!{
                let segment = #store_field_name.try_get_segment_at(#idx)?;
                let #field_name : #type_name  = #type_ident::try_create_from_segment_with_collection_args(
                    segment,
                    #struct_name :: #seed_const,
                    #struct_name :: #container_type_const,
                )?;
            });
            loads.push(quote!{
                let segment = #store_field_name.try_get_segment_at(#idx)?;
                let #field_name : #type_name  = #type_ident::try_load_from_segment_with_collection_args(
                    segment,
                    #struct_name :: #seed_const,
                    #struct_name :: #container_type_const
                )?;
            });
        } else {
            inits.push(quote! {
                let segment = #store_field_name.try_get_segment_at(#idx)?;
                let #field_name : #type_name  = #type_ident::try_create_from_segment(segment)?;
            });
            loads.push(quote! {
                let segment = #store_field_name.try_get_segment_at(#idx)?;
                let #field_name : #type_name  = #type_ident::try_load_from_segment(segment)?;
            });
        }
    }

    let field_ident_vec: Vec<Ident> = segments.iter().map(|f| f.field_name.clone()).collect();
    let field_idents = quote! {
        #store_field_name,
        #(#field_ident_vec),*
    };
    let field_type_vec: Vec<Type> = segments.iter().map(|f| f.type_name.clone()).collect();
    let field_visibility_vec: Vec<Visibility> =
        segments.iter().map(|f| f.visibility.clone()).collect();

    let inits_create = quote! {
        #struct_name {
            #field_idents
        }
    };

    let loads_create = quote! {
        #struct_name {
            #field_idents
        }
    };

    let mut inits_ts2 = TokenStream2::new();
    inits_ts2.extend(inits);
    let mut loads_ts2 = TokenStream2::new();
    loads_ts2.extend(loads);

    let struct_redef = quote! {
        pub struct #struct_name #struct_params #where_clause {
            #store_field_visibility #store_field_name : #store_field_type,
            #( #field_visibility_vec #field_ident_vec : #field_type_vec ),*
        }
    };

    let mut sizes_ts = Vec::new();
    for segment in segments.iter() {
        if segment.type_ident.is_none() || segment.is_meta() || segment.is_store() {
            continue;
        }

        let ts_ = if let Some(args) = &segment.args.segment {
            let reserve = Ident::new("reserve", Span::call_site());
            match args.map.get(&reserve) {
                Some(reserve) => match reserve {
                    Some(reserve) => {
                        let ts = reserve.to_token_stream();
                        Some(ts.clone())
                    }
                    None => {
                        return Error::new_spanned(
                                segment.field_name.clone(),
                                format!("#[segment()]: reserve attribute for segment '{}' must contain a value",segment.name)
                            )
                            .to_compile_error()
                            .into();
                    }
                },
                None => None,
            }
        } else {
            None
        };

        let ts = match ts_ {
            Some(ts) => ts.clone(),
            None => {
                if segment.type_name_args.is_some() {
                    let ident = segment.type_ident.clone().unwrap();
                    let args = segment.type_name_args.clone().unwrap();
                    let args: proc_macro2::TokenStream = args.parse().unwrap();

                    let ts = quote! { #ident::<#args>::data_len_min()  };
                    ts
                } else {
                    let ident = segment.type_name.clone();
                    let ts = quote! { #ident::data_len_min()  };
                    ts
                }
            }
        };

        sizes_ts.push(ts.clone());
    }

    let sizes_ts_len = sizes_ts.len();

    let init_offset = if let Some(type_path) = &meta_type_path {
        quote! {
            let container_meta_offset = std::mem::size_of::<kaizen::container::ContainerHeader>();
            let segment_store_offset = std::mem::size_of::<kaizen::container::ContainerHeader>()
                + std::mem::size_of::<#type_path>();
        }
    } else {
        quote! {
            let segment_store_offset = std::mem::size_of::<kaizen::container::ContainerHeader>();
        }
    };

    let container_type = cattr.container_type;
    let index_unit_size = cattr.index_size_type;
    let struct_name_str = struct_name.to_string();
    let module_declaration = Ident::new(
        &format!("init_{}", struct_name_str).to_lowercase(),
        struct_name.span(),
    );
    let container_declaration_register_ = Ident::new(
        &format!("container_declaration_register_{}", struct_name_str).to_lowercase(),
        struct_name.span(),
    );

    let try_create_with_meta = if let Some(_type_path) = &meta_type_path {
        quote! {}
    } else {
        quote! {}
    };

    let struct_path_with_generics = if has_generics {
        quote! { #struct_name::#generics_only }
    } else {
        quote! { #struct_name }
    };

    let init_layout = match flex {
        None => {
            quote! {
                let layout = Self::layout();
            }
        }
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

    let init = quote! {

        #struct_redef

        impl #struct_params #struct_name #struct_params #where_clause {

            pub const CONTAINER_TYPE: u32 = #container_type as u32;

            #(#collection_inits)*

            pub fn try_allocate_default<'pid,'instr>(
                ctx: &std::rc::Rc<std::boxed::Box<kaizen::context::Context<'info,'refs,'pid,'instr>>>,
                allocation_args : &kaizen::context::AccountAllocationArgs<'info,'refs,'_>,
            ) -> kaizen::result::Result<#struct_name #struct_params> {

                Self::try_allocate(ctx,allocation_args,0)
            }

            pub fn try_allocate(
                ctx: &kaizen::context::ContextReference<'info,'refs,'_,'_>,
                allocation_args : &kaizen::context::AccountAllocationArgs<'info,'_,'_>,
                reserve_data_len : usize
            ) -> kaizen::result::Result<#struct_name #struct_params> {

                let data_len = Self::initial_data_len() + reserve_data_len;
                let account_info = ctx.try_create_pda(data_len,allocation_args)?;
                Self::try_create(account_info)
            }


            pub fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> kaizen::result::Result<#struct_name #struct_params> {

                #init_offset
                let container_type : u32 = #container_type as u32;
                #init_layout
                let #store_field_name = kaizen::container::segment::SegmentStore::try_create(
                    &account, segment_store_offset, &layout,
                )?;
                #inits_ts2

                {
                    let mut data = account.data.borrow_mut();
                    let header = unsafe { &mut *data.as_mut_ptr().cast::<&mut kaizen::container::ContainerHeader>() };
                    header.set_container_type(container_type);
                }

                Ok(#inits_create)
            }

            pub fn try_create_with_layout(account : &'refs solana_program::account_info::AccountInfo<'info>, layout : &kaizen::container::segment::Layout<#index_unit_size>) -> kaizen::result::Result<#struct_name #struct_params> {

                #init_offset
                let container_type : u32 = #container_type as u32;
                let #store_field_name = kaizen::container::segment::SegmentStore::try_create(
                    &account, segment_store_offset, &layout,
                )?;

                #inits_ts2

                {
                    let mut data = account.data.borrow_mut();
                    let header = unsafe { &mut *data.as_mut_ptr().cast::<&mut kaizen::container::ContainerHeader>() };
                    header.set_container_type(container_type);
                }

                Ok(#inits_create)
            }

            pub fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> kaizen::result::Result<#struct_name #struct_params> {

                #init_offset
                let container_type : u32 = #container_type as u32;

                {
                    let mut data = account.data.borrow_mut();
                    let header = unsafe { &mut *data.as_mut_ptr().cast::<&mut kaizen::container::ContainerHeader>()};

                    let header_container_type = header.get_container_type();
                    if header_container_type != container_type {
                        #[cfg(not(target_os = "solana"))] {
                            let header_container_type_str = if let Ok(Some(declaration)) = kaizen::container::registry::lookup(header_container_type) {
                                declaration.name
                            } else { "n/a" };
                            let container_type_str = if let Ok(Some(declaration)) = kaizen::container::registry::lookup(container_type) {
                                declaration.name
                            } else { "n/a" };

                            workflow_log::log_error!("Container type mismatch - expecting: {} 0x{:08x} receiving: {} 0x{:08x}",
                                workflow_log::style(container_type_str).red(),
                                container_type,
                                workflow_log::style(header_container_type_str).red(),
                                header_container_type,
                            );
                        }
                        return Err(
                            kaizen::error::Error::new()
                                .with_code(kaizen::error::ErrorCode::ContainerTypeMismatch)
                                .with_source(file!(),line!())
                        );
                    }
                }

                let layout = Self::layout();
                let #store_field_name = kaizen::container::segment::SegmentStore::try_load(
                    &account, segment_store_offset,
                )?;

                #loads_ts2

                Ok(#loads_create)
            }

            #try_create_with_meta

            #[inline]
            pub fn layout() -> kaizen::container::segment::Layout<#index_unit_size> {
                kaizen::container::segment::Layout::<#index_unit_size>::from(&#struct_path_with_generics::segments()) // @aspect
            }

            #[inline]
            pub fn initial_data_len() -> usize {
                #init_offset
                #struct_path_with_generics::layout().data_len() + segment_store_offset // @aspect
            }

            #[inline]
            pub fn sync_rent<'pid,'instr>(
                &self,
                ctx: &kaizen::context::ContextReference<'info,'refs,'pid,'instr>,
                rent_collector : &kaizen::rent::RentCollector<'info,'refs>,
            ) -> kaizen::result::Result<()> {
                ctx.sync_rent(self.account(),rent_collector)?;
                Ok(())
            }

            #[inline]
            pub fn purge<'pid,'instr>(
                &self,
                ctx: &kaizen::context::ContextReference<'info,'refs,'pid,'instr>,
                rent_collector : &kaizen::rent::RentCollector<'info,'refs>,
            ) -> kaizen::result::Result<()> {
                // TODO: move out lamports from the account
                ctx.purge(self.account(),rent_collector)?;
                Ok(())
            }

            //
            // if this function presents are returning &{unknown},
            // you forgot to include solana-program as a dependency
            // into Cargo.toml
            //
            #[inline(always)]
            pub fn account(&self) -> &'refs solana_program::account_info::AccountInfo<'info> {
                self.#store_field_name.account
            }

            #[inline(always)]
            pub fn pubkey(&self) -> &solana_program::pubkey::Pubkey {
                self.#store_field_name.account.key//.clone()
            }

        }

        impl #struct_params #struct_name #struct_params #where_clause {
            fn segments() -> [usize;#sizes_ts_len] {
                [
                    #(#sizes_ts), *
                ]
            }
        }

        impl #struct_params kaizen::container::Container<'info,'refs> for #struct_name #struct_params #where_clause {
            type T = Self;

            fn container_type() -> u32 {
                #container_type as u32
            }

            fn initial_data_len() -> usize {
                #struct_path_with_generics :: initial_data_len()
            }

            fn account(&self) -> &'refs solana_program::account_info::AccountInfo<'info> {
                self.account()
            }

            fn pubkey(&self) -> &solana_program::pubkey::Pubkey {
                self.pubkey()
            }

            fn try_allocate(
                ctx: &kaizen::context::ContextReference<'info,'refs,'_,'_>,
                allocation_args : &kaizen::context::AccountAllocationArgs<'info,'refs,'_>,
                reserve_data_len : usize
            ) -> kaizen::result::Result<#struct_name #struct_params> {
                #struct_name :: #struct_params :: try_allocate(ctx, allocation_args, reserve_data_len)
            }

            fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> kaizen::result::Result<#struct_name #struct_params> {
                #struct_name :: #struct_params :: try_create(account)
            }

            fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> kaizen::result::Result<#struct_name #struct_params> {
                #struct_name :: #struct_params :: try_load(account)
            }

        }

        #[cfg(not(any(target_os = "solana",target_arch = "wasm32")))]
        kaizen::inventory::submit! {
            kaizen::container::registry::ContainerDeclaration::new(
                #container_type as u32,
                #struct_name_str,
            )
        }

        #[cfg(target_arch = "wasm32")]
        #[macro_use]
        mod #module_declaration {
            use super::*;
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn #container_declaration_register_() -> kaizen::result::Result<()> {
                let container_declaration = kaizen::container::registry::ContainerDeclaration::new(
                    #container_type as u32,
                    #struct_name_str,
                );
                kaizen::container::registry::register_container_declaration(
                    container_declaration
                )?;
                Ok(())
            }
        }
    };

    init.into()
}
