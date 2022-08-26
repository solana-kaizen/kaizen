#![allow(unused_unsafe)]
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
// use downcast::{downcast_sync, AnySync};


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


// #[derive(Derivative)]
// #[derivative(Debug)]
pub struct Transport
// where Arc<dyn EmulatorInterface> : Sized
{

    mode : Mode,

    // pub emulator : Option<Arc<Box<dyn EmulatorInterface>>>,
    pub emulator : Option<Arc<dyn EmulatorInterface>>,
    
    // #[derivative(Debug="ignore")]
    pub client_ctx : Option<(RpcClient,Keypair,Pubkey)>,

    // #[derivative(Debug="ignore")]
    // pub entrypoints : RwLock<BTreeMap<Pubkey, EntrypointDeclaration>>,

    // timeout : u64,
    // confirm_transaction_initial_timeout : u64,
    // retries : usize,
    pub config : Arc<RwLock<TransportConfig>>,

    // #[derivative(Debug="ignore")]
    pub cache : Arc<Cache>,

    pub queue : Option<TransactionQueue>,

    // #[derivative(Debug="ignore")]
    pub lookup_handler : LookupHandler<Pubkey,Arc<AccountDataReference>>,
    // #[derivative(Debug="ignore")]
}

// declare_async_rwlock!(Transport,TransportInner);


// #[wasm_bindgen]
impl Transport {

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

    // pub fn simulator(&self) -> Result<Arc<Simulator>> {
    //     match &self.simulator {
    //         Some(simulator) => Ok(simulator.clone()),
    //         None => {
    //             panic!("Transport is missing simulator")
    //             // Err(error!("transport is missing simulator"))
    //         }
    //     }
    // }

    pub async fn try_new_for_unit_tests(config : TransportConfig) -> Result<Arc<Transport>> {
        let mut transport_env_var = std::env::var("TRANSPORT").unwrap_or("simulator".into());
        if transport_env_var.starts_with("local") || transport_env_var.starts_with("native") {
            transport_env_var = "http://127.0.0.1:8899".into();
        }
        Self::try_new(transport_env_var.as_str(), config)//.await
    }

    pub fn try_new(network: &str, config : TransportConfig) -> Result<Arc<Transport>> {

        // let timeout = Duration::from_secs(60u64);
        // let confirm_transaction_initial_timeout = Duration::from_secs(5u64);
        // let retries = 2usize;



        
        let (mode, client_ctx, emulator) = // match network {

            if network == "inproc" {

            // "inproc" => {
                // let emulator: Arc<Box<dyn EmulatorInterface>> = Arc::new(Box::new(Simulator::try_new_with_store()?));
                let emulator: Arc<dyn EmulatorInterface> = Arc::new(Simulator::try_new_with_store()?);
                // let simulator = Simulator::try_new_inproc()?;
                (Mode::Inproc, None, Some(emulator))
            // } else 
            } else if regex::Regex::new(r"^rpc?://").unwrap().is_match(network) {
                // let emulator: Arc<Box<dyn EmulatorInterface>> = Arc::new(Box::new(EmulatorRpcClient::new(network)?));
                let emulator = EmulatorRpcClient::new(network)?;
                // emulator.connect(true).await?;
                let emulator: Arc<dyn EmulatorInterface> = Arc::new(emulator);
                (Mode::Emulator, None, Some(emulator))

            } else {

            // TODO: native

            // _ => {
                // let timeout_sec = 60u64; // args.timeout.parse::<u64>()?;
                // let max_retries = 2;
                let url = network; //args.rpc_endpoint;
                // let timeout = Duration::from_secs(timeout_sec);
                let commitment_config = CommitmentConfig::confirmed();
                // let confirm_transaction_initial_timeout = Duration::from_secs(5);
                // let send_config = RpcSendTransactionConfig {
                //     preflight_commitment: Some(CommitmentLevel::Confirmed),
                //     max_retries: Some(max_retries),
                //     ..Default::default()
                // };
            
                let client = RpcClient::new_with_timeouts_and_commitment(
                    url,
                    config.timeout,
                    commitment_config,
                    config.confirm_transaction_initial_timeout,
                );
            
                // Payer
                
                let home = home::home_dir().expect("unable to get home directory");
                let home = Path::new(&home);
                let payer_kp_path = home.join(".config/solana/id.json");
            
                // let payer_kp_path = 
                let payer_kp =
                    read_keypair_file(payer_kp_path).expect("Couldn't read payer keypair");
                let payer_pk = payer_kp.pubkey();
            
                println!("Payer: {}", payer_pk.to_string());

                // let payer_balance = client
                // .get_balance(&payer_pk)
                // .expect("Couldn't get payer balance");

                (Mode::Validator, Some((client, payer_kp, payer_pk)), None)
            };

        // };


        // let timeout_sec = 60u64; // args.timeout.parse::<u64>()?;
        // let max_retries = 2;
        // let url = network; //args.rpc_endpoint;
        // let timeout = Duration::from_secs(timeout_sec);
        // let commitment_config = CommitmentConfig::confirmed();
        // let confirm_transaction_initial_timeout = Duration::from_secs(5);
        // let send_config = RpcSendTransactionConfig {
        //     preflight_commitment: Some(CommitmentLevel::Confirmed),
        //     max_retries: Some(config.retries),
        //     ..Default::default()
        // };


        // let entrypoints = RwLock::new(BTreeMap::new());
        //Rc::new(RefCell::new(HashMap::new()));

        // TODO implement transaction queue support
        let queue = None;
        let cache = Arc::new(Cache::new_with_default_capacity());

        let config = Arc::new(RwLock::new(config));
        let lookup_handler = LookupHandler::new();

        // let transport = Transport::new_with_inner( TransportInner {
        let transport = Transport {
            mode,
            emulator,
            client_ctx,
            // entrypoints,
            config,
            cache,
            queue,
            lookup_handler,
        };

        let transport = Arc::new(transport);

        // let clone = transport.clone();
        unsafe { TRANSPORT = Some(transport.clone()); }
        Ok(transport)
    }

    // pub async fn with_programs(self : Arc<Self>, declarations : &[EntrypointDeclaration]) -> Result<Arc<Transport>> {
    //     {
    //         // let mut inner = self.try_inner_mut().unwrap();
    //         let mut entrypoints = self.entrypoints.write().await;
    //         for declaration in declarations {
    //             entrypoints.insert( declaration.program_id, declaration.clone());
    //         }
    //     }
    //     Ok(self)
    // }

    #[inline(always)]
    pub fn emulator<'transport>(&'transport self) -> &'transport Arc<dyn EmulatorInterface> {
        self.emulator.as_ref().expect("missing emulator interface")
    }
    // fn emulator<'transport>(&'transport self) -> &'transport Arc<Box<dyn EmulatorInterface>> {
    //     self.emulator.as_ref().expect("missing emulator interface")
    // }

    pub fn global() -> Result<Arc<Transport>> {
        let clone = unsafe { (&TRANSPORT).as_ref().unwrap().clone() };
        Ok(clone)
    }

}

impl Transport {

    pub async fn balance(&self) -> Result<u64> {

        // let simulator = { self.try_inner()?.simulator.clone() };//.unwrap().clone();//Simulator::from(&self.0.borrow().simulator);
        // match &self.emulator {
        //     Some(emulator) => {

        match self.mode { //&self.emulator {
            Mode::Inproc | Mode::Emulator => {
    
                let pubkey: Pubkey = self.get_payer_pubkey()?;
                let result = self.emulator().lookup(&pubkey).await?;
                match result {
                    Some(reference) => Ok(reference.lamports().await),
                    None => {
                        return Err(error!("[Emulator] - Transport::balance() unable to lookup account: {}", pubkey)); 
                    }
                }


                // TODO emulator
                // match emulator.lookup(&simulator.authority()).await? {
                //     Some(authority) => {
                //         Ok(authority.lamports().await)
                //     },
                //     None => {
                //         Err(error!("Transport: simulator dataset is missing authority account"))
                //     }
                // }
                // unimplemented!("transport authority")
            },
            Mode::Validator => {
                // let inner = self.try_inner()?;
                // let (client, _payer_kp, payer_pk) = if let Some(client_ctx) = &inner.client_ctx {
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

    // pub async fn execute(&self, instr : &Instruction) -> Result<()> {
    //     Ok(self.execute_with_args(&instr.program_id, &instr.accounts, &instr.data).await?)
    // }



    pub fn get_payer_pubkey(&self) -> Result<Pubkey> {
        // let simulator = { self.try_inner()?.simulator.clone() };
        // match &self.emulator {
        match self.mode {

            // Some(emulator) => {
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

    // pub async fn get_account_cache(&self, pubkey:&Pubkey) -> Result<AccountData> {


    //     self.get_account_data(pubkey).await
    // }

    // pub async fn get_account_data(&self, pubkey:&Pubkey) -> Result<AccountData> {
    // pub async fn get_account_data(&self, pubkey:&Pubkey, _range: Option<Range<usize>>) -> Result<Arc<RwLock<AccountData>>> {

    //     // let (from,to) = match &range {
    //     //     Some(range) => { (range.start as u32, range.end as u32) },
    //     //     None => { (0,0) }
    //     // };

    //     //let simulator : &Simulator = &self.0.borrow().simulator;//.into();
    //     // let simulator = { self.try_inner()?.simulator.clone() };
    // }

    // pub fn entrypoints(&self) -> Rc<RefCell<HashMap<Pubkey,Rc<ProcessInstruction>>>> {
    //     self.inner().unwrap().entrypoints.clone()
    // }

    async fn send_and_confirm_transaction_with_config(
        &self,
        client: &RpcClient,
        transaction: &Transaction,
        commitment: CommitmentConfig,
        config: RpcSendTransactionConfig,
        timeout: u64,
    ) -> Result<Signature> {
        // let args = Args::parse();
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
                    //args.sleep
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
// #[async_trait]
// impl Transport {
    fn get_authority_pubkey(&self) -> Result<Pubkey> {
        self.get_payer_pubkey()
        // let simulator = { self.try_inner()?.simulator.clone() };
        // match &self.emulator {
        //     Some(emulator) => {
        //         // TODO emulator
        //         unimplemented!("transport authority")
        //         // Ok(simulator.authority())
        //     },
        //     None => {
        //         // let inner = self.try_inner()?;
        //         let (_client, _payer_kp, payer_pk) = if let Some(client_ctx) = &self.client_ctx {
        //             client_ctx
        //         } else {
        //             return Err(error_code!(ErrorCode::MissingClient));
        //         };

        //         Ok(payer_pk.clone())
        //     }
        // }

    }

    async fn execute(self : &Arc<Self>, instruction : &Instruction) -> Result<()> { 
    // pub async fn execute_with_args(&self, program_id: &Pubkey, accounts: &[AccountMeta], data: &[u8]) -> Result<()> {
        // let simulator = { self.try_inner()?.simulator.clone() };//.unwrap().clone();//Simulator::from(&self.0.borrow().simulator);
        match &self.emulator {
            Some(emulator) => {

                // let fn_entrypoint = {
                //     match workflow_allocator::program::registry::lookup(&instruction.program_id)? {
                //         Some(entry_point) => { entry_point.entrypoint_fn },
                //         None => {
                //             log_trace!("program entrypoint not found: {:?}",instruction.program_id);
                //             return Err(error!("program entrypoint not found: {:?}",instruction.program_id).into());
                //         }
                //     }
                // };


                emulator.clone().execute(
                    instruction
                    // &instruction.program_id,
                    // &instruction.accounts,
                    // &instruction.data,
                    // // fn_entrypoint
                ).await?;

            },
            None => {
                // log_trace!("native A");
                // let inner = self.try_inner()?;
                
                log_trace!("transport: running in native mode");

                // let inner = self.try_inner()?;
                // let (client, payer_kp, payer_pk) = if let Some(client_ctx) = &inner.client_ctx {
                let (client, payer_kp, payer_pk) = if let Some(client_ctx) = &self.client_ctx {
                    client_ctx
                } else {
                    panic!("No client");
                };
                // let instruction = Instruction::new_with_bytes(*program_id,data,accounts.to_vec());

                let recent_hash = client
                    .get_latest_blockhash()
                    .expect("Couldn't get recent blockhash");

                // let native_instruction : solana_program::instruction::Instruction = (*instruction).into();
                    
                let transaction = Transaction::new_signed_with_payer(
                    &[instruction.clone()],
                    // &[instruction.into()],
                    Some(&payer_pk),
                    &[payer_kp],
                    recent_hash,
                );

                // let max_retries = 2;
                let config = &self.config.read().await.clone();
                let commitment_config = CommitmentConfig::confirmed();
                let send_config = RpcSendTransactionConfig {
                    preflight_commitment: Some(CommitmentLevel::Confirmed),
                    max_retries : Some(config.retries),
                    ..Default::default()
                };
            
                log_trace!("transprt: send_and_confirm_transaction_with_config");
                // let timeout_sec = 60u64;
                let _result = self.send_and_confirm_transaction_with_config(
                    &client,
                    &transaction,
                    commitment_config,
                    send_config,
                    config.timeout.as_secs(),
                ).await?;

                // match result {
                //     Ok(_write_hash) => {
                //         // Ok(())
                //     },
                //     Err(err) => {
                //         // log_trace!("{}", err);
                //         log_trace!("{}", err);
                //         return Err(err); //error!("{:#?}",err))
                //     }
                // };

                // .expect("Write tx error");
            }
        }

        Ok(())
    // Ok(self.execute_with_args(&instr.program_id, &instr.accounts, &instr.data).await?)
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

    // async fn lookup_remote(self : Arc<Self>, pubkey:&Pubkey) -> ManualFuture<Result<Option<Arc<RwLock<AccountData>>>>> {
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