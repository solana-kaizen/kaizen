//!
//! Kaizen WASM32 browser initialization API
//! 
use wasm_bindgen::prelude::*;
use workflow_wasm::init::init_workflow;
pub use workflow_wasm::init::{global, modules, workflow};
pub use workflow_wasm::panic::*;

pub fn init_kaizen(
    workflow: &JsValue,
    solana: &JsValue,
    mods: &JsValue,
) -> std::result::Result<(), JsValue> {
    init_workflow(workflow, mods)?;

    let g = global()?;
    js_sys::Reflect::set(&g, &"solana".into(), solana)?;
    Ok(())
}

pub fn solana() -> std::result::Result<JsValue, JsValue> {
    js_sys::Reflect::get(&global()?, &"solana".into())
}

pub fn wallet_ready_state() -> std::result::Result<JsValue, JsValue> {
    js_sys::Reflect::get(&modules()?, &"WalletReadyState".into())
}

pub fn adapters() -> std::result::Result<Vec<JsValue>, JsValue> {
    let mut list = Vec::new();
    let names = vec!["PhantomWalletAdapter", "SolflareWalletAdapter"];
    let mods = modules()?;
    for name in names {
        match js_sys::Reflect::get(&mods, &name.into()) {
            Ok(adapter_ctr) => {
                //log_trace!("adapter_ctr: {:?}", adapter_ctr);
                match js_sys::Reflect::construct(
                    &adapter_ctr.into(),
                    &js_sys::Array::new_with_length(0),
                ) {
                    Ok(adapter) => {
                        list.push(adapter);
                    }
                    Err(_e) => {
                        //
                    }
                }
            }
            Err(_e) => {
                //
            }
        }
    }

    Ok(list)
}
