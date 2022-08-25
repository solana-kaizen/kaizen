use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize,BorshDeserialize};
use solana_program::pubkey::Pubkey;
use solana_program::instruction;
use workflow_core::u32_try_from;
use crate::accounts::AccountData;

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

impl Into<instruction::AccountMeta> for AccountMeta {
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
}


impl From<(Pubkey, Vec<instruction::AccountMeta>, Vec<u8>)> for ExecuteReq {
    fn from((program_id, accounts, instruction_data): (Pubkey, Vec<instruction::AccountMeta>, Vec<u8>)) -> Self {
        let accounts : Vec<AccountMeta> = accounts.iter().map(|meta| meta.into()).collect();
        Self {
            program_id,
            accounts,
            instruction_data,
        }
    }
}




#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LookupReq {
    pub pubkey : Pubkey,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LookupResp {
    pub account_data : Option<AccountData>
}

u32_try_from! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[repr(u32)]
    pub enum EmulatorOps {
        Lookup = 0,
        Execute,
    }
}

impl Into<u32> for EmulatorOps {
    fn into(self) -> u32 {
        self as u32
    }
}

// enum Request {
//     Lookup(LookupReq),
//     Execute(ExecuteReq)
// }


// #[allow(non_camel_case_types)]
// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
// pub enum RespV1 {
//     v1(RespV1)
// }

// #[allow(non_camel_case_types)]
// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
// pub enum Resp {
//     v1(RespV1)
// }

// #[allow(non_camel_case_types)]
// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
// pub enum ReqV1 {
//     Lookup(LookupReq),
//     EntryPoint(EntryPointReq)
// }

// #[allow(non_camel_case_types)]
// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
// pub enum Req {
//     v1(ReqV1)
// }
