use wasm_bindgen::prelude::*;
use crate::result::Result;
pub use workflow_wasm::options::OptionsTrait;
use solana_program::instruction::Instruction;
use crate::wasm::solana_sys::TransactionInstructionConfig;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=solanaWeb3, js_name = TransactionInstruction)]
    #[derive(Debug, Clone)]
    pub type TransactionInstruction;

    #[wasm_bindgen(constructor, js_namespace=["solanaWeb3"])]
    /// Create TransactionInstruction
    ///
    /// ⧉ [Solana Documentation](https://solana-labs.github.io/solana-web3.js/classes/TransactionInstruction.html)
    ///
    pub fn new(options: &TransactionInstructionConfig) -> TransactionInstruction;
}

impl TryFrom<&Instruction> for TransactionInstruction{
    type Error = crate::error::Error;
    fn try_from(instruction: &Instruction) -> Result<Self> {
        let mut accounts_list = vec![];

        for account in &instruction.accounts {
            accounts_list.push(account.try_into()?);
        }

        let cfg = TransactionInstructionConfig::new()
            .data(&instruction.data)
            .keys(accounts_list)
            .program_id(&instruction.program_id)?;

        Ok(TransactionInstruction::new(&cfg))
    }
}
