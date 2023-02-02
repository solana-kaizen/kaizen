use crate::accounts::AccountDataStore;
use async_trait::async_trait;
use borsh::{BorshDeserialize, BorshSerialize};
use kaizen::cache::Cache;
use kaizen::emulator::interface::EmulatorInterface;
use kaizen::emulator::rpc::*;
use kaizen::result::Result;
use kaizen::store::FileStore;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use workflow_rpc::result::ServerResult;
use workflow_rpc::server::RpcHandler;
use workflow_rpc::server::ServerError;

use super::interface::EmulatorConfig;
use super::Emulator;
use workflow_log::*;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error("error")]
    SomeError,
}

impl From<Error> for ServerError {
    fn from(err: Error) -> Self {
        ServerError::Text(err.to_string())
    }
}

const DEFAULT_CAPACITY: u64 = 1024u64 * 1024u64 * 256u64; // 256 megabytes

pub struct Server {
    pub emulator: Arc<Emulator>,
}

impl Server {
    // #[allow(dead_code)]
    pub fn try_new() -> Result<Server> {
        let cache = Arc::new(Cache::new_with_capacity(DEFAULT_CAPACITY));
        let store = Arc::new(FileStore::try_new_with_cache(cache)?);
        let emulator = Arc::new(Emulator::new(store));

        let server = Server { emulator };

        Ok(server)
    }

    pub async fn init(&self) -> Result<()> {
        self.emulator.init().await
    }
}

#[async_trait]
// impl RpcHandlerBorsh<EmulatorOps> for Server
// impl RpcHandler<EmulatorOps> for Server {
impl RpcHandler for Server {
    async fn handle_request(self: Arc<Self>, op: EmulatorOps, data: &[u8]) -> RpcResult {
        match op {
            EmulatorOps::Lookup => {
                let req = LookupReq::try_from_slice(data)?;
                let reference = self.emulator.clone().lookup(&req.pubkey).await?;
                let resp = match reference {
                    Some(reference) => {
                        let account_data_store =
                            AccountDataStore::from(&*reference.account_data.lock()?);
                        LookupResp {
                            account_data_store: Some(account_data_store),
                        }
                    }
                    None => LookupResp {
                        account_data_store: None,
                    },
                };
                Ok(resp.try_to_vec()?)
            }
            EmulatorOps::Execute => {
                let req = ExecuteReq::try_from_slice(data)?;
                let (authority, instruction): (Pubkey, Instruction) = req.into();
                let resp = self.emulator.execute(&authority, &instruction).await?;
                Ok(resp.try_to_vec()?)
            }
            EmulatorOps::Fund => {
                let req = FundReq::try_from_slice(data)?;
                self.emulator
                    .fund(&req.key, &req.owner, req.lamports)
                    .await?;
                log_trace!("fundinng done...");
                Ok(().try_to_vec()?)
            }
            EmulatorOps::List => {
                let resp = self.emulator.list().await?;
                Ok(resp.try_to_vec()?)
            }
            EmulatorOps::Configure => {
                let _config = EmulatorConfig::try_from_slice(data)?;
                Ok(().try_to_vec()?)
            }
        }
    }
}
