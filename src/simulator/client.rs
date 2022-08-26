use std::sync::Arc;
use async_trait::async_trait;
use regex::Regex;
use solana_program::pubkey::Pubkey;
use solana_program::instruction::Instruction;
use workflow_allocator::accounts::AccountDataReference;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
use workflow_log::log_trace;
use workflow_rpc::asynchronous::client::RpcClient;
use workflow_rpc::asynchronous::client::result::Result as RpcResult;
use super::interface::{EmulatorInterface, ExecutionResponse};
use super::rpc::*;

#[derive(Clone)]
pub struct EmulatorRpcClient {
    rpc : Arc<RpcClient<EmulatorOps>>,
}

impl EmulatorRpcClient {
    pub fn new(url : &str) -> RpcResult<EmulatorRpcClient> {

        let re = Regex::new(r"^rpc").unwrap();
        let url = re.replace(url, "ws");
        log_trace!("Emulator RPC client url: {}", url);
        let client = EmulatorRpcClient {
            rpc: Arc::new(RpcClient::new(&url)?),
        };

        Ok(client)
    }

    pub async fn connect(&self, block : bool) -> Result<()> {
        Ok(self.rpc.connect(block).await.map_err(|e|error!("{}",e))?)
    }
}

#[async_trait]
impl EmulatorInterface for EmulatorRpcClient {
    // async fn lookup(self : Arc<Self>, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let message = LookupReq { pubkey : *pubkey };
        let resp : Result<LookupResp> = self.rpc.call(EmulatorOps::Lookup, message).await
            .map_err(|err|err.into());
        match resp {
            Ok(LookupResp { account_data }) => {
                Ok(account_data.map(|account_data|Arc::new(AccountDataReference::new(account_data))))
            },
            Err(err) => Err(err)
        }
    }
    async fn execute(
        &self,// : Arc<Self>,
        instruction : &Instruction,
    ) -> Result<ExecutionResponse> {
        // we try to re-use existing data types from Solana but 
        // these do not implement Borsh serialization
        let message = ExecuteReq { 
            program_id: instruction.program_id.clone(),
            accounts: instruction.accounts.iter().map(|account| account.into()).collect(),
            instruction_data: instruction.data.clone(),
        };
        let resp : Result<ExecutionResponse> = self.rpc.call(EmulatorOps::Execute, message).await
            .map_err(|err|err.into());
        resp
    }
}