use std::rc::Rc;
use std::sync::Arc;
use ahash::AHashSet;
use async_std::sync::RwLock;
use derivative::Derivative;
use borsh::{BorshSerialize,BorshDeserialize};
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::slot_history::AccountInfo;
use solana_program::account_info::IntoAccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::entrypoint::ProcessInstruction;
use workflow_log::*;
use workflow_allocator::realloc::account_info_realloc;
use workflow_allocator::context::SimulationHandlerFn;
use workflow_allocator::utils::generate_random_pubkey;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
// use workflow_allocator::console::style;
use workflow_allocator::address::ProgramAddressData;
use workflow_allocator::context::Context;
use workflow_allocator::accounts::*;
use workflow_allocator::builder::{
    InstructionBuilder,
    InstructionBuilderConfig
};
use workflow_allocator::store::MemoryStore;
use crate::accounts::AccountData;
use crate::container::try_get_container_type;

// #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
// enum RpcReq {
//     Lookup(Pubkey),
//     Execute(Instruction),
// }


// #[derive(Debug)]
// pub struct KeyStore {
//     pub store: Vec<Pubkey>,
// }

// impl KeyStore {
//     pub fn new(len: usize) -> KeyStore {
//         let store = (0..len).map(|_| generate_random_pubkey()).collect();
//         KeyStore { store }
//     }
// }





// #[derive(Derivative)]
// #[derivative(Debug)]
// #[wasm_bindgen]

/* 
#[cfg(not(target_arch = "bpf"))]
pub mod client {
    use super::AccountData;
    use js_sys::*;
    use wasm_bindgen::prelude::*;

    //  This is not a bingen function!  AccountData is not exposed to bindgen
    pub fn account_data_to_jsv(
        account_data: &AccountData,
    ) -> std::result::Result<JsValue, JsValue> {
        let resp = js_sys::Object::new();
        unsafe {
            js_sys::Reflect::set(
                &resp,
                &"data".into(),
                &JsValue::from(Uint8Array::view(&account_data.data)),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"owner".into(),
                &JsValue::from(Uint8Array::view(&account_data.owner.to_bytes())),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"lamports".into(),
                &JsValue::from_f64(account_data.lamports as f64),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"rentEpoch".into(),
                &JsValue::from_f64(account_data.rent_epoch as f64),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"executable".into(),
                &JsValue::from_bool(account_data.executable),
            )?;
        }
        Ok(resp.into())
    }

}
*/

