use std::convert::Into;
use proc_macro::{TokenStream};
use quote::{quote, ToTokens};
use syn::{
    Result, parse_macro_input, ExprArray, PathArguments,
    punctuated::Punctuated, Expr, Token, 
    parse::{Parse, ParseStream}, Error,
};


#[derive(Debug)]
struct Primitive {
    // handler_struct : ExprPath,
    handler_struct_decl : String,
    handler_lifetimes : Option<String>,
    handler_methods : ExprArray
}

impl Parse for Primitive {
    fn parse(input: ParseStream) -> Result<Self> {

        let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(input).unwrap();
        if parsed.len() != 2 {
            return Err(Error::new_spanned(
                parsed,
                format!("usage: declare_handlers!(<struct>,[<method>, ..])")
            ));

        }

        let handler_struct_expr = parsed.first().unwrap().clone();
        let mut handler_struct = match handler_struct_expr {
            Expr::Path(path) => path,
            _ => {
                return Err(Error::new_spanned(
                    handler_struct_expr,
                    // format!("unsupported segment attribute: {}, supported attributes are {}", name, SEGMENT_ATTRIBUTES.join(", "))
                    format!("first argument should be a struct name (and an optional lifetime)")
                ));
            }
        };

        let mut target = handler_struct.path.segments.last_mut().unwrap();
        let handler_lifetimes = match &target.arguments {
            PathArguments::AngleBracketed(params) => {
                let mut ts = proc_macro2::TokenStream::new();
                params.args.clone().to_tokens(&mut ts);
                let lifetimes = ts.to_string();
                target.arguments = PathArguments::None;
                Some(lifetimes)
            },
            _ => None
        };

        let mut ts = proc_macro2::TokenStream::new();
        handler_struct.to_tokens(&mut ts);
        let handler_struct_decl = ts.to_string();

        let handler_methods_ = parsed.last().unwrap().clone();
        let handler_methods = match handler_methods_ {
            Expr::Array(array) => array,
            _ => {
                return Err(Error::new_spanned(
                    handler_methods_,
                    format!("second argument must be an array of static functions")
                ));
            }
        };

        let handlers = Primitive {
            // handler_struct,
            handler_struct_decl,
            handler_lifetimes, // : lifetimes.unwrap_or("".into()),
            handler_methods
        };
        Ok(handlers)
    }
}


// #[proc_macro]
pub fn declare_handlers(input: TokenStream) -> TokenStream {

    let primitive = parse_macro_input!(input as Primitive);
    let handler_struct_name = primitive.handler_struct_decl.to_string();
    let handler_methods = primitive.handler_methods;
    let len = handler_methods.elems.len();
    let impl_str = match &primitive.handler_lifetimes {
        Some(lifetimes) => format!("impl<{}> {}<{}>",lifetimes, primitive.handler_struct_decl, lifetimes),
        None => format!("impl {}", primitive.handler_struct_decl),
    };
    let impl_ts: proc_macro2::TokenStream = impl_str.parse().unwrap();

    // let struct_decl = primitive.struct_decl;//.clone();
    let handler_struct_path: proc_macro2::TokenStream = primitive.handler_struct_decl.parse().unwrap();


    let output = quote!{
        // pub static PRIMITIVE_HANDLERS : [HandlerFn;#len] = #handler_methods;

        #impl_ts {

            pub const PRIMITIVE_HANDLERS : [HandlerFn;#len] = #handler_methods;

            pub fn bind() -> &'static str { #handler_struct_name }
            // pub const fn handlers() -> &'static [HandlerFn] { &PRIMITIVE_HANDLERS[..] }

            pub fn handler_id(handler_fn: HandlerFn) -> u16 {
                #handler_struct_path::PRIMITIVE_HANDLERS.iter()
                .position(|&hfn| hfn as HandlerFnCPtr == handler_fn as HandlerFnCPtr )
                .expect("invalid primitive handler")
                as u16
            }

            pub fn program(ctx:&std::rc::Rc<workflow_allocator::context::Context>) -> solana_program::entrypoint::ProgramResult {
                if ctx.handler_id >= #handler_struct_path::PRIMITIVE_HANDLERS.len() {
                    println!("Error - invalid argument in program handler");
                    return Err(solana_program::program_error::ProgramError::InvalidArgument);
                }
                // println!("executing program ctx: {:#?}", ctx);
                #handler_struct_path::PRIMITIVE_HANDLERS[ctx.handler_id](ctx)
            }
        }
    };

    output.into()
}

// ~