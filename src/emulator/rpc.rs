use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize,BorshDeserialize};
use solana_program::pubkey::Pubkey;
use solana_program::instruction;
use workflow_core::u32_try_from;
use crate::accounts::AccountDataStore;

#[derive(Debug, Default, PartialEq, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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

impl Into<instruction::AccountMeta> for &AccountMeta {
    fn into(self) -> instruction::AccountMeta {
        instruction::AccountMeta {
            pubkey: self.pubkey,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
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


impl From<(&Pubkey,instruction::Instruction)> for ExecuteReq {
    fn from((authority,instruction) : (&Pubkey,instruction::Instruction)) -> Self {
        // let accounts : Vec<AccountMeta> = accounts.iter().map(|meta| meta.into()).collect();
        
        Self {
            program_id: instruction.program_id.clone(),
            accounts: instruction.accounts.iter().map(|account| account.into()).collect(),
            instruction_data: instruction.data.clone(),
            authority: authority.clone(),
            // program_id,
            // accounts,
            // instruction_data,
        }
    }
}

impl Into<(Pubkey,instruction::Instruction)> for ExecuteReq {
    fn into(self) -> (Pubkey,instruction::Instruction) {
        (
            self.authority,
            instruction::Instruction {
                program_id : self.program_id.clone(),
                accounts : self.accounts.iter().map(|account| account.into()).collect(),
                data : self.instruction_data.clone(),
            }
        )
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LookupReq {
    pub pubkey : Pubkey,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LookupResp {
    pub account_data_store : Option<AccountDataStore>
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct FundReq {
    pub key : Pubkey,
    pub owner : Pubkey,
    pub lamports : u64,
}

u32_try_from! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[repr(u32)]
    pub enum EmulatorOps {
        Lookup = 0,
        Execute,
        Fund,
        List,
        Configure,
    }
}

impl Into<u32> for EmulatorOps {
    fn into(self) -> u32 {
        self as u32
    }
}

