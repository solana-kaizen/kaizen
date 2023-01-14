use crate::error;
use crate::prelude::log_trace;
use crate::transport::Transport;
use async_trait::async_trait;
use js_sys;
use kaizen::error::{js_error, parse_js_error};
use kaizen::{result::Result, wasm as wasm_utils};
use solana_program::pubkey::Pubkey;
use wasm_bindgen_futures::JsFuture;
use workflow_wasm::utils;

pub struct Wallet {}

impl Wallet {
    pub fn try_new() -> Result<Wallet> {
        let wallet = Self {};

        Ok(wallet)
    }
}

#[async_trait(?Send)]
impl super::WalletInterface for Wallet {
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
        let wallet_ready_state =
            wasm_utils::wallet_ready_state().expect("Wallet: unable to get wallet_ready_state.");
        let installed = utils::try_get_string(&wallet_ready_state, "Installed")
            .expect("Wallet: unable to get Installed property from WalletReadyState.");
        let mut adapters_info = Vec::new();
        for (index, adapter) in adapters.iter().enumerate() {
            let ready_state = utils::try_get_string(&adapter, "readyState")?;
            adapters_info.push(super::Adapter {
                icon: utils::try_get_string(&adapter, "icon")?,
                name: utils::try_get_string(&adapter, "name")?,
                index,
                detected: ready_state.eq(&installed),
            });
        }

        Ok(Some(adapters_info))
    }

    async fn connect(&self, adapter: Option<super::Adapter>) -> Result<()> {
        let adapters = wasm_utils::adapters()?;
        let mut adapter_selection = None;
        for (index, a) in adapters.iter().enumerate() {
            let name = utils::try_get_string(&a, "name")?;
            if let Some(adapter) = &adapter {
                if adapter.index == index && adapter.name.eq(&name) {
                    adapter_selection = Some(a);
                }
            } else {
                adapter_selection = Some(a);
                break;
            }
        }

        if let Some(adapter_jsv) = adapter_selection {
            let promise_jsv = utils::apply_with_args0(adapter_jsv, "connect")
                .expect("Wallet: Unable to get 'connect' method from WalletAdapter Object");
            let future = JsFuture::from(js_sys::Promise::from(promise_jsv));
            log_trace!("Wallet: WalletAdapter.connect() ........");
            let res = match future.await {
                Ok(r) => r,
                Err(e) => return Err(js_error! {e, "Wallet: WalletAdapter.connect() failed"}),
            };
            log_trace!(
                "Wallet: WalletAdapter.connect() future.await success: {:?}",
                res
            );
            Transport::global()?.with_wallet(adapter_jsv.clone())?;
            log_trace!("Wallet: WalletAdapter.connect() transport updated");
            Ok(())
        } else {
            Err(error!("Wallet: Unable to find wallet adapter."))
        }
    }
}
