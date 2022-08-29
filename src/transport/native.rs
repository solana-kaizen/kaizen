// #![allow(unused_unsafe)]
use std::*;
use async_std::sync::RwLock;
use std::time::Duration;
use std::time::SystemTime;
use std::sync::Arc;
use async_std::path::Path;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::IntoAccountInfo;
use crate::emulator::Simulator;
use crate::accounts::*;
use crate::error::*;
use crate::result::Result;
use crate::accounts::AccountData;
use crate::emulator::client::EmulatorRpcClient;
use crate::emulator::interface::EmulatorInterface;
use crate::transport::queue::TransactionQueue;
use workflow_log::log_trace;
use workflow_allocator::cache::Cache;
use solana_program::instruction::Instruction;
use crate::transport::TransportConfig;
use crate::transport::Mode;
use crate::transport::lookup::{LookupHandler,RequestType};

use solana_client::{
    rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig,
};

use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    signature::{read_keypair_file, Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

static mut TRANSPORT : Option<Arc<Transport>> = None;

pub struct Transport
{
    mode : Mode,
    pub emulator : Option<Arc<dyn EmulatorInterface>>,
    pub client_ctx : Option<(RpcClient,Keypair,Pubkey)>,
    pub config : Arc<RwLock<TransportConfig>>,
    pub cache : Arc<Cache>,
    pub queue : Option<TransactionQueue>,
    pub lookup_handler : LookupHandler<Pubkey,Arc<AccountDataReference>>,
}

impl Transport {

    pub async fn root(&self) -> Pubkey {
        self.config.read().await.root
    }

    pub async fn connect(&self, block : bool) -> Result<()> {
        match self.mode {
            Mode::Emulator => {
                let emulator = self.emulator
                    .clone()
                    .unwrap()
                    .downcast_arc::<EmulatorRpcClient>()
                    .expect("Unable to downcast to EmulatorRpcClient");

                emulator.connect(block).await?;

                Ok(())
            },
            _ => { Ok(()) }
        }
    }

    pub async fn try_new_for_unit_tests(config : TransportConfig) -> Result<Arc<Transport>> {
        let mut transport_env_var = std::env::var("TRANSPORT").unwrap_or("simulator".into());
        if transport_env_var.starts_with("local") || transport_env_var.starts_with("native") {
            transport_env_var = "http://127.0.0.1:8899".into();
        }
        Self::try_new(transport_env_var.as_str(), config)//.await
    }

    pub fn try_new(network: &str, config : TransportConfig) -> Result<Arc<Transport>> {

        let (mode, client_ctx, emulator) = // match network {

            if network == "inproc" {
                let emulator: Arc<dyn EmulatorInterface> = Arc::new(Simulator::try_new_with_store()?);
                (Mode::Inproc, None, Some(emulator))
            } else if regex::Regex::new(r"^rpc?://").unwrap().is_match(network) {
                let emulator = EmulatorRpcClient::new(network)?;
                let emulator: Arc<dyn EmulatorInterface> = Arc::new(emulator);
                (Mode::Emulator, None, Some(emulator))
            } else {

                let url = network;
                let commitment_config = CommitmentConfig::confirmed();
                let client = RpcClient::new_with_timeouts_and_commitment(
                    url,
                    config.timeout,
                    commitment_config,
                    config.confirm_transaction_initial_timeout,
                );
            
                // authority is the local pk
                let home = home::home_dir().expect("unable to get home directory");
                let home = Path::new(&home);
                let payer_kp_path = home.join(".config/solana/id.json");
            
                let payer_kp =
                    read_keypair_file(payer_kp_path).expect("Couldn't read authority keypair");
                let payer_pk = payer_kp.pubkey();
            
                log_trace!("User authority: {}", payer_pk.to_string());

                (Mode::Validator, Some((client, payer_kp, payer_pk)), None)
            };

        // TODO implement transaction queue support
        let queue = None;
        let cache = Arc::new(Cache::new_with_default_capacity());

        let config = Arc::new(RwLock::new(config));
        let lookup_handler = LookupHandler::new();

        let transport = Transport {
            mode,
            emulator,
            client_ctx,
            config,
            cache,
            queue,
            lookup_handler,
        };

        let transport = Arc::new(transport);
        unsafe { TRANSPORT = Some(transport.clone()); }
        
        Ok(transport)

    }

    #[inline(always)]
    pub fn emulator<'transport>(&'transport self) -> &'transport Arc<dyn EmulatorInterface> {
        self.emulator.as_ref().expect("missing emulator interface")
    }

    pub fn global() -> Result<Arc<Transport>> {
        let clone = unsafe { (&TRANSPORT).as_ref().unwrap().clone() };
        Ok(clone)
    }

    pub async fn balance(&self) -> Result<u64> {

        match self.mode {
            Mode::Inproc | Mode::Emulator => {
    
                let pubkey: Pubkey = self.get_authority_pubkey_impl()?;
                let result = self.emulator().lookup(&pubkey).await?;
                match result {
                    Some(reference) => Ok(reference.lamports().await),
                    None => {
                        return Err(error!("[Emulator] - Transport::balance() unable to lookup account: {}", pubkey)); 
                    }
                }
            },
            Mode::Validator => {
                let (client, _payer_kp, payer_pk) = if let Some(client_ctx) = &self.client_ctx {
                    client_ctx
                } else {
                    panic!("Transport: Missing RPC Client");
                };

                let payer_balance = client
                    .get_balance(&payer_pk)
                    .expect("Could not get payer balance");

                Ok(payer_balance)
            }
        }
    }

    pub fn get_authority_pubkey_impl(&self) -> Result<Pubkey> {
        match self.mode {
            Mode::Inproc => {

                let simulator = self.emulator
                    .clone()
                    .unwrap()
                    .downcast_arc::<Simulator>()
                    .expect("Unable to downcast to Simulator");

                Ok(simulator.authority())
                
            },
            
            Mode::Emulator => {
                let home = home::home_dir().expect("unable to get home directory");
                let home = Path::new(&home);
                let payer_kp_path = home.join(".config/solana/id.json");
                let payer_kp =
                    read_keypair_file(payer_kp_path).expect("Couldn't read payer keypair");
                let payer_pk = payer_kp.pubkey();
                Ok(payer_pk)
            },
            Mode::Validator => {
                let (_client, _payer_kp, payer_pk) = if let Some(client_ctx) = &self.client_ctx {
                    client_ctx
                } else {
                    return Err(error_code!(ErrorCode::MissingClient));
                };

                Ok(payer_pk.clone())
            }
        }
    }

    async fn send_and_confirm_transaction_with_config(
        &self,
        client: &RpcClient,
        transaction: &Transaction,
        commitment: CommitmentConfig,
        config: RpcSendTransactionConfig,
        timeout: u64,
    ) -> Result<Signature> {
        let mut hash;
    
        'outer: loop {
            hash = client.send_transaction_with_config(transaction, config)?;
    
            let start_time = SystemTime::now();
    
            loop {
                if let Ok(resp) = client.confirm_transaction_with_commitment(&hash, commitment) {
                    if resp.value {
                        break 'outer;
                    }
                }
    
                let current_time = SystemTime::now();
                if current_time.duration_since(start_time).unwrap().as_secs() > timeout {
                    break;
                }
    
                async_std::task::sleep(Duration::from_millis(
                    1_000,
                )).await;
            }
        }
    
        Ok(hash)
    }
    

    async fn lookup_remote_impl(self : Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        match &self.emulator {
            Some(emulator) => {
                Ok(emulator.clone().lookup(pubkey).await?)
            },
            None => {
                let (client, _payer_kp, _payer_pk) = if let Some(client_ctx) = &self.client_ctx {
                    client_ctx
                } else {
                    panic!("No client");
                };

                let mut account = client.get_account(pubkey)?;
                let account_info = (pubkey, &mut account).into_account_info();
                let account_data = AccountData::clone_from_account_info(&account_info);
                Ok(Some(Arc::new(AccountDataReference::new(account_data))))
            }
        }
    }


}

#[async_trait(?Send)]
impl super::Interface for Transport {
    fn get_authority_pubkey(&self) -> Result<Pubkey> {
        self.get_authority_pubkey_impl()
    }

    async fn execute(self : &Arc<Self>, instruction : &Instruction) -> Result<()> { 
        match &self.emulator {
            Some(emulator) => {
                emulator.clone().execute(
                    instruction
                ).await?;
            },
            None => {
                
                log_trace!("transport: running in native mode");

                let (client, payer_kp, payer_pk) = if let Some(client_ctx) = &self.client_ctx {
                    client_ctx
                } else {
                    panic!("No client");
                };

                let recent_hash = client
                    .get_latest_blockhash()
                    .expect("Couldn't get recent blockhash");

                let transaction = Transaction::new_signed_with_payer(
                    &[instruction.clone()],
                    Some(&payer_pk),
                    &[payer_kp],
                    recent_hash,
                );

                let config = &self.config.read().await.clone();
                let commitment_config = CommitmentConfig::confirmed();
                let send_config = RpcSendTransactionConfig {
                    preflight_commitment: Some(CommitmentLevel::Confirmed),
                    max_retries : Some(config.retries),
                    ..Default::default()
                };
            
                log_trace!("transprt: send_and_confirm_transaction_with_config");
                let _result = self.send_and_confirm_transaction_with_config(
                    &client,
                    &transaction,
                    commitment_config,
                    send_config,
                    config.timeout.as_secs(),
                ).await?;
            }
        }

        Ok(())
    }
 
    async fn lookup(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let account_data = self.clone().lookup_local(pubkey).await?;
        match account_data {
            Some(account_data) => Ok(Some(account_data.clone())),
            None => {
                Ok(self.lookup_remote(pubkey).await?)
            }
        }
    }

    async fn lookup_local(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        Ok(self.cache.lookup(pubkey).await?)
    }

    async fn lookup_remote(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        let request_type = self.clone().lookup_handler.queue(pubkey);
        match request_type {
            RequestType::New(future) => {
                let response = self.clone().lookup_remote_impl(pubkey).await;
                self.clone().lookup_handler.complete(pubkey, response).await;
                future.await
            },
            RequestType::Pending(future) => {
                future.await
            }
        }

    }

}