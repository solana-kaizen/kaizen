use std::sync::Mutex;
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
    sequencer : Sequencer,
}

impl User {
    pub fn new() -> Self {
        User {
            inner : Arc::new(Mutex::new(Inner::new())),
            sequencer : Sequencer::default(),
        }
    }

    // pub async fn load<'cr>() -> Result<Option<ContainerReference<'cr,IdentityContainer<'cr,'cr>>>> {

    pub async fn load_identity(&self, program_id : &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    // pub async fn load_identity<'cr>(&self, program_id : &Pubkey) -> Result<Option<ContainerReference<'cr,Identity<'cr,'cr>>>> {
    
        match workflow_allocator::identity::client::load_identity(program_id).await? {
            Some(identity) => {
                // let identity = identity.try_load_container::<Identity>()?;
                // self.sequencer.load_from_identity(&identity_container)?;

                self.sequencer.load_from_identity(&identity)?;
                let mut inner = self.inner.lock()?;
                inner.identity_state = IdentityState::Present;
                inner.identity_pubkey = Some(identity.pubkey().clone());
                Ok(Some(identity))
            },
            None => {
                let mut inner = self.inner.lock()?;
                inner.identity_state = IdentityState::Missing;
                inner.identity_pubkey = None;
                Ok(None)
            }
        }

    }

    pub fn is_present(&self) -> Result<bool> {
        let inner = self.inner.lock()?;
        match inner.identity_state {
            IdentityState::Present => Ok(true),
            _ => Ok(false)
        }
    }

}