use ahash::HashMap;
use solana_program::pubkey::Pubkey;
use std::sync::{Arc, Mutex};
use workflow_core::channel::{unbounded, Receiver, Sender};
use workflow_core::id::Id;
use workflow_log::log_error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    PendingLookups(usize),
    PendingTransactions(usize),
    WalletRefresh(String, Pubkey),
    WalletBalance(String, Pubkey, u64),
    EmulatorLogs(Vec<String>),
    Halt,
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
