//!
//! Client-side Transport activity tracker (for transactions, wallet and emulator updates).
//!
use crate::error::Error;
use crate::result::Result;
use ahash::HashMap;
use futures::{select, FutureExt};
use js_sys::{Function, Object};
use solana_program::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use workflow_core::channel::{unbounded, DuplexChannel, Receiver, Sender};
use workflow_core::id::Id;
use workflow_core::task::*;
use workflow_log::log_error;
use workflow_wasm::prelude::*;

use super::Transport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    PendingLookups(usize),
    PendingTransactions(usize),
    WalletRefresh(String, Pubkey),
    WalletBalance(String, Pubkey, u64),
    EmulatorLogs(Vec<String>),
    Halt,
}

impl TryFrom<&Event> for JsValue {
    type Error = Error;
    fn try_from(event: &Event) -> std::result::Result<JsValue, Self::Error> {
        let object = Object::new();
        match event {
            Event::PendingLookups(count) => {
                object.set("event", &"pending-lookups".into())?;
                object.set("count", &JsValue::from_f64(*count as f64))?;
            }
            Event::PendingTransactions(count) => {
                object.set("event", &"pending-transactions".into())?;
                object.set("count", &JsValue::from_f64(*count as f64))?;
            }
            Event::WalletRefresh(token, authority) => {
                object.set("event", &"wallet-refresh".into())?;
                object.set("token", &token.into())?;
                object.set("authority", &(*authority).into())?;
            }
            Event::WalletBalance(token, authority, balance) => {
                object.set("event", &"wallet-balance".into())?;
                object.set("token", &token.into())?;
                object.set("authority", &(*authority).into())?;
                object.set("balance", &JsValue::from_f64(*balance as f64))?;
            }
            Event::EmulatorLogs(logs) => {
                object.set("event", &"emulator-logs".into())?;
                let logs = logs
                    .iter()
                    .map(|log| JsValue::from(log.as_str()))
                    .collect::<Vec<JsValue>>();
                object.set_vec("logs", &logs)?;
            }
            Event::Halt => {
                object.set("event", &"halt".into())?;
            }
        }
        Ok(object.into())
    }
}

#[derive(Clone)]
pub struct Reflector {
    pub channels: Arc<Mutex<HashMap<Id, Sender<Event>>>>,
}

impl Default for Reflector {
    fn default() -> Self {
        Self::new()
    }
}

impl Reflector {
    pub fn new() -> Reflector {
        Reflector {
            channels: Arc::new(Mutex::new(HashMap::default())),
        }
    }

    pub fn register_event_channel(&self) -> (Id, Sender<Event>, Receiver<Event>) {
        let (sender, receiver) = unbounded();
        let id = Id::new();
        self.channels.lock().unwrap().insert(id, sender.clone());
        (id, sender, receiver)
    }

    pub fn unregister_event_channel(&self, id: Id) {
        self.channels.lock().unwrap().remove(&id);
    }

    pub fn reflect(&self, event: Event) {
        let channels = self.channels.lock().unwrap();
        for (_, sender) in channels.iter() {
            match sender.try_send(event.clone()) {
                Ok(_) => {}
                Err(err) => {
                    log_error!(
                        "Transport Reflector: error reflecting event {:?}: {:?}",
                        event,
                        err
                    );
                }
            }
        }
    }
}

///
/// [`ReflectorClient`] is an object meant to be use in WASM environment to
/// process [`Transport`] events. [`ReflectorClient`] auto-registers with the
/// global [`Transport`] and on the event processing task start and unregisters
/// when the event processing stop.  
///
///
///
#[wasm_bindgen]
pub struct ReflectorClient {
    callback: Arc<Mutex<Option<Sendable<Function>>>>,
    task_running: AtomicBool,
    task_ctl: DuplexChannel,
}

impl Default for ReflectorClient {
    fn default() -> Self {
        ReflectorClient::new()
    }
}

#[wasm_bindgen]
impl ReflectorClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ReflectorClient {
        ReflectorClient {
            callback: Arc::new(Mutex::new(None)),
            task_running: AtomicBool::new(false),
            task_ctl: DuplexChannel::oneshot(),
        }
    }

    #[wasm_bindgen(js_name = "setHandler")]
    pub fn set_handler(&self, callback: JsValue) -> Result<()> {
        if callback.is_function() {
            let fn_callback: Function = callback.into();
            self.callback
                .lock()
                .unwrap()
                .replace(fn_callback.into());

            // self.start_notification_task()?;
        } else {
            self.remove_handler()?;
        }
        Ok(())
    }

    /// `removeHandler` must be called when releasing ReflectorClient
    /// to stop the background event processing task
    #[wasm_bindgen(js_name = "removeHandler")]
    pub fn remove_handler(&self) -> Result<()> {
        *self.callback.lock().unwrap() = None;
        Ok(())
    }

    #[wasm_bindgen(js_name = "start")]
    pub async fn start_notification_task(&self) -> Result<()> {
        if self.task_running.load(Ordering::SeqCst) {
            panic!("ReflectorClient task is already running");
        }
        let ctl_receiver = self.task_ctl.request.receiver.clone();
        let ctl_sender = self.task_ctl.response.sender.clone();
        let callback = self.callback.clone();
        self.task_running.store(true, Ordering::SeqCst);

        let transport = Transport::global().unwrap_or_else(|err| {
            panic!("ReflectorClient - missing global transport: {err}");
        });
        let (channel_id, _, receiver) = transport.reflector().register_event_channel();

        spawn(async move {
            loop {
                select! {
                    _ = ctl_receiver.recv().fuse() => {
                        break;
                    },
                    msg = receiver.recv().fuse() => {
                        // log_info!("notification: {:?}",msg);
                        if let Ok(notification) = &msg {
                            if let Some(callback) = callback.lock().unwrap().as_ref() {
                                if let Ok(event) = JsValue::try_from(notification) {
                                    if let Err(err) = callback.0.call1(&JsValue::undefined(), &event) {
                                        log_error!("Error while executing notification callback: {:?}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            transport.reflector().unregister_event_channel(channel_id);
            ctl_sender.send(()).await.ok();
        });

        Ok(())
    }

    #[wasm_bindgen(js_name = "stop")]
    pub async fn stop_notification_task(&self) -> Result<()> {
        if self.task_running.load(Ordering::SeqCst) {
            self.task_running.store(false, Ordering::SeqCst);
            self.task_ctl
                .signal(())
                .await
                .map_err(|err| JsError::new(&err.to_string()))?;
        }
        Ok(())
    }
}
