use std::*;
use async_std::sync::RwLock;
use workflow_log::log_error;
use std::time::Duration;
use std::time::SystemTime;
use std::sync::{Mutex, Arc};
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
use crate::transport::TransportMode;
use crate::transport::lookup::{LookupHandler,RequestType};
use crate::transport::{reflector,Reflector};
use crate::wallet::*;

use solana_client::{
    rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig,
};

use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    signature::{read_keypair_file, Signature},
    signer::Signer,
    transaction::Transaction,
};

static mut TRANSPORT : Option<Arc<Transport>> = None;

pub struct Transport
{
    mode : TransportMode,
    pub emulator : Option<Arc<dyn EmulatorInterface>>,
    pub rpc_client : Option<RpcClient>,
    pub wallet : Arc<dyn foreign::WalletInterface>,
    pub config : Arc<RwLock<TransportConfig>>,
    pub cache : Arc<Cache>,
    pub queue : Arc<TransactionQueue>,
    pub lookup_handler : LookupHandler<Pubkey,Arc<AccountDataReference>>,
    pub custom_authority: Arc<Mutex<Option<Pubkey>>>,
    pub reflector : Reflector,
}

impl Transport {

    pub fn set_custom_authority(&self, key:Option<Pubkey>)-> Result<()> {
        (*self.custom_authority.lock()?) = key;
        Ok(())
    }

    pub fn mode(&self) -> TransportMode {
        self.mode.clone()
    }

    pub fn reflector(&self) -> Reflector {
        self.reflector.clone()
    }

    pub async fn root(&self) -> Pubkey {
        self.config.read().await.root
    }

    pub async fn connect(&self, block : bool) -> Result<()> {
        match self.mode {
            TransportMode::Emulator => {
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

    pub async fn try_new_for_unit_tests(program_id : Pubkey, authority : Option<Pubkey>, config : TransportConfig) -> Result<Arc<Transport>> {
        let mut network = std::env::var("TRANSPORT").unwrap_or("inproc".into());
        if network.starts_with("local") {
            network = "http://127.0.0.1:8899".into();
        }

        if network == "inproc" {
            let simulator = Simulator::try_new_for_testing()?.with_mock_accounts(program_id, authority).await?;
            let emulator: Arc<dyn EmulatorInterface> = Arc::new(simulator);
            Transport::try_new_with_args(TransportMode::Inproc, None, Some(emulator), config).await
        } else if regex::Regex::new(r"^rpc?://").unwrap().is_match(&network) {
            let emulator = EmulatorRpcClient::new(&network)?;
            let emulator: Arc<dyn EmulatorInterface> = Arc::new(emulator);
            Transport::try_new_with_args(TransportMode::Emulator, None, Some(emulator), config).await
        } else {
            panic!("Unabel to create transport for network '{}'", network);
        }

    }

    pub async fn try_new(network: &str, config : TransportConfig) -> Result<Arc<Transport>> {

        if regex::Regex::new(r"^rpc?://").unwrap().is_match(network) {
            let emulator = EmulatorRpcClient::new(network)?;
            let emulator: Arc<dyn EmulatorInterface> = Arc::new(emulator);
            Transport::try_new_with_args(TransportMode::Emulator, None, Some(emulator), config).await
        } else {

            let url = network;
            let commitment_config = CommitmentConfig::confirmed();
            let client = RpcClient::new_with_timeouts_and_commitment(
                url,
                config.timeout,
                commitment_config,
                config.confirm_transaction_initial_timeout,
            );
        
            Transport::try_new_with_args(TransportMode::Validator, Some(client), None, config).await
        }
    }

    pub async fn try_new_with_args(
        mode : TransportMode,
        rpc_client : Option<RpcClient>,
        emulator : Option<Arc<dyn EmulatorInterface>>,
        config : TransportConfig,
    ) -> Result<Arc<Transport>> {

        let wallet = Arc::new(foreign::native::Wallet::try_new()?);

        // TODO implement transaction queue support
        let queue = Arc::new(TransactionQueue::new());
        let cache = Arc::new(Cache::new_with_default_capacity());
        let config = Arc::new(RwLock::new(config));
        let lookup_handler = LookupHandler::new();
        let reflector = Reflector::new();

        let transport = Transport {
            mode,
            emulator,
            wallet,
            rpc_client,
            config,
            cache,
            queue,
            lookup_handler,
            reflector,
            custom_authority:Arc::new(Mutex::new(None))
        };

        let transport = Arc::new(transport);
        unsafe { TRANSPORT = Some(transport.clone()); }
        
        Ok(transport)

    }

    #[inline(always)]
    pub fn emulator<'transport>(&'transport self) -> Option<&'transport Arc<dyn EmulatorInterface>> {
        self.emulator.as_ref()
    }

    pub fn simulator<'transport>(&'transport self) -> Arc<Simulator> {
        let simulator = self.emulator
            .clone()
            .expect("Transport::simulator() - emulator interface not present")
            .downcast_arc::<Simulator>()
            .expect("Transport::simulator() - unable to downcast to Simulator");

        simulator
    }

    pub fn global() -> Result<Arc<Transport>> {
        let clone = unsafe { (&TRANSPORT).as_ref().unwrap().clone() };
        Ok(clone)
    }

    #[inline(always)]
    pub fn wallet(&self) -> Arc<dyn foreign::WalletInterface> {
        self.wallet.clone()
    }

    pub async fn balance(&self) -> Result<u64> {

        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {
    
                let pubkey: Pubkey = self.get_authority_pubkey_impl()?;
                match self.emulator()
                    .ok_or("Missing emulator interface")?
                    .lookup(&pubkey).await? {
                    Some(reference) => Ok(reference.lamports()?),
                    None => {
                        return Err(error!("[Emulator] - Transport::balance() unable to lookup account: {}", pubkey)); 
                    }
                }
            },
            TransportMode::Validator => {
                let rpc_client = self.rpc_client.as_ref().expect("Transport: Missing RPC client");
                let payer_balance = rpc_client
                    .get_balance(&self.wallet.pubkey()?)
                    .expect("Could not get payer balance");

                Ok(payer_balance)
            }
        }
    }

    pub fn get_authority_pubkey_impl(&self) -> Result<Pubkey> {
        match self.mode {
            TransportMode::Inproc => {

                let simulator = self.emulator
                    .clone()
                    .unwrap()
                    .downcast_arc::<Simulator>()
                    .expect("Unable to downcast to Simulator");

                Ok(simulator.authority())
                
            },
            
            TransportMode::Emulator => {
                if let Some(key) = self.custom_authority.lock()?.as_ref(){
                    return Ok(key.clone());
                }
                let home = home::home_dir().expect("unable to get home directory");
                let home = Path::new(&home);
                let payer_kp_path = home.join(".config/solana/id.json");
                let payer_kp =
                    read_keypair_file(payer_kp_path).expect("Couldn't read payer keypair");
                let payer_pk = payer_kp.pubkey();
                Ok(payer_pk)
            },
            TransportMode::Validator => {

                Ok(self.wallet.pubkey()?.clone())
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

    async fn lookup_remote_impl(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        self.cache.purge(Some(pubkey))?;

        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {

                let reference = self.emulator()
                    .ok_or("Missing emulator interface")?
                    .lookup(pubkey).await?;
                match reference {
                    Some(reference) => {
                        self.cache.store(&reference)?;
                        Ok(Some(reference))
                    },
                    None => Ok(None)
                }
            },
            TransportMode::Validator => {

                let rpc_client = self.rpc_client.as_ref().expect("Missing RPC Client");
                let commitment_config = CommitmentConfig::processed();
                let response = rpc_client.get_account_with_commitment(pubkey, commitment_config)?;
                match response.value {
                    Some(mut account) => {
                        let account_info = (pubkey, &mut account).into_account_info();
                        let account_data = AccountData::clone_from_account_info(&account_info);
                        let reference = Arc::new(AccountDataReference::new(account_data));
                        self.cache.store(&reference)?;
                        Ok(Some(reference))
                    },
                    None => {
                        return Ok(None);
                    }
                }
            }
        }
    }

}

#[async_trait]
impl super::Interface for Transport {
    fn get_authority_pubkey(&self) -> Result<Pubkey> {
        self.get_authority_pubkey_impl()
    }

    fn purge(&self, pubkey: Option<&Pubkey>) -> Result<()> {
        Ok(self.cache.purge(pubkey)?)
    }

    async fn post(&self, tx: Arc<super::transaction::Transaction>) -> Result<()> { 
        self.queue.enqueue(tx).await?;
        Ok(())
    }

    async fn post_multiple(&self, txs : Vec<Arc<super::transaction::Transaction>>) -> Result<()> { 
        self.queue.enqueue_multiple(txs).await
    }

    async fn execute(&self, instruction : &Instruction) -> Result<()> { 
        match &self.emulator {
            Some(emulator) => {

                

                let authority = self.get_authority_pubkey()?;
                let resp = emulator.clone().execute(
                    &authority,
                    instruction
                ).await?;

                self.reflector.reflect(reflector::Event::EmulatorLogs(resp.logs));
                self.reflector.reflect(reflector::Event::WalletRefresh("SOL".into(),authority.clone()));
                match self.balance().await {
                    Ok(balance) => {
                        self.reflector.reflect(reflector::Event::WalletBalance("SOL".into(),authority.clone(),balance));
                    },
                    Err(err) => {
                        log_error!("Unable to update wallet balance: {}", err);
                    }
                }
            },
            None => {
                
                log_trace!("transport: running in native mode");
                let rpc_client = self.rpc_client.as_ref().expect("Missing RPC Client");

                let wallet = self.wallet.clone().downcast_arc::<foreign::native::Wallet>()
                    .expect("Unable to downcast native wallt");

                let recent_hash = rpc_client
                    .get_latest_blockhash()
                    .expect("Couldn't get recent blockhash");

                let transaction = Transaction::new_signed_with_payer(
                    &[instruction.clone()],
                    Some(&wallet.keypair().pubkey()),
                    &[wallet.keypair()],
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
                    rpc_client,
                    &transaction,
                    commitment_config,
                    send_config,
                    config.timeout.as_secs(),
                ).await?;
            }
        }

        Ok(())
    }
 
    async fn lookup(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let account_data = self.clone().lookup_local(pubkey).await?;
        match account_data {
            Some(account_data) => Ok(Some(account_data.clone())),
            None => {
                Ok(self.lookup_remote(pubkey).await?)
            }
        }
    }

    async fn lookup_local(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        Ok(self.cache.lookup(pubkey)?)
    }

    async fn lookup_remote(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let lookup_handler = &self.clone().lookup_handler;
        let request_type = lookup_handler.queue(pubkey).await;
        let result = match request_type {
            RequestType::New(receiver) => {
                self.reflector.reflect(reflector::Event::PendingLookups(lookup_handler.pending()));
 
                let response = self.lookup_remote_impl(pubkey).await;
                lookup_handler.complete(pubkey, response).await;
                receiver.recv().await?
            },
            RequestType::Pending(receiver) => {
                receiver.recv().await?
            }
        };
        self.reflector.reflect(reflector::Event::PendingLookups(lookup_handler.pending()));
        result
    }

}