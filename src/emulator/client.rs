use crate::accounts::AccountDescriptorList;
use async_trait::async_trait;
use kaizen::accounts::AccountDataReference;
use kaizen::error::*;
use kaizen::result::Result;
use regex::Regex;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use workflow_core::trigger::Listener;
use workflow_log::*;
use workflow_rpc::asynchronous::client::result::Result as RpcResult;
use workflow_rpc::asynchronous::client::RpcClient;

use super::interface::{EmulatorConfig, EmulatorInterface, ExecutionResponse};
use super::rpc::*;

#[derive(Clone)]
pub struct EmulatorRpcClient {
    rpc: Arc<RpcClient<EmulatorOps>>,
}

impl EmulatorRpcClient {
    pub fn new(url: &str) -> RpcResult<EmulatorRpcClient> {
        let re = Regex::new(r"^rpc").unwrap();
        let url = re.replace(url, "ws");
        log_trace!("Emulator RPC client url: {}", url);
        let client = EmulatorRpcClient {
            rpc: Arc::new(RpcClient::new(&url)?),
        };

        Ok(client)
    }

    pub async fn connect(&self, block: bool) -> Result<Option<Listener>> {
        Ok(self.rpc.connect(block).await.map_err(|e| error!("{}", e))?)
    }

    pub fn connect_as_task(self: &Arc<Self>) -> Result<()> {
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
        let message = LookupReq { pubkey: *pubkey };
        let resp: Result<LookupResp> = self
            .rpc
            .call(EmulatorOps::Lookup, message)
            .await
            .map_err(|err| err.into());
        match resp {
            Ok(LookupResp { account_data_store }) => {
                Ok(account_data_store.map(|account_data_store| {
                    Arc::new(AccountDataReference::from(&account_data_store))
                }))
            }
            Err(err) => Err(err),
        }
    }

    async fn execute(
        &self,
        authority: &Pubkey,
        instruction: &Instruction,
    ) -> Result<ExecutionResponse> {
        // we try to re-use existing data types from Solana but
        // these do not implement Borsh serialization
        let message = ExecuteReq {
            program_id: instruction.program_id.clone(),
            accounts: instruction
                .accounts
                .iter()
                .map(|account| account.into())
                .collect(),
            instruction_data: instruction.data.clone(),
            authority: authority.clone(),
        };
        let resp: Result<ExecutionResponse> = self
            .rpc
            .call(EmulatorOps::Execute, message)
            .await
            .map_err(|err| err.into());
        if let Ok(resp) = &resp {
            // TODO setup verbose flag somewhere in configuration
            for line in resp.logs.iter() {
                for l in line.split("\n") {
                    log_trace!("| {}", l);
                }
            }
        }

        resp
    }

    async fn fund(&self, key: &Pubkey, owner: &Pubkey, lamports: u64) -> Result<()> {
        let message = FundReq {
            key: key.clone(),
            owner: owner.clone(),
            lamports,
        };
        let resp: Result<()> = self
            .rpc
            .call(EmulatorOps::Fund, message)
            .await
            .map_err(|err| err.into());
        resp
    }

    async fn list(&self) -> Result<AccountDescriptorList> {
        let resp: Result<AccountDescriptorList> = self
            .rpc
            .call(EmulatorOps::List, ())
            .await
            .map_err(|err| err.into());
        resp
    }

    async fn configure(&self, config: EmulatorConfig) -> Result<()> {
        let resp: Result<()> = self
            .rpc
            .call(EmulatorOps::Configure, config)
            .await
            .map_err(|err| err.into());
        resp
    }
}
