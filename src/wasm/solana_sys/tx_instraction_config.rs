use wasm_bindgen::prelude::*;
use crate::prelude::Pubkey;
use crate::result::Result;
pub use workflow_wasm::options::OptionsTrait;
use crate::wasm::solana_sys::*;

use js_sys::{
    Array, Object, Uint8Array
};


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type TransactionInstructionConfig;
}


impl OptionsTrait for TransactionInstructionConfig{}


impl TransactionInstructionConfig{
    /// Set keys
    pub fn keys(self, keys: Vec<AccountMeta>)->Self{
        let list = Array::new();
        for key in keys{
            list.push(&key.into());
        }
        self.set("keys", JsValue::from(list))
    }

    /// Set programId
    pub fn program_id(self, program_id: &Pubkey)->Result<Self>{
        Ok(self.set("programId", pubkey_to_jsvalue(program_id)?))
    }

    /// Set data
    pub fn data(self, data: &[u8])->Self{
        self.set("data", Uint8Array::from(data).into())
    }
}
