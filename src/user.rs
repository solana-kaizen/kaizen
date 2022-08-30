use async_std::sync::Mutex;
use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;

pub enum IdentityState {
    Unknown,
    Missing,
    Present,
}

pub struct Inner {
    identity_state : IdentityState,
    identity_pubkey : Option<Pubkey>
}

impl Inner {
    pub fn new() -> Self {
        Self {
            identity_state: IdentityState::Unknown,
            identity_pubkey: None
        }
    }
}

#[derive(Clone)]
pub struct User {
    inner : Arc<Mutex<Inner>>,
}

impl User {
    pub fn new() -> Self {
        User {
            inner : Arc::new(Mutex::new(Inner::new()))
        }
    }

    pub async fn load_identity(&self, program_id : &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    
        match workflow_allocator::identity::client::load_identity(program_id).await? {
            Some(identity) => {
                let mut inner = self.inner.lock().await;
                inner.identity_state = IdentityState::Present;
                inner.identity_pubkey = Some(*identity.key);
                Ok(Some(identity))
            },
            None => {
                let mut inner = self.inner.lock().await;
                inner.identity_state = IdentityState::Missing;
                inner.identity_pubkey = None;
                Ok(None)
            }
        }

    }

    pub async fn is_present(&self) -> bool {
        let inner = self.inner.lock().await;
        match inner.identity_state {
            IdentityState::Present => true,
            _ => false
        }
    }

}