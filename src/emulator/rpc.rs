use crate::accounts::AccountDataStore;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::instruction;
use solana_program::pubkey::Pubkey;
use workflow_core::u32_try_from;

#[derive(
    Debug, Default, Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&instruction::AccountMeta> for AccountMeta {
    fn from(meta: &instruction::AccountMeta) -> Self {
        Self {
            pubkey: meta.pubkey,
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        }
    }
}

impl From<&AccountMeta> for instruction::AccountMeta {
    fn from(meta: &AccountMeta) -> Self {
        instruction::AccountMeta {
            pubkey: meta.pubkey,
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        }
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ExecuteReq {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub instruction_data: Vec<u8>,
    pub authority: Pubkey,
}

impl From<(&Pubkey, instruction::Instruction)> for ExecuteReq {
    fn from((authority, instruction): (&Pubkey, instruction::Instruction)) -> Self {
        Self {
            program_id: instruction.program_id,
            accounts: instruction
                .accounts
                .iter()
                .map(|account| account.into())
                .collect(),
            instruction_data: instruction.data.clone(),
            authority: *authority,
        }
    }
}

impl From<ExecuteReq> for (Pubkey, instruction::Instruction) {
    fn from(req: ExecuteReq) -> Self {
        (
            req.authority,
            instruction::Instruction {
                program_id: req.program_id,
                accounts: req.accounts.iter().map(|account| account.into()).collect(),
                data: req.instruction_data.clone(),
            },
        )
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LookupReq {
    pub pubkey: Pubkey,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LookupResp {
    pub account_data_store: Option<AccountDataStore>,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct FundReq {
    pub key: Pubkey,
    pub owner: Pubkey,
    pub lamports: u64,
}

u32_try_from! {
    #[derive(Clone, Debug, Hash, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
    // #[repr(u32)]
    pub enum EmulatorOps {
        Lookup = 0,
        Execute,
        Fund,
        List,
        Configure,
    }
}

impl From<EmulatorOps> for u32 {
    fn from(ops: EmulatorOps) -> u32 {
        ops as u32
    }
}
