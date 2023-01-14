use kaizen::prelude::*;
use kaizen::result::Result;
use kaizen::transport::TransportMode;
use std::sync::Mutex;

use crate::error;

#[derive(Clone)]
pub enum IdentityState {
    Unknown,
    Missing,
    Present,
}

pub struct Inner {
    authority_pubkey: Option<Pubkey>,
    identity_pubkey: Option<Pubkey>,
    identity_state: IdentityState,
    transport_mode: Option<TransportMode>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            authority_pubkey: None,
            identity_pubkey: None,
            identity_state: IdentityState::Unknown,
            transport_mode: None,
        }
    }

    pub fn new_with_args(
        transport_mode: &TransportMode,
        authority_pubkey: &Pubkey,
        identity_pubkey: &Pubkey,
    ) -> Self {
        Self {
            authority_pubkey: Some(authority_pubkey.clone()),
            identity_pubkey: Some(identity_pubkey.clone()),
            identity_state: IdentityState::Present,
            transport_mode: Some(transport_mode.clone()),
        }
    }
}

#[derive(Clone)]
pub struct User {
    inner: Arc<Mutex<Inner>>,
    sequencer: Sequencer,
}

impl User {
    pub fn new() -> Self {
        User {
            inner: Arc::new(Mutex::new(Inner::new())),
            sequencer: Sequencer::default(),
        }
    }

    pub fn new_with_args(
        transport_mode: &TransportMode,
        authority_pubkey: &Pubkey,
        identity_pubkey: &Pubkey,
        sequencer: &Sequencer,
    ) -> Self {
        User {
            inner: Arc::new(Mutex::new(Inner::new_with_args(
                transport_mode,
                authority_pubkey,
                identity_pubkey,
            ))),
            sequencer: sequencer.clone(),
        }
    }

    pub fn transport_mode(&self) -> Option<TransportMode> {
        self.inner.lock().unwrap().transport_mode.as_ref().cloned()
    }

    pub fn identity(&self) -> Pubkey {
        self.inner
            .lock()
            .unwrap()
            .identity_pubkey
            .expect("User::identity() missing identity pubkey")
            .clone()
    }

    pub fn authority(&self) -> Pubkey {
        self.inner
            .lock()
            .unwrap()
            .authority_pubkey
            .expect("User::authority() missing authority pubkey")
            .clone()
    }

    pub fn sequencer(&self) -> Sequencer {
        self.sequencer.clone()
    }

    pub fn builder_args(&self) -> Result<(Pubkey, Pubkey, Sequencer)> {
        let sequencer = self.sequencer.clone();
        let inner = self.inner.lock().unwrap();
        let authority = inner
            .authority_pubkey
            .ok_or(error!("User record is missing authority"))?;
        let identity = inner
            .identity_pubkey
            .ok_or(error!("User record is missing identity"))?;
        Ok((authority, identity, sequencer))
    }

    pub async fn load_identity(
        &self,
        program_id: &Pubkey,
        authority: &Pubkey,
    ) -> Result<Option<Arc<AccountDataReference>>> {
        match kaizen::identity::client::load_identity(program_id, authority).await? {
            Some(identity) => {
                let transport_mode = Transport::global()?.mode();
                self.sequencer.load_from_identity(&identity)?;
                let mut inner = self.inner.lock()?;
                inner.identity_state = IdentityState::Present;
                inner.identity_pubkey = Some(identity.pubkey().clone());
                inner.authority_pubkey = Some(authority.clone());
                inner.transport_mode = Some(transport_mode);
                Ok(Some(identity))
            }
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
            _ => Ok(false),
        }
    }

    pub fn is_checked(&self) -> Result<bool> {
        let inner = self.inner.lock()?;
        match inner.identity_state {
            IdentityState::Present => Ok(true),
            IdentityState::Missing => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn state(&self) -> Result<IdentityState> {
        let inner = self.inner.lock()?;
        Ok(inner.identity_state.clone())
    }
}
