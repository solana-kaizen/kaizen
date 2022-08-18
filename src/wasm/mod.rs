// pub mod timers;
// pub mod utils;

use wasm_bindgen::prelude::*;

pub fn bind(workflow: &JsValue, solana: &JsValue) -> std::result::Result<(), JsValue> {
    let global = js_sys::Object::new();
    js_sys::Reflect::set(&js_sys::global(), &"$workflow".into(), &global)?;
    js_sys::Reflect::set(&global,&"workflow".into(),&workflow)?;
    js_sys::Reflect::set(&global,&"solana".into(),&solana)?;
    Ok(())
}

pub fn global() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&js_sys::global(), &"$workflow".into())?)
}

pub fn workflow() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&global()?, &"workflow".into())?)
}

pub fn solana() -> std::result::Result<JsValue,JsValue> {
    Ok(js_sys::Reflect::get(&workflow()?, &"solana".into())?)
}
