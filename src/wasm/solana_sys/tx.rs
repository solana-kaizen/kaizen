use wasm_bindgen::prelude::*;
//use crate::result::Result;
pub use workflow_wasm::options::OptionsTrait;
use crate::wasm::solana_sys::TransactionInstruction;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=solanaWeb3, js_name = Transaction)]
    #[derive(Debug, Clone)]
    pub type Transaction;

    #[wasm_bindgen(constructor, js_namespace=["solanaWeb3"])]
    /// Construct an empty Transaction
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Transaction.html)
    ///
    pub fn new() -> Transaction;

    #[wasm_bindgen(setter, method, js_namespace=["solanaWeb3"], js_name="feePayer")]
    /// Set the transaction fee payer
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Transaction.html#feePayer)
    ///
    pub fn set_fee_payer(this: &Transaction, fee_payer_pubkey: JsValue);

    #[wasm_bindgen(setter, method, js_namespace=["solanaWeb3"], js_name="recentBlockhash")]
    /// A recent transaction id. Must be populated by the caller
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Transaction.html#recentBlockhash)
    ///
    pub fn set_recent_block_hash(this: &Transaction, recent_blockhash: JsValue);

    #[wasm_bindgen(method, js_namespace=["solanaWeb3"], js_name="add")]
    /// Add one instruction to this Transaction
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Transaction.html#add)
    ///
    pub fn add(this: &Transaction, instruction: TransactionInstruction);

    #[wasm_bindgen(method, js_namespace=["solanaWeb3"], js_name="serialize")]
    /// Serialize the Transaction in the wire format.
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/Transaction.html#serialize)
    ///
    pub fn serialize(this: &Transaction)->JsValue;
    
}

impl Transaction{
    
}
