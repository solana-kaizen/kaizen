#![allow(unused_unsafe)]
// use std::cell::RefCell;
use std::*;
// use std::sync::Mutex;
// use std::collections::HashMap;
use async_std::sync::RwLock;
use wasm_bindgen::prelude::*;
use solana_program::pubkey::Pubkey;
// use solana_program::entrypoint::ProcessInstruction;
use crate::accounts::AccountData;
use crate::emulator::Simulator;
use crate::emulator::client::EmulatorRpcClient;
use crate::emulator::interface::EmulatorInterface;
// use crate::wasm::*;
use workflow_wasm::utils;
use crate::transport::queue::TransactionQueue;
use js_sys::*;
use wasm_bindgen_futures::JsFuture;
// use derivative::Derivative;
use solana_program::instruction::Instruction;
use crate::result::Result;
// use crate::error::*;
use crate::error;
use workflow_log::*;
use async_trait::async_trait;
use std::sync::{Mutex, Arc};
// use async_std::sync::RwLock;
use workflow_allocator::cache::Cache;
use std::convert::From;
use crate::transport::{Transaction, TransportConfig};
use crate::transport::lookup::{LookupHandler,RequestType};
use crate::transport::{reflector, Reflector};
use wasm_bindgen_futures::future_to_promise;
use crate::accounts::AccountDataReference;
use super::TransportMode;
use crate::wallet::*;

// use crate::wallet::wasm;

// pub mod router {
//     use super::*;

//     thread_local!{
//         pub static PIEP : RefCell<Vec<(String, Pubkey, Arc<ProcessInstruction>)>> = RefCell::new(Vec::new());
//     }

//     pub fn register_entry_point(ident:&str,id:Pubkey,piep: ProcessInstruction) {
//         PIEP.with(|list| {
//             list.borrow_mut().push((ident.into(),id,Arc::new(piep)));
//         });
//     }
// }


static mut TRANSPORT : Option<Arc<Transport>> = None;

mod wasm_bridge {
    use super::*;

    #[wasm_bindgen]
    pub struct Transport {
        #[wasm_bindgen(skip)]
        pub transport : Arc<super::Transport>
    }
    
    #[wasm_bindgen]
    impl Transport {
        #[wasm_bindgen(constructor)]
        pub fn new(network: String) -> std::result::Result<Transport, JsValue> {
            log_trace!("Creating Transport (WASM bridge)");
            let transport = super::Transport::try_new(network.as_str(), super::TransportConfig::default())
                .map_err(|e| JsValue::from(e))?;
            Ok(Transport { transport })
        }
        #[wasm_bindgen(js_name="withWallet")]
        pub fn with_wallet(&mut self, wallet: JsValue) -> std::result::Result<(), JsValue> {
            self.transport.with_wallet(wallet)?;
            Ok(())
        }

        #[wasm_bindgen(js_name="getAuthorityPubkey")]
        pub fn get_authority_pubkey(&self) -> Result<Pubkey> {
            self.transport.get_authority_pubkey_impl()
        }

        #[wasm_bindgen(js_name="balance")]
        pub fn balance(&self) -> Promise {
            let transport = self.transport.clone();
            future_to_promise(async move{
                let balance = transport.balance().await?;
                Ok(JsValue::from(balance))
            })
        }

    
/* 
        pub fn with_programs(&self, pkg: JsValue) -> Result<()>  {

            let mut fn_names = Vec::new();
            let keys = unsafe { js_sys::Reflect::own_keys(&pkg)? };
            let keys_vec = keys.to_vec();
            for idx in 0..keys_vec.len() {
                let name: String = keys_vec[idx].as_string().unwrap_or("".into());
                if name.starts_with("piep_register") {
                    // log_trace!("init_bindings() - found one: {}", name);
                    fn_names.push(keys_vec[idx].clone());
                }
            }
    
            if fn_names.len() == 0 {
                panic!("workflow_allocator::Transport::init_bindings(): no wasm bindings found!");
            }
    
            for fn_name in fn_names.iter() {
                let fn_jsv = unsafe { js_sys::Reflect::get(&pkg,fn_name)? };
                let args = Array::new();
                let _ret_jsv = unsafe { js_sys::Reflect::apply(&fn_jsv.into(),&pkg,&args.into())? };
            }
    
            let mut entrypoints = self.entrypoints.try_write().ok_or(error!("unable to acquire write lock on transport entrypoints"))?;
    
            router::PIEP.with(|list_ref| {
                let list = list_ref.borrow();
                for (ident,id,piep) in list.iter() {
                    log_trace!("binding program {} ▷ {}",id.to_string(),ident);
                    entrypoints.insert(id.clone(),piep.clone());
                }
            });
    
            Ok(())
        }
*/    

    }
}

// pub struct Transport

// #[derive(Derivative)]
// #[derivative(Debug)] // , Clone)]
pub struct Transport {
    // pub simulator : Option<Arc<Simulator>>,

    mode : TransportMode,

    pub emulator : Option<Arc<dyn EmulatorInterface>>,

    pub wallet : Arc<dyn foreign::WalletInterface>,

    // #[wasm_bindgen(skip)]
    pub queue : Arc<TransactionQueue>,
    // #[wasm_bindgen(skip)]
    cache : Cache, //Arc<Store>,
    
    pub config : Arc<RwLock<TransportConfig>>,

    pub custom_authority: Arc<Mutex<Option<Pubkey>>>,

    connection : JsValue,
    // wallet : JsValue,
    // #[wasm_bindgen(skip)]
    // #[derivative(Debug="ignore")]
    // pub entrypoints : Arc<RwLock<HashMap<Pubkey,Arc<ProcessInstruction>>>>,
    // #[derivative(Debug="ignore")]
    pub lookup_handler : LookupHandler<Pubkey,Arc<AccountDataReference>>,

    pub reflector : Reflector,

}

unsafe impl Send for Transport {}
unsafe impl Sync for Transport {}

impl Transport {


    pub fn workflow() -> std::result::Result<JsValue,JsValue> {
        Ok(js_sys::Reflect::get(&js_sys::global(), &"$workflow".into())?)
    }

    pub fn solana() -> std::result::Result<JsValue,JsValue> {
        Ok(js_sys::Reflect::get(&Self::workflow()?, &"solana".into())?)
    }

    pub fn mode(&self) -> TransportMode {
        self.mode.clone()
    }

    pub fn reflector(&self) -> Reflector {
        self.reflector.clone()
    }

    pub fn connection(&self) -> std::result::Result<JsValue,JsValue> {
        Ok(self.connection.clone())
        // Ok(js_sys::Reflect::get(&Self::solana()?, &"connection".into())?)
    }

    pub fn with_wallet(&self, wallet: JsValue) -> std::result::Result<JsValue, JsValue> {
        js_sys::Reflect::set(&Self::workflow()?, &"wallet".into(), &wallet)?;
        Ok(JsValue::from(true))
    }

    pub fn wallet_adapter(&self) -> std::result::Result<JsValue, JsValue> {
        let wallet = js_sys::Reflect::get(&Self::workflow()?, &"wallet".into())?;
        if wallet == JsValue::UNDEFINED{
            log_trace!("wallet adapter is missing");
            return Err(error!("WalletAdapterIsMissing, use `transport.with_wallet(walletAdapter);`").into());
        }
        Ok(wallet.clone())
    }

    pub fn set_custom_authority(&self, key:Option<Pubkey>)-> Result<()> {
        (*self.custom_authority.lock()?) = key;
        Ok(())
    }

    pub fn public_key_ctor() -> std::result::Result<JsValue,JsValue> {
        Ok(js_sys::Reflect::get(&Self::solana()?,&JsValue::from("PublicKey"))?)
    }

    pub async fn try_new_for_unit_tests(
        _program_id : Pubkey, 
        _authority : Option<Pubkey>,
        config : TransportConfig) -> Result<Arc<Transport>> {
        // let mut transport_env_var = std::env::var("TRANSPORT").unwrap_or("simulator".into());
        // if transport_env_var.starts_with("local") || transport_env_var.starts_with("native") {
        //     transport_env_var = "http://127.0.0.1:8899".into();
        // }
        // Self::try_new(transport_env_var.as_str(), config)//.await
        Self::try_new("inproc", config)//.await
    }

    // pub fn simulator(&self) -> Result<Arc<Simulator>> {
    //     match &self.simulator {
    //         Some(simulator) => Ok(simulator.clone()),
    //         None => Err(error!("transport is missing simulator"))
    //     }
    // }

    #[inline(always)]
    pub fn new_wallet(&self) -> Arc<dyn foreign::WalletInterface> {
        self.wallet.clone()
    }

    pub fn is_emulator(&self)->Result<bool>{
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => Ok(true),
            _=>Ok(false)
        }
    }

    pub async fn balance(&self) -> Result<u64> {

        // let simulator = { self.try_inner()?.simulator.clone() };//.unwrap().clone();//Simulator::from(&self.0.borrow().simulator);
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {
                let pubkey: Pubkey = self.get_authority_pubkey_impl()?;
                let result = self.emulator().lookup(&pubkey).await?;
                match result {
                    Some(reference) => Ok(reference.lamports()?),
                    None => {
                        return Err(error!("[Emulator] - WASM::Transport::balance() unable to lookup account: {}", pubkey)); 
                    }
                }

                // Ok(0u64)
                // match simulator.store.lookup(&simulator.authority()).await? {
                //     Some(authority) => {
                //         Ok(authority.lamports().await)
                //     },
                //     None => {
                //         Err(error!("WASM::Transport: simulator dataset is missing authority account"))
                //     }
                // }
            },
            TransportMode::Validator => {
                let pubkey: Pubkey = self.get_authority_pubkey_impl()?;
                let result = self.lookup_remote_impl(&pubkey).await?;
                match result{
                    Some(reference)=>{
                        Ok(reference.lamports()?)
                        // match Arc::try_unwrap(data_arc){
                        //     Ok(data_rwlock)=>{
                        //         let account_data = data_rwlock.read().await;
                        //         log_trace!("account_data: {:#?}", account_data);
                        //         return Ok(account_data.lamports);
                        //     },
                        //     Err(err)=>{
                        //         return Err(error!("WASM::Transport::balance() account_data read error {:?}", err)); 
                        //     }
                        // };
                    },
                    None=>{
                        return Err(error!("WASM::Transport::balance() unable to lookup account: {}", pubkey)); 
                    }
                }
                
            }
        }
    }

    pub fn get_authority_pubkey_impl(&self) -> Result<Pubkey> {

        match self.mode {

        // }
        // match &self.emulator {
            // Some(simulator) => {
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
                let wallet_adapter = &self.wallet_adapter()?;
                let public_key = unsafe{js_sys::Reflect::get(wallet_adapter, &JsValue::from("publicKey"))?};
                let pubkey = Pubkey::new(&utils::try_get_vec_from_bn(&public_key)?);
                Ok(pubkey)

            },
    
                // Ok(simulator.authority())
            //     unimplemented!("TODO")
            // },
            TransportMode::Validator => {
                let wallet_adapter = &self.wallet_adapter()?;
                let public_key = unsafe{js_sys::Reflect::get(wallet_adapter, &JsValue::from("publicKey"))?};
                let pubkey = Pubkey::new(&utils::try_get_vec_from_bn(&public_key)?);
                Ok(pubkey)
            }
        }
    }

    pub async fn root(&self) -> Pubkey {
        self.config.read().await.root
    }

    // #![feature(local_key_cell_methods)]
    pub fn try_new(network: &str, config : TransportConfig) -> Result<Arc<Transport>> {

        // let transport = ;
        log_trace!("Creating transport (rust) for network {}", network);
        if let Some(_) = unsafe { (&TRANSPORT).as_ref() } {
            return Err(error!("Transport already initialized"));
            // log_trace!("Transport already initialized");
            // panic!("Transport already initialized");
        }

        // log_trace!("loading workflow global");
        // let workflow = js_sys::Reflect::get(&js_sys::global(), &"$workflow".into())?;
        // log_trace!("loading solana global");
        // let solana = js_sys::Reflect::get(&workflow, &"solana".into())?;
        // log_trace!("initializing network setup, solana: {:?}", solana);
        let solana = Self::solana()?;
        let (mode, connection, emulator) = 
            if network == "inproc" {
                let emulator: Arc<dyn EmulatorInterface> = Arc::new(Simulator::try_new_with_store()?);
                (TransportMode::Inproc, JsValue::NULL, Some(emulator))
            } else if regex::Regex::new(r"^rpcs?://").unwrap().is_match(network) {
                // let emulator = EmulatorRpcClient::new(network)?;
                let emulator = Arc::new(EmulatorRpcClient::new(network)?);
                emulator.connect_as_task()?;
                // workflow_core::task::spawn(async move {
                //     emulator.connect(false).await;
                // });
                let emulator: Arc<dyn EmulatorInterface> = emulator;
                (TransportMode::Emulator, JsValue::NULL, Some(emulator))
            } else if network == "mainnet-beta" || network == "testnet" || network == "devnet" {
                let cluster_api_url_fn = js_sys::Reflect::get(&solana,&JsValue::from("clusterApiUrl"))?;
                let args = Array::new_with_length(1);
                args.set(0, JsValue::from(network));
                let url = js_sys::Reflect::apply(&cluster_api_url_fn.into(),&JsValue::NULL,&args.into())?;
                log_trace!("{network}: {:?}", url);
        
                let args = Array::new_with_length(1);
                args.set(0, url);
                let ctor = js_sys::Reflect::get(&solana,&JsValue::from("Connection"))?;
                (TransportMode::Validator, js_sys::Reflect::construct(&ctor.into(),&args)?, None)
            } else if regex::Regex::new(r"^https?://").unwrap().is_match(network) {
                let args = Array::new_with_length(1);
                args.set(0, JsValue::from(network));
                let ctor = js_sys::Reflect::get(&solana,&JsValue::from("Connection"))?;
                log_trace!("ctor: {:?}", ctor);
                (TransportMode::Validator, js_sys::Reflect::construct(&ctor.into(),&args)?, None)
            } else {
                return Err(error!("Transport cluster must be mainnet-beta, devnet, testnet, simulation").into());
            };
        // match network {
            // "simulator" | "simulation" => {
            //     // (JsValue::NULL, Some(Arc::new(Box::new(Simulator::try_new_with_store()?))))
            // },
            // "mainnet-beta" | "testnet" | "devnet" => {
            // },
            // _ => {
            //     let is_match = regex::Regex::new(r"^https?://").unwrap().is_match(network);
            //     if is_match {
            //     } else {
            //         //log_trace!("Transport creation error");
            //     }
            // }
        // };

        let wallet = Arc::new(foreign::Wallet::try_new()?);

        log_trace!("Transport interface creation ok...");
        
        // let entrypoints = Arc::new(RwLock::new(HashMap::new()));
        let queue  = Arc::new(TransactionQueue::new());
        log_trace!("Creating caching store");
        let cache = Cache::new_with_default_capacity();
        log_trace!("Creating lookup handler");
        let lookup_handler = LookupHandler::new();
        let reflector = Reflector::new();

        let config = Arc::new(RwLock::new(config));

        let transport = Arc::new(Transport {
            mode,
            emulator,
            config,
            connection,
            wallet,
            // wallet : JsValue::UNDEFINED,
            queue,
            cache,
            lookup_handler,
            reflector,
            custom_authority:Arc::new(Mutex::new(None))
        });

        unsafe { TRANSPORT = Some(transport.clone()); }
        log_trace!("Transport init successful");

        Ok(transport)
    }


    pub fn global() -> Result<Arc<Transport>> {
        let transport = unsafe { (&TRANSPORT).as_ref().unwrap().clone() };
        Ok(transport.clone())
    }

    #[inline(always)]
    pub fn emulator<'transport>(&'transport self) -> &'transport Arc<dyn EmulatorInterface> {
        self.emulator.as_ref().expect("missing emulator interface")
    }

    pub async fn lookup_remote_impl(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        self.cache.purge(Some(pubkey))?;
        
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {
                // let emulator = self.emulator.as_ref().unwrap();
                // self.emulator().lookup(pubkey).await
            // Some(emulator) => {
            //     Ok(emulator.lookup(&pubkey).await?)

                workflow_core::task::sleep(std::time::Duration::from_millis(5000)).await;

                let reference = self.emulator().lookup(pubkey).await?;
                match reference {
                    Some(reference) => {
                        self.cache.store(&reference)?;
                        Ok(Some(reference))
                    },
                    None => Ok(None)
                }

            },
            TransportMode::Validator => {

                let response = {
                    let pk_jsv = self.pubkey_to_jsvalue(&pubkey).unwrap();
                    let args = Array::new_with_length(1);
                    args.set(0 as u32, pk_jsv);
                    let connection = &self.connection()?;
                    let get_account_info_fn = unsafe { js_sys::Reflect::get(connection, &JsValue::from("getAccountInfo"))? };
                    let promise_jsv = unsafe { js_sys::Reflect::apply(&get_account_info_fn.into(), connection, &args.into())? };
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise_jsv)).await?
                };

                if response.is_null(){
                    // TODO review error handling & return None if success but no data
                    return Err(error!("Error fetching account data for {}",pubkey));
                }

                let rent_epoch = utils::try_get_u64_from_prop(&response,"rentEpoch")?;
                let lamports = utils::try_get_u64_from_prop(&response,"lamports")?;
                let owner = Pubkey::new(&utils::try_get_vec_from_bn_prop(&response,"owner")?);
                let data = utils::try_get_vec_from_prop(&response,"data")?;
                let _executable = utils::try_get_bool_from_prop(&response,"executable")?;

                let reference = Arc::new(AccountDataReference::new(AccountData::new_static_with_args(pubkey.clone(), owner, lamports, &data, rent_epoch)));
                self.cache.store(&reference)?;
                Ok(Some(reference))
            }
        }
    }

    pub fn pubkey_to_jsvalue(&self, pubkey: &Pubkey) -> Result<JsValue> {
        let pubkey_bytes = pubkey.to_bytes();
        let u8arr = unsafe { js_sys::Uint8Array::view(&pubkey_bytes[..]) };
        let pkargs = Array::new_with_length(1);
        pkargs.set(0 as u32, u8arr.into());
        // ? TODO - cache ctor inside Transport
        // let ctx = self.try_read().ok_or(JsValue::from("Transport rwlock solana"))?;
        // let inner = self.try_inner_with_msg("rwlock: Transport::solana")?;
        // let bridge = self.bridge.read().await;
        let ctor = unsafe { js_sys::Reflect::get(&Self::solana()?,&JsValue::from("PublicKey"))? };
        let pk_jsv = unsafe { js_sys::Reflect::construct(&ctor.into(),&pkargs)? };
        Ok(pk_jsv)
    }


    // async fn execute(self: &Arc<Self>, instruction : &Instruction) -> Result<()> { 
    async fn execute_impl(&self, instruction : &Instruction) -> Result<()> { 
        log_trace!("transport execute");
        // match &self.emulator {
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {

                let authority = self.get_authority_pubkey_impl()?;

                let resp = self.emulator().execute(
                    &authority,
                    instruction
                ).await?;

                // TODO - migrate into server
                workflow_core::task::sleep(std::time::Duration::from_millis(5000)).await;

                self.reflector.reflect(reflector::Event::EmulatorLogs(resp.logs));
                self.reflector.reflect(reflector::Event::WalletRefresh("SOL".into(), authority.clone()));
                match self.balance().await {
                    Ok(balance) => {
                        self.reflector.reflect(reflector::Event::WalletBalance("SOL".into(),authority.clone(),balance));
                    },
                    Err(err) => {
                        log_error!("Unable to update wallet balance: {}", err);
                    }
                }

                Ok(())
            },
            TransportMode::Validator => {
                log_trace!("native A");
                let wallet_adapter = &self.wallet_adapter()?;
                let accounts = &instruction.accounts;
                let accounts_arg = js_sys::Array::new_with_length(accounts.len() as u32);
                log_trace!("native B accounts.len():{}", accounts.len());
                for idx in 0..accounts.len() {
                    let account = &accounts[idx];
                    let account_public_key_jsv = self.pubkey_to_jsvalue(&account.pubkey)?;

                    let cfg = js_sys::Object::new();
                    unsafe {
                        js_sys::Reflect::set(&cfg, &"isWritable".into(), &JsValue::from(account.is_writable))?;
                        js_sys::Reflect::set(&cfg, &"isSigner".into(), &JsValue::from(account.is_signer))?;
                        js_sys::Reflect::set(&cfg, &"pubkey".into(), &account_public_key_jsv)?;
                    }
                    accounts_arg.set(idx as u32, cfg.into());
                }
                log_trace!("native C");
                let program_id = self.pubkey_to_jsvalue(&instruction.program_id)?;

                let instr_data_u8arr = unsafe { js_sys::Uint8Array::view(&instruction.data) };
                let instr_data_jsv : JsValue = instr_data_u8arr.into();
                
                let ctor = unsafe { js_sys::Reflect::get(&Self::solana()?, &JsValue::from("TransactionInstruction"))? };
                let cfg = js_sys::Object::new();
                unsafe {
                    js_sys::Reflect::set(&cfg, &"keys".into(), &accounts_arg)?;
                    js_sys::Reflect::set(&cfg, &"programId".into(), &program_id)?;
                    js_sys::Reflect::set(&cfg, &"data".into(), &instr_data_jsv)?;
                }

                log_trace!("native D");
                let tx_ins_args = js_sys::Array::new_with_length(1);
                tx_ins_args.set(0, JsValue::from(cfg));
                let tx_instruction_jsv = unsafe { js_sys::Reflect::construct(&ctor.into(), &tx_ins_args)? };
                
                let ctor = unsafe { js_sys::Reflect::get(&Self::solana()?, &JsValue::from("Transaction"))? };
                let tx_jsv = unsafe { js_sys::Reflect::construct(&ctor.into(), &js_sys::Array::new_with_length(0))? };
                
                
                let recent_block_hash = unsafe {
                    let get_latest_block_hash_fn = js_sys::Reflect::get(&self.connection()?, &"getLatestBlockhash".into())?;
                    let v = js_sys::Reflect::apply(&get_latest_block_hash_fn.into(), &self.connection()?, &js_sys::Array::new_with_length(0))?;
                    let prom = js_sys::Promise::from(v);
                    let recent_block_hash_result = JsFuture::from(prom).await?;
                    
                    log_trace!("recent_block_hash_result: {:?}", recent_block_hash_result);
                    js_sys::Reflect::get(&recent_block_hash_result, &"blockhash".into())?
                };

                log_trace!("recent_block_hash: {:?}", recent_block_hash);

                unsafe {
                    let wallet_public_key = js_sys::Reflect::get(&wallet_adapter, &JsValue::from("publicKey"))?;
                    js_sys::Reflect::set(&tx_jsv, &"feePayer".into(), &JsValue::from(wallet_public_key))?;
                    js_sys::Reflect::set(&tx_jsv, &"recentBlockhash".into(), &recent_block_hash)?;
                }
                
                utils::apply_with_args1(&tx_jsv, "add", tx_instruction_jsv)?;
                let promise_jsv = utils::apply_with_args1(&wallet_adapter, "signTransaction", tx_jsv.clone())?;
                let promise = js_sys::Promise::from(promise_jsv);
                let result = JsFuture::from(promise).await?;
                log_trace!("signTransaction result {:?}", result);
                let buffer_jsv = utils::apply_with_args0(&tx_jsv, "serialize")?;

                //let result = utils::apply_with_args1(&inner.connection, "sendRawTransaction", buffer_jsv)?;
                
                let options = js_sys::Object::new();
                unsafe {
                    js_sys::Reflect::set(&options, &"skipPreflight".into(), &JsValue::from(true))?;
                }

                let result = utils::apply_with_args2(&self.connection()?, "sendRawTransaction", buffer_jsv, options.into());
                match result {
                    Ok(_e)=>{
                        return Ok(());
                    },
                    Err(err)=>{
                        return Err(err.into());
                    }
                }
            }
        }
    }

}

// unsafe impl Send for Transport {}

#[async_trait(?Send)]
// #[async_trait]
impl super::Interface for Transport {

    fn get_authority_pubkey(&self) -> Result<Pubkey> {
        self.get_authority_pubkey_impl()
        // // let simulator = { self.try_inner()?.simulator.clone() };
        // match &self.emulator {
        //     Some(emulator) => {
        //         unimplemented!("TODO")
        //         // Ok(simulator.authority())
        //     },
        //     None => {
        //         todo!("not implemented")
        //     }
        // }

    }

    async fn post(&self, tx : Arc<Transaction>) -> Result<()> { 
        self.queue.enqueue(tx).await
    }
    async fn post_multiple(&self, txs : Vec<Arc<Transaction>>) -> Result<()> { 
        self.queue.enqueue_multiple(txs).await
    }

    async fn execute(&self, instruction : &Instruction) -> Result<()> { 
        self.execute_impl(instruction).await
    }

    fn purge(&self, pubkey: Option<&Pubkey>) -> Result<()> {
        Ok(self.cache.purge(pubkey)?)
    }


    // async fn lookup(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    async fn lookup(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let reference = self.clone().lookup_local(pubkey).await?;
        match reference {
            Some(reference) => Ok(Some(reference)),
            None => {
                Ok(self.lookup_remote(pubkey).await?)
            }
        }
    }

    // async fn lookup_local(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    async fn lookup_local(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let pubkey = Arc::new(pubkey.clone());
        Ok(self.cache.lookup(&pubkey)?)
    }


    async fn lookup_remote(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        let lookup_handler = &self.clone().lookup_handler;
        let request_type = lookup_handler.queue(pubkey).await;
        let result = match request_type {
            RequestType::New(receiver) => {
                self.reflector.reflect(reflector::Event::PendingLookups(lookup_handler.pending()));
                let response = self.clone().lookup_remote_impl(pubkey).await;
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



    // async fn lookup_remote(self : Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<RwLock<AccountData>>>> {
    //     match &self.simulator {
    //         Some(simulator) => {
    //             Ok(simulator.lookup(&pubkey).await?)
    //         },
    //         None => {
    //             Ok(self.lookup_remote_impl(&pubkey).await?)
    //         }
    //     }
    // }
}

