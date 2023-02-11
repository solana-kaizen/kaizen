use solana_program::pubkey::Pubkey;
use wasm_bindgen::prelude::*;
use workflow_wasm::options::OptionsTrait;
use js_sys::Object;

use crate::result::Result;
use crate::wasm::solana_sys::{
    pubkey_to_jsvalue
};



#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=solanaWeb3, js_name = Connection)]
    #[derive(Debug, Clone)]
    pub type Connection;

    #[wasm_bindgen(constructor, js_namespace=["solanaWeb3"])]
    /// Create Connection
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html)
    ///
    pub fn new(endpoint: String) -> Connection;

    #[wasm_bindgen(constructor, js_namespace=["solanaWeb3"])]
    /// Create Connection
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html)
    ///
    pub fn new_with_commitment(endpoint: String, commitment: String) -> Connection;

    #[wasm_bindgen(method, catch, js_name="getLatestBlockhash")]
    /// Fetch the latest blockhash from the cluster
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#getLatestBlockhash)
    ///
    pub async fn get_latest_block_hash_impl(this: &Connection)->Result<JsValue>;

    #[wasm_bindgen(method, catch, js_name="getLatestBlockhash")]
    /// Fetch the latest blockhash from the cluster
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#getLatestBlockhash)
    ///
    pub async fn get_latest_block_hash_with_commitment(this: &Connection, commitment: String)->Result<JsValue>;

    #[wasm_bindgen(method, catch, js_name="sendRawTransaction")]
    /// Send a transaction that has already been signed and serialized into the wire format
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#sendRawTransaction)
    ///
    pub async fn send_raw_transaction(this: &Connection, tx:JsValue)->Result<JsValue>;

    #[wasm_bindgen(method, catch, js_name="sendRawTransaction")]
    /// Send a transaction that has already been signed and serialized into the wire format
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#sendRawTransaction)
    ///
    pub async fn send_raw_transaction_with_options_impl(this: &Connection, tx:JsValue, options: JsValue)->Result<JsValue>;

    #[wasm_bindgen(method, catch, js_name="getAccountInfo")]
    /// Fetch all the account info for the specified public key
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#getAccountInfo)
    ///
    pub async fn get_account_info_impl(this: &Connection, public_key: JsValue)->Result<JsValue>;

    #[wasm_bindgen(method, catch, js_name="getAccountInfo")]
    /// Fetch all the account info for the specified public key
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#getAccountInfo)
    ///
    pub async fn get_account_info_with_options_impl(this: &Connection, public_key: JsValue, options: JsValue)->Result<JsValue>;
   

    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type LatestBlockhashInfo;

    #[wasm_bindgen(getter, method, js_name="blockhash")]
    /// get blockhash
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#getLatestBlockhash)
    ///
    pub fn block_hash(this: &LatestBlockhashInfo)->JsValue;

    #[wasm_bindgen(getter, method, js_name="lastValidBlockHeight")]
    /// get lastValidBlockHeight
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Connection.html#getLatestBlockhash)
    ///
    pub fn last_valid_block_height(this: &LatestBlockhashInfo)->JsValue;

    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SendRawTxOptions;
}

impl Connection{
    pub async fn get_latest_block_hash(&self)->Result<LatestBlockhashInfo>{
        Ok(self.get_latest_block_hash_impl().await?.into())
    }

    pub async fn get_account_info(&self, pubkey: &Pubkey)->Result<JsValue>{
        self.get_account_info_impl(pubkey_to_jsvalue(pubkey)?).await
    }

    pub async fn get_account_info_with_options(&self, pubkey: &Pubkey, options: JsValue)->Result<JsValue>{
        self.get_account_info_with_options_impl(pubkey_to_jsvalue(pubkey)?, options).await
    }

    pub async fn send_raw_transaction_with_options(&self, tx:JsValue,  options: SendRawTxOptions)->Result<JsValue>{
        self.send_raw_transaction_with_options_impl(tx, options.into()).await
    }
}

impl OptionsTrait for SendRawTxOptions{}

impl SendRawTxOptions{
    /// set skipPreflight
    pub fn skip_preflight(self, skip_preflight:bool)->Self{
        self.set("skipPreflight", JsValue::from(skip_preflight))
    }
}



