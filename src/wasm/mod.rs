use wasm_bindgen::prelude::*;
use workflow_panic_hook::{set_once, Type, show_logs as show_wasm_logs};
#[wasm_bindgen]
pub fn start_console_logger(){
    set_once(Type::Console);
}
#[wasm_bindgen]
pub fn start_popup_logger(){
    set_once(Type::Popup);
}
#[wasm_bindgen]
pub fn show_logs(){
    show_wasm_logs();
}

pub fn bind(workflow: &JsValue, solana: &JsValue, mods: &JsValue) -> std::result::Result<(), JsValue> {
    let global = js_sys::Object::new();
    js_sys::Reflect::set(&js_sys::global(), &"$workflow".into(), &global)?;
    js_sys::Reflect::set(&global,&"workflow".into(),&workflow)?;
    js_sys::Reflect::set(&global,&"solana".into(),&solana)?;
    js_sys::Reflect::set(&global,&"modules".into(),&mods)?;
    Ok(())
}

pub fn global() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&js_sys::global(), &"$workflow".into())?)
}

pub fn workflow() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&global()?, &"workflow".into())?)
}

pub fn solana() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&global()?, &"solana".into())?)
}

fn mods() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&global()?, &"modules".into())?)
}

pub fn wallet_ready_state() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&mods()?, &"WalletReadyState".into())?)
}

pub fn adapters() -> std::result::Result<Vec<JsValue>, JsValue> {
    let mut list = Vec::new();
    let names = vec!["PhantomWalletAdapter", "SolflareWalletAdapter"];
    let mods = mods()?;
    for name in names{
        match js_sys::Reflect::get(&mods, &name.into()){
            Ok(adapter_ctr)=>{
                //log_trace!("adapter_ctr: {:?}", adapter_ctr);
                match js_sys::Reflect::construct(&adapter_ctr.into(), &js_sys::Array::new_with_length(0)){
                    Ok(adapter)=>{
                        list.push(adapter);
                    }
                    Err(_e)=>{
                        //
                    }
                }
            }
            Err(_e)=>{
                //
            }
        }
    }

    Ok(list)
}
