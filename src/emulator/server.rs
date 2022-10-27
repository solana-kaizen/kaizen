use std::sync::Arc;
use async_trait::async_trait;
use borsh::{BorshSerialize,BorshDeserialize};
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use workflow_rpc::asynchronous::server::RpcHandler;
use workflow_rpc::asynchronous::server::RpcResponseError;
use workflow_rpc::asynchronous::result::RpcResult;
use workflow_allocator::emulator::interface::EmulatorInterface;
use workflow_allocator::emulator::rpc::*;
use workflow_allocator::store::FileStore;
use workflow_allocator::cache::Cache;
use workflow_allocator::result::Result;
use crate::accounts::AccountDataStore;

use super::Emulator;
use super::interface::EmulatorConfig;
use workflow_log::*;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error("error")]
    SomeError,
}

impl From<Error> for RpcResponseError {
    fn from(err: Error) -> Self {
        RpcResponseError::Text(err.to_string())
    }
}

const DEFAULT_CAPACITY : u64 = 1024u64 * 1024u64 * 256u64; // 256 megabytes

pub struct Server {
    pub emulator : Arc<Emulator>,
}

impl Server {
    // #[allow(dead_code)]
    pub fn try_new() -> Result<Server> {
        let cache = Arc::new(Cache::new_with_capacity(DEFAULT_CAPACITY));
        let store = Arc::new(FileStore::try_new_with_cache(cache)?);
        let emulator = Arc::new(Emulator::new(store.clone()));
        // let log_sink = downcast_::<LogSink>(emulator.clone())?;

        let server = Server {
            emulator
        };

        Ok(server)
    }

    pub async fn init(&self) -> Result<()> {
        self.emulator.init().await
    }
}

#[async_trait]
// impl RpcHandlerBorsh<EmulatorOps> for Server
impl RpcHandler<EmulatorOps> for Server
{
    async fn handle_request(self : Arc<Self>, op : EmulatorOps, data : &[u8]) -> RpcResult {
        match op {
            EmulatorOps::Lookup => {
                let req = LookupReq::try_from_slice(data)?;
                let reference = self.emulator.clone().lookup(&req.pubkey).await?;
                let resp = match reference {
                    Some(reference) => {
                        let account_data_store = AccountDataStore::from(&*reference.account_data.lock()?);
                        // let account_data = AccountData::from(&account_data_store);
                        // rpc-failure
                        LookupResp { account_data_store : Some(account_data_store) }
                    },
                    None => {
                        LookupResp { account_data_store : None }
                    } 
                };
                Ok(resp.try_to_vec()?)
            },
            EmulatorOps::Execute => {

                // std::thread::sleep(std::time::Duration::from_millis(5000));

                let req = ExecuteReq::try_from_slice(data)?;
                // let authority = req.authority.clone();
                let (authority,instruction) : (Pubkey,Instruction) = req.into();
                let resp = self.emulator.execute(&authority,&instruction).await?;
                // let resp = ExecutionResponse::new(None,None);
                Ok(resp.try_to_vec()?)
            },
            EmulatorOps::Fund => {
                let req = FundReq::try_from_slice(data)?;
                self.emulator.fund(&req.key,&req.owner,req.lamports).await?;
                log_trace!("fundinng done...");
                Ok(().try_to_vec()?)
            },
            EmulatorOps::List => {
                let resp = self.emulator.list().await?;
                Ok(resp.try_to_vec()?)
            },
            EmulatorOps::Configure => {
                let _config = EmulatorConfig::try_from_slice(data)?;
                // let resp = self.emulator.list().await?;
                Ok(().try_to_vec()?)
            },
        }
    }
}

