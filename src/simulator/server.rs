use std::sync::Arc;

use async_trait::async_trait;
use borsh::{BorshSerialize,BorshDeserialize};
use solana_program::instruction::Instruction;
use workflow_rpc::asynchronous::server::RpcHandlerBorsh;
use workflow_rpc::asynchronous::server::RpcResponseError;
use workflow_rpc::asynchronous::result::RpcResult;
use workflow_allocator::simulator::interface::EmulatorInterface;
use workflow_allocator::simulator::rpc::*;
use workflow_allocator::store::FileStore;
use workflow_allocator::cache::Cache;
use workflow_allocator::result::Result;

use super::{interface::ExecutionResponse, Emulator};

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
    #[allow(dead_code)]
    pub fn try_new() -> Result<Server> {
        let cache = Arc::new(Cache::new_with_capacity(DEFAULT_CAPACITY));
        let store = Arc::new(FileStore::try_new_with_cache(cache)?);
        let emulator = Arc::new(Emulator::new(store.clone()));

        let server = Server {
            emulator
        };

        Ok(server)
    }
}

#[async_trait]
impl RpcHandlerBorsh<EmulatorOps> for Server
{
    async fn handle_request(self : Arc<Self>, op : EmulatorOps, data : &[u8]) -> RpcResult {
        match op {
            EmulatorOps::Lookup => {
                let req = LookupReq::try_from_slice(data)?;
                let reference = self.emulator.clone().lookup(&req.pubkey).await?;
                let resp = match reference {
                    Some(reference) => {
                        let account_data = reference.account_data.read().await;
                        LookupResp { account_data : Some(account_data.clone()) };
                    },
                    None => {
                        LookupResp { account_data : None };
                    } 
                };
                Ok(resp.try_to_vec()?)
            },
            EmulatorOps::Execute => {

                let req = ExecuteReq::try_from_slice(data)?;
                // entry_point

                let instruction : Instruction = req.into();

                self.emulator.execute(&instruction).await?;

                let resp = ExecutionResponse::new(None,None);

                // let vec = resp.try_to_vec()?;
                // log_trace!("**** TEST VEC: {:?}",vec);
                // let xx = ExecutionResponse::try_from_slice(&vec)?;

                Ok(resp.try_to_vec()?)
            }

        }
    }
}
