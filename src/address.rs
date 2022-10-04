use workflow_log::*;
use crate::error::*;
use crate::result::Result;

pub struct ProgramAddressData<'data> {
    pub seed: &'data [u8],
}

impl<'data> ProgramAddressData<'data> {
    pub fn from_bytes(seed: &'data [u8]) -> Self {
        ProgramAddressData { seed }
    }
    
    pub fn try_from(data : &'data [u8]) -> Result<(ProgramAddressData<'data>,usize)> {
        if data.len() < 1 {
            log_trace!("Error: ProgramAddressData is receiving data len {} (you are not supplying valid template accounts?)",data.len());
            return Err(error_code!(ErrorCode::PADDataBufferSizeAvailable));
        }
        let data_len = data[0] as usize;
        let bytes_used = data_len + 1;
        // log_trace!("| pda: data_len: {} full data len: {}", data_len, data.len());
        let seed = &data[1..bytes_used];
        let pad = ProgramAddressData {
            seed
        };
        // log_trace!("| pda: consuming {} bytes", bytes_used);
        Ok((pad, bytes_used))
    }
}

