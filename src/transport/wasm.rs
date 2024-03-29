//!
//! Solana network interface (WASM-browser)
//!
#![allow(unused_unsafe)]
use super::TransportMode;
//use crate::accounts::AccountData;
use crate::accounts::AccountDataReference;
use crate::emulator::client::EmulatorRpcClient;
use crate::emulator::interface::EmulatorInterface;
use crate::emulator::Simulator;
use crate::error;
use crate::result::Result;
use crate::transport::lookup::{LookupHandler, RequestType};
use crate::transport::queue::TransactionQueue;
use crate::transport::{reflector, Reflector};
use crate::transport::{Transaction, TransportConfig};
use crate::utils::pubkey_from_slice;
use crate::wallet::*;
use workflow_core::id::Id;
// use crate::wasm::*;
use async_std::sync::RwLock;
use async_trait::async_trait;
use js_sys::*;
use kaizen::{
    cache::Cache,
    wasm::{solana, workflow},
};
// use rand::*;
//use super::api::RpcProgramAccountsConfig;
use crate::transport::api::*;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
//use solana_rpc_client_api::RpcProgramAccountsConfig;
use std::convert::From;
use std::sync::{Arc, Mutex};
use std::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
//use wasm_bindgen_futures::JsFuture;
use solana_web3_sys::prelude::*;
use workflow_log::*;
use workflow_wasm::{init::global, utils};

static mut TRANSPORT: Option<Arc<Transport>> = None;

pub struct UnitTestConfig {
    pub program_id: Pubkey,
    pub authority: Pubkey,
}

mod wasm_bridge {
    use super::*;

    #[wasm_bindgen]
    pub struct Transport {
        #[wasm_bindgen(skip)]
        pub transport: Arc<super::Transport>,
    }

    #[wasm_bindgen]
    impl Transport {
        #[wasm_bindgen(constructor)]
        pub async fn new(network: String) -> std::result::Result<Transport, JsValue> {
            log_trace!("Creating Transport (WASM bridge)");
            let transport =
                super::Transport::try_new(network.as_str(), super::TransportConfig::default())
                    .await
                    .map_err(JsValue::from)?;
            Ok(Transport { transport })
        }
        #[wasm_bindgen(js_name = "withWallet")]
        pub fn with_wallet(&mut self, wallet: JsValue) -> std::result::Result<(), JsValue> {
            self.transport.with_wallet(wallet)?;
            Ok(())
        }

        #[wasm_bindgen(js_name = "InProcUnitTests")]
        pub async fn in_proc_unit_tests(
            program_id: Pubkey,
            authority: Option<Pubkey>,
        ) -> std::result::Result<Transport, JsValue> {
            let transport = super::Transport::try_new_for_unit_tests(
                program_id,
                authority,
                TransportConfig::default(),
            )
            .await
            .map_err(JsValue::from)?;

            Ok(Transport { transport })
        }

        #[wasm_bindgen(js_name = "getAuthorityPubkey")]
        pub fn get_authority_pubkey(&self) -> Result<Pubkey> {
            self.transport.get_authority_pubkey_impl()
        }

        #[wasm_bindgen(js_name = "balance")]
        pub fn balance(&self) -> Promise {
            let transport = self.transport.clone();
            future_to_promise(async move {
                let balance = transport.balance().await?;
                Ok(JsValue::from(balance))
            })
        }

        /*
        #[wasm_bindgen(js_name = "testTx")]
        pub async fn test_tx(&self)->Result<JsValue>{
            let transport = &self.transport;
            use std::str::FromStr;

            let wallet_adapter: WalletAdapter = transport.wallet_adapter()?.into();
            //return Ok(wallet_adapter.into());
            let connection = transport.connection()?.unwrap();

            let recent_block_hash = connection.get_latest_block_hash().await?.block_hash();
            log_trace!("recent_block_hash: {:?}", recent_block_hash);

            let wallet_public_key = wallet_adapter.pubkey();
            log_trace!("wallet_public_key: {:?}", wallet_public_key);

            let instruction = Instruction{
                program_id: Pubkey::from_str("5UAQGzYRWKEgdbpZCqoUjKDKiWpNbHeataWknRpvswEH").unwrap(),
                accounts: vec![
                    solana_program::instruction::AccountMeta {
                        pubkey: Pubkey::from_str("J92gL9eTqSLMGZQzr2yw2Jh2Wbsk1UEtJEnsMNY2HS9D").unwrap(),
                        is_signer: true,
                        is_writable: true
                    },
                    solana_program::instruction::AccountMeta {
                        pubkey: Pubkey::from_str("YA7NvczboDEtoBUUqFQzhX1NLtDf6qKEYQFiLqrNubm").unwrap(),
                        is_signer: false,
                        is_writable: true
                    }
                ],
                data: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 17, 0, 2, 0, 1, 0, 0, 0, 1, 1, 0, 0, 250, 137, 185, 186, 2, 0, 0]
            };

            let ins  = TransactionInstruction::try_from(&instruction).unwrap();
            log_trace!("wallet_public_key: {:?}", wallet_public_key);

            let tx_jsv = solana_web3_sys::transaction::Transaction::new();
            tx_jsv.set_fee_payer(wallet_public_key);
            tx_jsv.set_recent_block_hash(recent_block_hash);
            tx_jsv.add(ins);
            let result = wallet_adapter.send_transaction(tx_jsv.clone(), connection).await;
            log_trace!("sign_and_send_transaction result: {:?}", result);
            Ok(tx_jsv.into())
        }
        */

        #[wasm_bindgen(js_name = "getProgramAccounts")]
        pub async fn get_program_accounts_with_config(
            &self,
            pubkey: &Pubkey,
            config: JsValue,
        ) -> Result<JsValue> {
            log_trace!("getProgramAccounts: pubkey: {pubkey:?}");
            log_trace!("getProgramAccounts: config: {config:?}");
            let config: RpcProgramAccountsConfig = match config.try_into() {
                Ok(config) => config,
                Err(err) => {
                    return Err(
                        format!("Unable to convert JsValue to Config object: {err:?}").into(),
                    );
                }
            };

            let list = self
                .transport
                .connection()?
                .unwrap()
                .get_program_accounts_with_config(pubkey, config)
                .await?;

            let array = Array::new();
            for item in list {
                let item_array = Array::new();
                item_array.push(&PublicKey::try_from(&item.0)?.into());
                item_array.push(&ProgramAccount::try_from(item.1)?.into());
                array.push(&item_array.into());
            }

            Ok(array.into())
        }
    }
}
pub struct Transport {
    mode: TransportMode,
    pub emulator: Option<Arc<dyn EmulatorInterface>>,
    pub wallet: Arc<dyn foreign::WalletInterface>,
    pub queue: Arc<TransactionQueue>,
    cache: Arc<Cache>,
    pub config: Arc<RwLock<TransportConfig>>,
    pub custom_authority: Arc<Mutex<Option<Pubkey>>>,
    connection: Option<Connection>,
    pub lookup_handler: LookupHandler<Pubkey, Arc<AccountDataReference>>,
    pub reflector: Reflector,
}

unsafe impl Send for Transport {}
unsafe impl Sync for Transport {}

impl Transport {
    pub fn workflow() -> std::result::Result<JsValue, JsValue> {
        workflow()
    }

    pub fn solana() -> std::result::Result<JsValue, JsValue> {
        solana()
    }

    pub fn mode(&self) -> TransportMode {
        self.mode.clone()
    }

    pub fn reflector(&self) -> Reflector {
        self.reflector.clone()
    }

    pub fn connection(&self) -> std::result::Result<Option<Connection>, JsValue> {
        Ok(self.connection.clone())
    }

    pub fn with_wallet(&self, wallet: JsValue) -> std::result::Result<JsValue, JsValue> {
        js_sys::Reflect::set(&global()?, &"wallet".into(), &wallet)?;
        Ok(JsValue::from(true))
    }

    pub fn wallet_adapter(&self) -> std::result::Result<JsValue, JsValue> {
        let wallet = js_sys::Reflect::get(&global()?, &"wallet".into())?;
        if wallet == JsValue::UNDEFINED {
            log_trace!("wallet adapter is missing");
            return Err(error!(
                "WalletAdapterIsMissing, use `transport.with_wallet(walletAdapter);`"
            )
            .into());
        }
        Ok(wallet)
    }

    pub fn set_custom_authority(&self, key: Option<Pubkey>) -> Result<()> {
        (*self.custom_authority.lock()?) = key;
        Ok(())
    }

    /// Returns all accounts owned by the provided program pubkey.
    ///
    /// # RPC Reference
    ///
    /// This method is built on the [`getProgramAccounts`] RPC method.
    ///
    /// [`getProgramAccounts`]: https://docs.solana.com/developing/clients/jsonrpc-api#getprogramaccounts
    ///
    /// # Examples
    ///
    /// ```
    /// let config = GetProgramAccountsConfig::new()
    ///    .add_filters(vec![
    ///        AccountFilter::MemcmpEncodedBase58(8, pubkey.to_string()),
    ///        AccountFilter::MemcmpEncodeBase58(40, vec![1]),
    ///    ])?
    ///    .encoding(AccountEncoding::Base64)?;
    ///
    /// let transport = Transport::global()?;
    /// let accounts = transport
    ///    .get_program_accounts_with_config(&crate::program_id(), config)
    ///    .await?;
    /// ```
    pub async fn get_program_accounts_with_config(
        &self,
        pubkey: &Pubkey,
        config: GetProgramAccountsConfig,
    ) -> Result<Vec<(Pubkey, Account)>> {
        //log_trace!("config: {config:#?}");
        Ok(self
            .connection()?
            .unwrap()
            .get_program_accounts_with_config(pubkey, config.try_into()?)
            .await?)
    }

    pub fn public_key_ctor() -> std::result::Result<JsValue, JsValue> {
        js_sys::Reflect::get(&Self::solana()?, &JsValue::from("PublicKey"))
    }

    #[inline(always)]
    pub fn new_wallet(&self) -> Arc<dyn foreign::WalletInterface> {
        self.wallet.clone()
    }

    pub fn is_emulator(&self) -> Result<bool> {
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn balance(&self) -> Result<u64> {
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {
                let pubkey: Pubkey = self.get_authority_pubkey_impl()?;
                let result = self
                    .emulator()
                    .expect("Transport::balance(): Missing emulator interface")
                    .lookup(&pubkey)
                    .await?;
                match result {
                    Some(reference) => Ok(reference.lamports()?),
                    None => {
                        return Err(error!(
                            "[Emulator] - WASM::Transport::balance() unable to lookup account: {}",
                            pubkey
                        ));
                    }
                }
            }
            TransportMode::Validator => {
                let pubkey: Pubkey = self.get_authority_pubkey_impl()?;
                let result = self.lookup_remote_impl(&pubkey).await?;
                match result {
                    Some(reference) => Ok(reference.lamports()?),
                    None => {
                        return Err(error!(
                            "WASM::Transport::balance() unable to lookup account: {}",
                            pubkey
                        ));
                    }
                }
            }
        }
    }

    pub fn get_authority_pubkey_impl(&self) -> Result<Pubkey> {
        match self.mode {
            TransportMode::Inproc => {
                let simulator = self
                    .emulator
                    .clone()
                    .unwrap()
                    .downcast_arc::<Simulator>()
                    .expect("Unable to downcast to Simulator");

                Ok(simulator.authority())
            }

            TransportMode::Emulator => {
                if let Some(key) = self.custom_authority.lock()?.as_ref() {
                    return Ok(*key);
                }
                let wallet_adapter = &self.wallet_adapter()?;
                let public_key =
                    unsafe { js_sys::Reflect::get(wallet_adapter, &JsValue::from("publicKey"))? };
                let pubkey = pubkey_from_slice(&utils::try_get_vec_u8_from_bn(&public_key)?)?;
                Ok(pubkey)
            }

            TransportMode::Validator => {
                let wallet_adapter = &self.wallet_adapter()?;
                let public_key =
                    unsafe { js_sys::Reflect::get(wallet_adapter, &JsValue::from("publicKey"))? };
                let pubkey = pubkey_from_slice(&utils::try_get_vec_u8_from_bn(&public_key)?)?;
                Ok(pubkey)
            }
        }
    }

    pub async fn root(&self) -> Pubkey {
        self.config.read().await.root
    }

    pub async fn try_new_for_unit_tests(
        program_id: Pubkey,
        authority: Option<Pubkey>,
        config: TransportConfig,
    ) -> Result<Arc<Transport>> {
        let simulator = Simulator::try_new_for_testing()?
            .with_mock_accounts(program_id, authority)
            .await?;
        let emulator: Arc<dyn EmulatorInterface> = Arc::new(simulator);
        Transport::try_new_with_args(TransportMode::Inproc, None, Some(emulator), config).await
    }

    pub async fn try_new(network: &str, config: TransportConfig) -> Result<Arc<Transport>> {
        log_trace!("Creating transport (rust) for network {}", network);
        if unsafe { TRANSPORT.is_some() } {
            return Err(error!("Transport already initialized"));
        }

        let solana = Self::solana()?;

        if network == "inproc" {
            let emulator: Arc<dyn EmulatorInterface> = Arc::new(Simulator::try_new_with_store()?);
            Transport::try_new_with_args(TransportMode::Inproc, None, Some(emulator), config).await
        } else if regex::Regex::new(r"^rpcs?://").unwrap().is_match(network) {
            let emulator = Arc::new(EmulatorRpcClient::new(network)?);
            emulator.connect_as_task()?;
            let emulator: Arc<dyn EmulatorInterface> = emulator;
            Transport::try_new_with_args(TransportMode::Emulator, None, Some(emulator), config)
                .await
        } else if network == "mainnet-beta" || network == "testnet" || network == "devnet" {
            let cluster_api_url_fn =
                js_sys::Reflect::get(&solana, &JsValue::from("clusterApiUrl"))?;
            let args = Array::new_with_length(1);
            args.set(0, JsValue::from(network));
            let url = js_sys::Reflect::apply(&cluster_api_url_fn.into(), &JsValue::NULL, &args)?;
            log_trace!("{network}: {:?}", url.as_string());

            // let args = Array::new_with_length(1);
            // args.set(0, url);
            // let ctor = js_sys::Reflect::get(&solana, &JsValue::from("Connection"))?;
            Transport::try_new_with_args(
                TransportMode::Validator,
                // js_sys::Reflect::construct(&ctor.into(), &args)?,
                Some(Connection::new_with_commitment(
                    url.as_string().unwrap(),
                    "confirmed".into(),
                )),
                None,
                config,
            )
            .await
        } else if regex::Regex::new(r"^https?://").unwrap().is_match(network) {
            // let args = Array::new_with_length(1);
            // args.set(0, JsValue::from(network));
            // let ctor = js_sys::Reflect::get(&solana, &JsValue::from("Connection"))?;
            // log_trace!("ctor: {:?}", ctor);

            Transport::try_new_with_args(
                TransportMode::Validator,
                // js_sys::Reflect::construct(&ctor.into(), &args)?,
                Some(Connection::new_with_commitment(
                    network.to_string(),
                    "confirmed".into(),
                )),
                None,
                config,
            )
            .await
        } else {
            return Err(error!(
                "Transport cluster must be mainnet-beta, devnet, testnet, simulation"
            ));
        }
    }

    pub async fn try_new_with_args(
        mode: TransportMode,
        connection: Option<Connection>,
        emulator: Option<Arc<dyn EmulatorInterface>>,
        config: TransportConfig,
    ) -> Result<Arc<Transport>> {
        let wallet = Arc::new(foreign::Wallet::try_new()?);

        let queue = Arc::new(TransactionQueue::new());
        let cache = Arc::new(Cache::new_with_default_capacity());
        let config = Arc::new(RwLock::new(config));
        let lookup_handler = LookupHandler::new();
        let reflector = Reflector::new();

        let transport = Transport {
            mode,
            emulator,
            config,
            connection,
            wallet,
            queue,
            cache,
            lookup_handler,
            reflector,
            custom_authority: Arc::new(Mutex::new(None)),
        };

        let transport = Arc::new(transport);
        unsafe {
            TRANSPORT = Some(transport.clone());
        }

        Ok(transport)
    }

    pub fn global() -> Result<Arc<Transport>> {
        let transport = unsafe {
            TRANSPORT
                .as_ref()
                .expect("Transport is not initialized")
                .clone()
        };
        Ok(transport)
    }

    #[inline(always)]
    pub fn emulator(&self) -> Option<&Arc<dyn EmulatorInterface>> {
        self.emulator.as_ref()
    }

    pub async fn lookup_remote_impl(
        &self,
        pubkey: &Pubkey,
    ) -> Result<Option<Arc<AccountDataReference>>> {
        self.cache.purge(Some(pubkey))?;

        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {
                // let delay: u64 = rand::thread_rng().gen_range(500..1500);
                // workflow_core::task::sleep(std::time::Duration::from_millis(delay)).await;

                let reference = self
                    .emulator()
                    .expect("Transport::lookup_remote_impl(): Missing emulator interface")
                    .lookup(pubkey)
                    .await?;
                match reference {
                    Some(reference) => {
                        self.cache.store(&reference)?;
                        Ok(Some(reference))
                    }
                    None => Ok(None),
                }
            }
            TransportMode::Validator => {
                let account = self.connection()?.unwrap().get_account_info(pubkey).await?;

                //log_trace!("get_account_info ({}) response: {:#?}", pubkey, account);

                let reference = Arc::new(AccountDataReference::from((*pubkey, account)));
                self.cache.store(&reference)?;
                Ok(Some(reference))

                //if response.is_null() {
                // TODO review error handling & return None if success but no data
                //   return Err(error!("Error fetching account data for {}", pubkey));
                //}

                /*
                let rent_epoch = utils::try_get_u64_from_prop(&response, "rentEpoch")?;
                let lamports = utils::try_get_u64_from_prop(&response, "lamports")?;
                // let owner = Pubkey::new_from_array(
                //     <[u8; 32]>::try_from(
                //         <&[u8]>::clone(
                //             &utils::try_get_vec_u8_from_bn_prop(&response, "owner")?.as_slice()
                //         )
                //     )?
                // );
                let owner =
                    pubkey_from_slice(&utils::try_get_vec_u8_from_bn_prop(&response, "owner")?)?;
                // let owner = Pubkey::new(&utils::try_get_vec_u8_from_bn_prop(&response, "owner")?);
                let data = utils::try_get_vec_u8_from_prop(&response, "data")?;
                let _executable = utils::try_get_bool_from_prop(&response, "executable")?;

                let reference = Arc::new(AccountDataReference::new(
                    AccountData::new_static_with_args(*pubkey, owner, lamports, &data, rent_epoch),
                ));
                self.cache.store(&reference)?;
                Ok(Some(reference))
                */
            }
        }
    }

    pub fn pubkey_to_jsvalue(&self, pubkey: &Pubkey) -> Result<JsValue> {
        let pubkey_bytes = pubkey.to_bytes();
        let u8arr = unsafe { js_sys::Uint8Array::view(&pubkey_bytes[..]) };
        let pkargs = Array::new_with_length(1);
        pkargs.set(0u32, u8arr.into());
        // TODO - cache ctor inside Transport
        let ctor = unsafe { js_sys::Reflect::get(&Self::solana()?, &JsValue::from("PublicKey"))? };
        let pk_jsv = unsafe { js_sys::Reflect::construct(&ctor.into(), &pkargs)? };
        Ok(pk_jsv)
    }

    async fn execute_impl(&self, instruction: &Instruction) -> Result<()> {
        log_trace!("transport execute");
        match self.mode {
            TransportMode::Inproc | TransportMode::Emulator => {
                let authority = self.get_authority_pubkey_impl()?;

                let resp = self
                    .emulator()
                    .expect("Transport::execute_impl(): Missing emulator interface")
                    .execute(&authority, instruction)
                    .await?;

                // TODO - migrate into server
                // workflow_core::task::sleep(std::time::Duration::from_millis(5000)).await;

                self.reflector
                    .reflect(reflector::Event::EmulatorLogs(resp.logs));
                self.reflector
                    .reflect(reflector::Event::WalletRefresh("SOL".into(), authority));
                match self.balance().await {
                    Ok(balance) => {
                        self.reflector.reflect(reflector::Event::WalletBalance(
                            "SOL".into(),
                            authority,
                            balance,
                        ));
                    }
                    Err(err) => {
                        log_error!("Unable to update wallet balance: {}", err);
                    }
                }

                Ok(())
            }
            TransportMode::Validator => {
                let wallet_adapter: WalletAdapter = self.wallet_adapter()?.into();
                let connection = self.connection()?.unwrap();

                let recent_block_hash = connection.get_latest_block_hash().await?.block_hash();
                log_trace!("recent_block_hash: {:?}", recent_block_hash);

                let wallet_public_key = wallet_adapter.pubkey();
                log_trace!("wallet_public_key: {:?}", wallet_public_key);

                let tx_jsv = solana_web3_sys::transaction::Transaction::new();
                tx_jsv.set_fee_payer(wallet_public_key);
                tx_jsv.set_recent_block_hash(recent_block_hash);
                tx_jsv.add(instruction.try_into()?);

                log_trace!("tx_jsv###: {tx_jsv:?}, instruction:{instruction:?}");

                let result = wallet_adapter.send_transaction(tx_jsv, connection).await?;
                log_trace!("send_transaction result: {:?}", result);

                // let result = wallet_adapter.sign_transaction(&tx_jsv).await?;
                // log_trace!("signTransaction result {:?}", result);

                // let config = SerializeConfig::new();
                // config.set_require_all_signatures(false);
                // config.set_verify_signatures(false);
                // let result = connection
                //     .send_raw_transaction_with_options(
                //         tx_jsv.serialize(config),
                //         SendRawTxOptions::new().skip_preflight(false),
                //     )
                //     .await?;

                // log_trace!("send_raw_transaction result: {:?}", result);
                Ok(())
                /*
                match result {
                    Ok(_e) => {
                        return Ok(());
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                }
                */
            }
        }
    }
}

#[async_trait(?Send)]
impl super::Interface for Transport {
    fn get_authority_pubkey(&self) -> Result<Pubkey> {
        self.get_authority_pubkey_impl()
    }

    async fn post(&self, tx: Arc<Transaction>) -> Result<()> {
        self.queue.enqueue(tx).await
    }
    async fn post_multiple(&self, txs: Vec<Arc<Transaction>>) -> Result<()> {
        self.queue.enqueue_multiple(txs).await
    }
    async fn discard_chain(&self, id: &Id) -> Result<()> {
        self.queue.discard_chain(id).await
    }

    async fn execute(&self, instruction: &Instruction) -> Result<()> {
        self.execute_impl(instruction).await
    }

    fn purge(&self, pubkey: Option<&Pubkey>) -> Result<()> {
        self.cache.purge(pubkey)
    }

    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let reference = self.lookup_local(pubkey).await?;
        match reference {
            Some(reference) => Ok(Some(reference)),
            None => Ok(self.lookup_remote(pubkey).await?),
        }
    }

    async fn lookup_local(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let pubkey = Arc::new(*pubkey);
        Ok(self.cache.lookup(&pubkey)?)
    }

    async fn lookup_remote(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let lookup_handler = &self.lookup_handler;
        let request_type = lookup_handler.queue(pubkey).await;
        let result = match request_type {
            RequestType::New(receiver) => {
                self.reflector
                    .reflect(reflector::Event::PendingLookups(lookup_handler.pending()));
                let response = self.lookup_remote_impl(pubkey).await;
                lookup_handler.complete(pubkey, response).await;
                receiver.recv().await?
            }
            RequestType::Pending(receiver) => receiver.recv().await?,
        };

        self.reflector
            .reflect(reflector::Event::PendingLookups(lookup_handler.pending()));
        result
    }
}
