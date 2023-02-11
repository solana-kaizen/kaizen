//!
//! Helper structures for PDA generation
//!

use crate::error::*;
use crate::result::Result;
#[allow(unused_imports)]
use solana_program::instruction::AccountMeta;
use workflow_log::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AddressDomain {
    /// No domain - `[program_id,seed]` only
    None,
    /// auto-select identity if available, otherwise authority
    Default,
    /// explicitly select authority: `[program_id,authority,seed]`
    Authority,
    /// explicitly select identity: `[program_id,identity,seed]`
    Identity,
    // /// custom
    // Custom(&'seed [u8])
}

impl AddressDomain {
    #[cfg(not(target_os = "solana"))]
    pub fn get_seed(
        &self,
        authority: Option<&AccountMeta>,
        identity: Option<&AccountMeta>,
    ) -> Result<Vec<u8>> {
        let seed_prefix = match self {
            AddressDomain::None => vec![],
            AddressDomain::Default => match identity.or(authority) {
                Some(meta) => meta.pubkey.to_bytes().to_vec(),
                None => {
                    return Err(error!(
                        "Missing identity or authority for default address domain"
                    ))
                }
            },
            AddressDomain::Authority => authority
                .ok_or(error!("Missing authority for address domain"))?
                .pubkey
                .to_bytes()
                .to_vec(),
            AddressDomain::Identity => identity
                .ok_or(error!("Missing identity for address domain"))?
                .pubkey
                .to_bytes()
                .to_vec(),
            // AddressDomain::Custom(seed) => seed.to_vec()
        };

        Ok(seed_prefix)
    }
}

#[derive(Debug)]
pub struct ProgramAddressData<'data> {
    pub seed: &'data [u8],
}

impl<'data> ProgramAddressData<'data> {
    pub fn from_bytes(seed: &'data [u8]) -> Self {
        ProgramAddressData { seed }
    }

    pub fn try_from(data: &'data [u8]) -> Result<(ProgramAddressData<'data>, usize)> {
        if data.is_empty() {
            log_trace!("Error: ProgramAddressData is receiving data len {} (you are not supplying valid template accounts?)",data.len());
            return Err(error_code!(ErrorCode::PADDataBufferSizeAvailable));
        }
        let data_len = data[0] as usize;
        let bytes_used = data_len + 1;
        // log_trace!("| pda: data_len: {} full data len: {}", data_len, data.len());
        let seed = &data[1..bytes_used];
        let pad = ProgramAddressData { seed };
        // log_trace!("| pda: consuming {} bytes", bytes_used);
        Ok((pad, bytes_used))
    }
}
