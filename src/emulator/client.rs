use std::sync::Arc;
use async_trait::async_trait;
use regex::Regex;
use solana_program::pubkey::Pubkey;
use solana_program::instruction::Instruction;
use workflow_allocator::accounts::AccountDataReference;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
use workflow_core::trigger::Listener;
use workflow_log::{log_trace, log_error};
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

    pub async fn connect(&self, block : bool) -> Result<Option<Listener>> {
        Ok(self.rpc.connect(block).await.map_err(|e|error!("{}",e))?)
    }

    pub fn connect_as_task(self : &Arc<Self>) -> Result<()> {
        let self_ = self.clone();
        workflow_core::task::spawn(async move {
            self_.rpc.connect(false).await.ok();
        });
        Ok(())
    }
}

#[async_trait]
impl EmulatorInterface for EmulatorRpcClient {
    
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let message = LookupReq { pubkey : *pubkey };
        let resp : Result<LookupResp> = self.rpc.call(EmulatorOps::Lookup, message).await
            .map_err(|err|err.into());
        match resp {
            Ok(LookupResp { account_data_store }) => {
                Ok(account_data_store.map(|account_data_store|Arc::new(AccountDataReference::from(&account_data_store))))
            },
            Err(err) => {
                Err(err)
            }
        }
    }

    async fn execute(
        &self,
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
            // log_trace!("response: {:?}", resp);
        if let Ok(resp) = &resp {
            for line in resp.logs.iter() {
                log_trace!("| {}",line);
            }
        }

        resp
    }

    async fn fund(
        &self,
        key : &Pubkey,
        owner : &Pubkey,
        lamports : u64
    ) -> Result<()> {
        let message = FundReq {
            key : key.clone(),
            owner : owner.clone(),
            lamports
        };
        let resp : Result<()> = self.rpc.call(EmulatorOps::Fund, message).await
            .map_err(|err|err.into());
            resp
    }
}