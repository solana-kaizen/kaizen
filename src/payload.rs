//!
//! Solana OS Program Instruction Payload header.
//!
//! This header defines the data layout for [`Context`](crate::context::Context) deserialization.
//!

use kaizen::result::Result;

pub const PAYLOAD_HAS_IDENTITY_ACCOUNT: u16 = 0x0001;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Payload {
    pub version: u8,
    pub flags: u16,

    pub system_accounts_len: u8,
    pub token_accounts_len: u8,
    pub index_accounts_len: u8,
    pub collection_accounts_len: u8,
    pub generic_template_accounts_len: u8,
    pub collection_template_accounts_len: u8,

    pub collection_data_offset: u16,
    pub instruction_data_offset: u16,

    pub interface_id: u16,
    pub handler_id: u16,
}

impl Payload {
    pub fn version() -> u8 {
        1u8
    }

    pub fn total_accounts(&self) -> usize {
        self.system_accounts_len as usize
            + self.token_accounts_len as usize
            + self.index_accounts_len as usize
            + self.collection_accounts_len as usize
            + self.generic_template_accounts_len as usize
            + self.collection_template_accounts_len as usize
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let payload = workflow_core::utils::struct_as_slice_u8(self);
        payload.to_vec()
    }

    pub fn try_from(data: &[u8]) -> Result<&Payload> {
        let payload = unsafe { std::mem::transmute(&data[0]) };
        Ok(payload)
    }

    pub fn new<T: Into<u16>>(interface_id: usize, program_instruction: T) -> Self {
        Payload {
            version: 1,
            flags: 0,
            system_accounts_len: 0,
            token_accounts_len: 0,
            index_accounts_len: 0,
            collection_accounts_len: 0,
            generic_template_accounts_len: 0,
            collection_template_accounts_len: 0,
            collection_data_offset: 0,
            instruction_data_offset: 0,
            interface_id: interface_id as u16,
            handler_id: program_instruction.into(),
        }
    }
}
