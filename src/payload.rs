use workflow_allocator::result::Result;
// use workflow_allocator::error::*;

pub const PAYLOAD_HAS_SYSTEM_ACCOUNT : u16      = 0x0001;
pub const PAYLOAD_HAS_IDENTITY_ACCOUNT : u16    = 0x0002;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Payload {
    pub version : u8,
    pub token_accounts_len : u8,      
    pub index_accounts_len : u8,      
    pub template_accounts_len : u8,   
    // pub handler_accounts_len : u8,    
    pub flags : u16,

//    pub template_address_data_len : u8,
    pub instruction_data_offset : u16,

    pub interface_id : u16,
    pub handler_id : u16,
}

impl Payload {

    pub fn version() -> u8 { 1u8 }

    pub fn total_accounts(&self) -> usize {
        self.token_accounts_len as usize
        + self.index_accounts_len as usize
        + self.template_accounts_len as usize
    }

    pub fn to_vec(&self) -> Vec<u8> {

        let payload = workflow_core::utils::struct_as_slice_u8(self);
        payload.to_vec()
    }

    pub fn try_from<'a>(data: &'a [u8]) -> Result<&'a Payload> {
        let payload = unsafe { std::mem::transmute(&data[0]) };
        Ok(payload)
    }

    pub fn new<T : Into<u16>>(interface_id: usize, program_instruction: T) -> Self {
        Payload {
            version : 1,
            token_accounts_len : 0,
            index_accounts_len : 0,
            template_accounts_len: 0,
            flags : 0,
            instruction_data_offset : 0,
            interface_id : interface_id as u16,
            handler_id : program_instruction.into(),
        }
    }

}

