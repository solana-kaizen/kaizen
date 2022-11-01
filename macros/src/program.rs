
use std::convert::Into;
use proc_macro::{TokenStream};
use proc_macro2::{Span, Ident};
use quote::quote;
use syn::{
    Result, parse_macro_input, ExprArray, PathArguments, ExprLit,
    punctuated::Punctuated, Expr, Token, 
    parse::{Parse, ParseStream}, PathSegment, Error, Lit,
};


#[derive(Debug)]
struct Program {
    program_id_string : String,
    program_name : ExprLit,
    program_id : ExprLit,
    primitive_handlers : ExprArray
}

impl Parse for Program {
    fn parse(input: ParseStream) -> Result<Self> {

        let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(input).unwrap();
        if parsed.len() != 3 {
            return Err(Error::new_spanned(
                parsed,
                format!("usage: declare_handlers!(<program_name>,<program_id>,[<primitive_dispatch_program>, ..])")
            ));
        }

        let mut iter = parsed.iter();
        let program_name_expr = iter.next().clone().unwrap().clone();
        let program_name = match program_name_expr {
            Expr::Lit(lit) => lit,
            _ => {
                return Err(Error::new_spanned(
                    program_name_expr,
                    format!("the first argument should be the program_name)")
                ));
            }
        };

        let program_id_expr = iter.next().clone().unwrap().clone();
        let program_id = match &program_id_expr {
            Expr::Lit(lit) => lit.clone(),
            _ => {
                return Err(Error::new_spanned(
                    program_id_expr,
                    format!("the second argument should be the program_id)")
                ));
            }
        };

        let program_id_string = match &program_id.lit {
            Lit::Str(lit) => {
                lit.value().to_string()
            },
            _ => {
                return Err(Error::new_spanned(
                    program_id_expr,
                    format!("handlers should contain path to struct")
                ));
            }
        };


        let primitive_handlers_ = iter.next().clone().unwrap().clone();
        let mut primitive_handlers = match primitive_handlers_ {
            Expr::Array(array) => array,
            _ => {
                return Err(Error::new_spanned(
                    primitive_handlers_,
                    format!("the third argument must be an array of static functions")
                ));
            }
        };

        for ph in primitive_handlers.elems.iter_mut() {
            match ph {
                Expr::Path(path) => {
                    let ident = Ident::new("program", Span::call_site());
                    let path_segment = PathSegment { ident, arguments : PathArguments::None };
                    path.path.segments.push_punct(Token![::](Span::call_site()));
                    path.path.segments.push_value(path_segment);
                },
                _ => {
                    return Err(Error::new_spanned(
                        ph,
                        format!("handlers should contain path to struct")
                    ));
                }
            }
        }


        let handlers = Program {
            program_id_string,
            program_name,
            program_id,
            primitive_handlers,
        };
        Ok(handlers)
    }
}


// #[proc_macro]
pub fn declare_program(input: TokenStream) -> TokenStream {

    let program = parse_macro_input!(input as Program);
    let primitive_handlers = program.primitive_handlers;
    let len = primitive_handlers.elems.len();
    let program_id = program.program_id;
    let program_name = program.program_name;
    let program_id_string = program.program_id_string;
    let entrypoint_declaration_register_ = Ident::new(
        &format!("entrypoint_declaration_register_{}",program_id_string), 
        Span::call_site()
    );

    let output = quote!{
        pub static PROGRAM_HANDLERS : [workflow_allocator::context::HandlerFn;#len] = #primitive_handlers;

        solana_program::declare_id!(#program_id);
        solana_program::entrypoint!(process_instruction);

        #[inline(never)]
        pub fn init() -> solana_program::pubkey::Pubkey { id() }

        #[inline(always)]
        pub fn program_id() -> solana_program::pubkey::Pubkey { id() }

        pub fn program_name() -> &'static str { #program_name }

        #[inline(always)]
        pub fn program_handlers() -> &'static [workflow_allocator::context::HandlerFn] { &PROGRAM_HANDLERS[..] }

        #[cfg(not(target_arch = "bpf"))]
        pub fn interface_id(handler_fn: workflow_allocator::context::HandlerFn) -> usize {
            PROGRAM_HANDLERS.iter()
                .position(|&hfn| hfn as workflow_allocator::context::HandlerFnCPtr == handler_fn as workflow_allocator::context::HandlerFnCPtr )
                .expect("Unknown interface handler! (check declare_program!())")
        }

        pub fn program(ctx:&workflow_allocator::context::ContextReference) -> solana_program::entrypoint::ProgramResult {
            if ctx.interface_id >= PROGRAM_HANDLERS.len() {
                println!("Error - invalid interface id");
                return Err(solana_program::program_error::ProgramError::InvalidArgument);
            }
            Ok(PROGRAM_HANDLERS[ctx.interface_id](ctx)?)
        }

        #[cfg(not(feature = "no-entrypoint"))]
        pub fn process_instruction(
            program_id: &solana_program::pubkey::Pubkey,
            accounts: &[solana_program::account_info::AccountInfo],
            instruction_data: &[u8],
        ) -> solana_program::entrypoint::ProgramResult {
            // solana_program::msg!("program_id: {}", program_id);
            // solana_program::msg!("accounts: {:?}", accounts);
            // solana_program::msg!("instruction_data: {:?}", instruction_data);
            match workflow_allocator::context::Context::try_from((program_id,accounts,instruction_data)) {
                Err(err) => {
                    #[cfg(not(target_arch = "bpf"))]
                    workflow_log::log_error!("Fatal: unable to load Context: {}", err);
                    return Err(err.into());
                },
                Ok(mut ctx) => {
                    PROGRAM_HANDLERS[ctx.interface_id](&mut std::rc::Rc::new(std::boxed::Box::new(ctx)))?;
                }
            }

            Ok(())
        }

        #[cfg(not(any(target_arch = "bpf",target_arch = "wasm32")))]
        inventory::submit! {
            workflow_allocator::program::registry::EntrypointDeclaration::new(
                ID,
                #program_name,
                process_instruction
            )
        }

        #[cfg(target_arch = "wasm32")]
        #[macro_use]
        mod wasm {
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn #entrypoint_declaration_register_() -> workflow_allocator::result::Result<()> {
                workflow_allocator::program::registry::register_entrypoint_declaration(
                    workflow_allocator::program::registry::EntrypointDeclaration::new(
                        super::ID,
                        super::program_name(),
                        super::process_instruction
                    )
                )?;
                Ok(())
            }
        }
    };

    output.into()
}
