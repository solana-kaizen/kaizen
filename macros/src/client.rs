use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::convert::Into;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Error, Expr, ExprPath, PathArguments, PathSegment, Result, Token,
};

#[derive(Debug)]
struct Execution {
    target_primitive_path: ExprPath,
    interface_dispatch_method: ExprPath,
    client_struct_decl: TokenStream,
    client_lifetimes: Option<String>,
}

impl Parse for Execution {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(input).unwrap();
        if parsed.len() != 2 {
            return Err(Error::new_spanned(
                parsed.clone(),
                format!("usage: declare_handlers!(<struct>,[<method>, ..])"),
            ));
        }

        let handler_struct_expr = parsed.first().unwrap().clone();
        let target_primitive_path = match handler_struct_expr {
            Expr::Path(path) => path,
            _ => {
                return Err(Error::new_spanned(
                    handler_struct_expr.clone(),
                    format!("first argument should be a struct name (and an optional lifetime)"),
                ));
            }
        };

        let mut interface_dispatch_method = target_primitive_path.clone();

        let ident = Ident::new("program", Span::call_site());
        let path_segment = PathSegment {
            ident,
            arguments: PathArguments::None,
        };
        interface_dispatch_method
            .path
            .segments
            .push_punct(Token![::](Span::call_site()));
        interface_dispatch_method
            .path
            .segments
            .push_value(path_segment);

        let client_struct_expr = parsed.last().unwrap().clone();
        let mut client_struct = match client_struct_expr {
            Expr::Path(path) => path,
            _ => {
                return Err(Error::new_spanned(
                    client_struct_expr.clone(),
                    format!("last argument should be a struct name (and an optional lifetime)"),
                ));
            }
        };

        let mut target = client_struct.path.segments.last_mut().unwrap();
        let client_lifetimes = match &target.arguments {
            PathArguments::AngleBracketed(params) => {
                let mut ts = proc_macro2::TokenStream::new();
                params.args.clone().to_tokens(&mut ts);
                let lifetimes = ts.to_string();
                target.arguments = PathArguments::None;
                Some(lifetimes)
            }
            _ => None,
        };

        let mut ts = proc_macro2::TokenStream::new();
        client_struct.to_tokens(&mut ts);
        let client_struct_decl: TokenStream = ts.into();

        let execution = Execution {
            target_primitive_path,
            interface_dispatch_method,
            client_struct_decl,
            client_lifetimes,
        };
        Ok(execution)
    }
}

// #[proc_macro]
pub fn declare_client(input: TokenStream) -> TokenStream {
    let execution = parse_macro_input!(input as Execution);

    let target_primitive_path = execution.target_primitive_path;
    let interface_dispatch_method = execution.interface_dispatch_method;
    let client_struct_name = execution.client_struct_decl.to_string();

    let impl_wasm_str = match &execution.client_lifetimes {
        Some(lifetimes) => format!(
            "impl<{}> {}<{}>",
            lifetimes, execution.client_struct_decl, lifetimes
        ),
        None => format!("impl {}", execution.client_struct_decl),
    };
    let impl_wasm_ts: proc_macro2::TokenStream = impl_wasm_str.parse().unwrap();

    let impl_client_str = match &execution.client_lifetimes {
        Some(lifetimes) => format!(
            "impl Client {}<{}>",
            execution.client_struct_decl, lifetimes
        ),
        None => format!("impl Client for {}", execution.client_struct_decl),
    };
    let impl_client_ts: proc_macro2::TokenStream = impl_client_str.parse().unwrap();

    let out = quote! {

        #impl_client_ts {
            fn handler_id(handler_fn: HandlerFn) -> usize {

                #target_primitive_path::INTERFACE_HANDLERS.iter()
                .position(|&hfn| hfn as HandlerFnCPtr == handler_fn as HandlerFnCPtr )
                .expect("invalid primitive handler")
            }

            fn execution_context_for(handler: HandlerFn) -> Arc<InstructionBuilder> {
                let interface_id = crate::interface_id(#interface_dispatch_method);
                let handler_id = Self::handler_id(handler);

                InstructionBuilder::new(
                    // program_id,
                    &crate::program_id(),
                    interface_id,
                    handler_id as u16
                )
            }
        }

        #impl_wasm_ts {

            pub fn bind() -> &'static str { #client_struct_name }

            pub async fn execute(
                instruction : solana_program::instruction::Instruction
            ) -> kaizen::result::Result<()> {
                use kaizen::transport::Interface;
                let transport = kaizen::transport::Transport::global()?;
                transport.execute(&instruction).await?;
                Ok(())
                // transport.execute(&instruction).await
            }

            pub async fn execute_with_transport(
                transport : &Arc<kaizen::transport::Transport>,
                instruction : solana_program::instruction::Instruction
            ) -> kaizen::result::Result<()> {
                use kaizen::transport::Interface;
                transport.execute(&instruction).await?;
                Ok(())
                // transport.execute(&instruction).await
            }
        }
    };

    out.into()
}
