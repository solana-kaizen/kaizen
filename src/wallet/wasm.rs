// use std::sync::Arc;

// use std::path::Path;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
// use solana_sdk::signature::{Keypair, read_keypair_file};
// use solana_sdk::signer::Signer;
use workflow_allocator::result::Result;
use workflow_wasm::utils;
use js_sys;
use wasm_bindgen_futures::JsFuture;
// use wasm_bindgen::JsValue;
//use std::sync::{Mutex, Arc};
use workflow_allocator::wasm as wasm_utils;
use crate::prelude::log_trace;
use crate::transport::Transport;

use crate::error;

/*
struct JsValueExt(JsValue);

unsafe impl Send for JsValueExt {}
unsafe impl Sync for JsValueExt {}
*/
// unsafe impl Send for Wallet {}
// unsafe impl Sync for Wallet {}

pub struct Wallet {

}


impl Wallet {

    pub fn try_new() -> Result<Wallet> {
        let wallet = Self {

        };

        Ok(wallet)
    }


    // async fn await_connect(promise: JsValue) ->

    // async fn get_balance(&self) -> Result<u64>;

}

#[async_trait(?Send)]
impl super::Wallet for Wallet {

    fn is_connected(&self) -> bool {
        true
    }
    
    fn pubkey(&self) -> Result<Pubkey> {
        // Ok(self.keypair.pubkey())
        // temporary stub
        Ok(Pubkey::default())
    }

    async fn get_adapter_list(&self) -> Result<Option<Vec<super::Adapter>>> {
        let adapters = wasm_utils::adapters()?;
        let wallet_ready_state = wasm_utils::wallet_ready_state()
            .expect("Wallet: unable to get wallet_ready_state.");
        let installed = utils::try_get_string(&wallet_ready_state, "Installed")
            .expect("Wallet: unable to get Installed property from WalletReadyState.");
        //let win = js_sys::global();
        //let adapters_jsv = js_sys::Reflect::get(&win, &"WalletAdapters".into())?;
        //let adapters = js_sys::Array::from(&adapters_jsv);
        let mut adapters_info = Vec::new();
        for (index, adapter) in adapters.iter().enumerate(){
            let ready_state = utils::try_get_string(&adapter, "readyState")?;
            adapters_info.push(super::Adapter{
                icon: utils::try_get_string(&adapter, "icon")?,
                name: utils::try_get_string(&adapter, "name")?,
                index,
                detected:ready_state.eq(&installed)
            });
        }
        //log_trace!("adapters_info: {:?}, adapters_jsv:{:?}", adapters_info, adapters_jsv);

        Ok(Some(adapters_info))
    }



    async fn connect(&self, adapter: Option<super::Adapter>) -> Result<()> {


        let win = js_sys::global();
        let adapters_jsv = js_sys::Reflect::get(&win, &"WalletAdapters".into())?;
        let adapters = js_sys::Array::from(&adapters_jsv);
        let mut adapter_selection = None;
        for (index, a) in adapters.iter().enumerate(){
            let name = utils::try_get_string(&a, "name")?;
            if let Some(adapter) = &adapter{
                if adapter.index == index && adapter.name.eq(&name){
                    adapter_selection = Some(a);
                }
            }else{
                adapter_selection = Some(a);
                break;
            }
        }

        if let Some(adapter_jsv) = &adapter_selection{
            let res = utils::apply_with_args0(adapter_jsv, "connect");
            match res{
                Ok(promise)=>{
                    let future = JsFuture::from(js_sys::Promise::from(promise));
                    log_trace!("wallet.connect ........");
                    match future.await{
                        Ok(v)=>{
                            log_trace!("wallet.connect future.await success: {:?}", v);
                            Transport::global()?.with_wallet(adapter_jsv.clone())?;
                            log_trace!("wallet.connect future.await transport updated");
                            Ok(())
                            // Ok(v)
                        }
                        Err(e)=>{
                            log_trace!("wallet.connect future.await error: {:?}", e);
                            let msg = utils::try_get_string(&e, "message")?;
                            Err(error!("Error: {:?}", msg))
                        }
                    }
                }
                Err(err)=>{
                    Err(error!("{:?}", err))
                }
            }
        }else{
            Err(error!("Unable to find wallet adapter."))
        }
    }






    // async fn connect(&self, _adapter: Option<super::Adapter>) -> Result<()> {
        /*
        let win = js_sys::global();
        let adapters_jsv = js_sys::Reflect::get(&win, &"WalletAdapters".into())?;
        let adapters = js_sys::Array::from(&adapters_jsv);
        let mut adapter_selection = None;
        for (index, a) in adapters.iter().enumerate(){
            let name = utils::try_get_string(&a, "name")?;
            if let Some(adapter) = &adapter{
                if adapter.index == index && adapter.name.eq(&name){
                    adapter_selection = Some(a);
                }
            }else{
                adapter_selection = Some(a);
                break;
            }
        }

        let future = if let Some(adapter_jsv) = adapter_selection{
            let res = utils::apply_with_args0(&adapter_jsv, "connect");
            match res{
                Ok(promise)=>{
                    JsFuture::from(js_sys::Promise::from(promise))
                }
                Err(err)=>{
                    return Err(error!("{:?}", err));
                }
            }
        }else{
            return Err(error!("Unable to find wallet adapter."));
        };
        let err:Arc<Mutex<Option<Result<()>>>> = Arc::new(Mutex::new(None));
        let err_ = err.clone();
        workflow_core::task::wasm::spawn(async move {
            match future.await{
                Ok(_v)=>{
                    
                }
                Err(e)=>{
                    let mut error = err_.lock().expect("Unable to lock");
                    *error = Some(Err(error!("{:?}", e)));
                }
            }
        });
        let error = err.lock().expect("Unable to lock");
        match error.as_ref(){
            Some(e)=>{
                match e {
                    Ok(_v)=>{
                        Ok(())
                    }
                    Err(e)=>{
                        Err(error!("{:?}", e))
                    }
                }
            }
            None=>{
                Ok(())
            }
        }
        */
    //     Ok(())
    // }

    // async fn get_balance(&self) -> Result<u64>;

}