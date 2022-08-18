use workflow_log::*;
use crate::error::*;
use crate::result::Result;

pub struct ProgramAddressData<'instr> {
    pub seed: &'instr [u8],
}

impl<'instr> ProgramAddressData<'instr> {
    pub fn try_from(data : &'instr [u8]) -> Result<(ProgramAddressData<'instr>,usize)> {
        if data.len() < 1 {
            log_trace!("Error: ProgramAddressDataReference is receiving data len {} (you are not supplying valid template accounts?)",data.len());
            return Err(error_code!(ErrorCode::PADDataBufferSizeAvailable));
        }
        let data_len = data[0] as usize;
        let bytes_used = data_len + 1;
        log_trace!("| pda: data_len: {} full data len: {}", data_len, data.len());
        let seed = &data[1..bytes_used];
        let pad = ProgramAddressData {
            seed
        };
        log_trace!("| pda: consuming {} bytes", bytes_used);
        Ok((pad, bytes_used))
    }
}

