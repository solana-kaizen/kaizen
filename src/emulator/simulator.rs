use std::str::FromStr;
use std::sync::Arc;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use workflow_allocator::store;
use workflow_allocator::result::Result;
use workflow_allocator::accounts::{ AccountData, AccountDataReference };
use workflow_allocator::builder::{ InstructionBuilder, InstructionBuilderConfig };
use workflow_allocator::context::SimulationHandlerFn;
// use workflow_log::log_trace;
// use crate::generate_random_pubkey;

// use crate::generate_random_pubkey;

use crate::accounts::AccountDescriptorList;

use super::interface::{EmulatorInterface, ExecutionResponse};
use super::mockdata::InProcMockData;
use super::emulator::Emulator;
use async_trait::async_trait;

pub struct Simulator {
    pub inproc_mock_data : Option<InProcMockData>,
    pub store: Arc<dyn store::Store>, 
    pub emulator: Arc<Emulator>,
}

impl Simulator {

    pub fn new(store: &Arc<dyn store::Store>) -> Simulator {
        let emulator = Arc::new(Emulator::new(store.clone()));
        Simulator {
            store : store.clone(),
            emulator,
            inproc_mock_data : None,
        }
    }

    
    pub fn try_new_with_store() -> Result<Simulator> {
        let store = Arc::new(store::MemoryStore::new_local()?);
        let emulator = Arc::new(Emulator::new(store.clone()));
        let simulator = Simulator {
            store,
            emulator,
            inproc_mock_data : None,
        };
        Ok(simulator)
    }

    pub fn try_new_for_testing() -> Result<Simulator> {
        let store = Arc::new(store::MemoryStore::new_local()?);
        let emulator = Arc::new(Emulator::new(store.clone()));

        let simulator = Simulator {
            store,
            emulator,
            inproc_mock_data : None,
        };

        Ok(simulator)
    }

    
    pub async fn with_mock_accounts(mut self, program_id : Pubkey, authority : Option<Pubkey>) -> Result<Self> {

        let lamports = crate::utils::u64sol_to_lamports(500_000_000);

        let authority = match authority {
            Some(authority) => authority,
            None => { Pubkey::from_str("42bML5qB3WkMwfa2cosypjUrN7F2PLQm4qhxBdRDyW7f")? }
             //generate_random_pubkey(); 
        };
        
        let authority_account_data = AccountData::new_static(
            authority.clone(),
            program_id.clone(),
        ).with_lamports(lamports);

        self.store.store(&Arc::new(AccountDataReference::new(authority_account_data))).await?;//.await?;

        let mock_data = InProcMockData::new(
            &authority,
            // &identity,
            &program_id,
        );

        self.inproc_mock_data = Some(mock_data);
        Ok(self)

    }

    pub fn inproc_mock_data<'simulator>(&'simulator self) -> &'simulator InProcMockData {
        &self.inproc_mock_data.as_ref().expect("simulator missing inproc mock account data")
    }

    // pub fn with_simulated_identity(self : &Arc<Simulator>) -> Result<()> {
    //     // let allocation_args = AccountAllocationArgs::default();
    //     // let identity_account = ctx.create_pda(Identity::initial_data_len(), &allocation_args)?;
    //     // let mut identity = Identity::try_create(identity_account)?;

    //     Ok(())
    // }

    pub fn program_id(&self) -> Pubkey {
        self.inproc_mock_data().program_id
    }

    pub fn authority(&self) -> Pubkey {
        self.inproc_mock_data().authority
    }

    // pub fn identity(&self) -> Pubkey {
    //     self.inproc_mock_data().identity
    // }

    pub fn new_instruction_builder_config(
        &self,
    ) -> InstructionBuilderConfig {

        let InProcMockData { program_id, authority} = self.inproc_mock_data();
        // self.inproc.expect("simulator is missing inproc mock data");

        // let inner = self.inner().unwrap();
        let config = InstructionBuilderConfig::new(
            program_id.clone(),
        )
        .with_authority(authority);
        // .with_identity(identity);

        config
    }

    pub fn new_instruction_builder(
        &self,
    ) -> Arc<InstructionBuilder> {
        let InProcMockData { program_id, authority} = self.inproc_mock_data();

        // let inner = self.inner().unwrap();

        let builder = InstructionBuilder::new(
            &program_id,
            0,
            0u16,
        )
        .with_authority(authority);
        // .with_identity(identity);

        builder
    }
    
    pub async fn execute_handler(
        &self,
        builder: Arc<InstructionBuilder>,
        handler: SimulationHandlerFn,
    ) -> Result<()> {
        self.emulator.clone().execute_handler(builder,handler).await
    }
}

#[async_trait]
impl EmulatorInterface for Simulator {
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        match self.emulator.lookup(pubkey).await? {
            Some(reference) => {
                // IMPORTANT:
                // We intentionally return account replicas
                // in order to decouple account_data mutexes
                // which can create deadlocks if held by client
                // while executing programs.
                Ok(Some(reference.replicate()?))
            },
            None => {
                return Ok(None);
            }
        }
    }

    async fn execute(
        &self,
        authority: &Pubkey,
        instruction : &Instruction,
    ) -> Result<ExecutionResponse> {
        self.emulator.execute(authority,instruction).await
    }

    async fn fund(
        &self,
        key: &Pubkey,
        owner: &Pubkey,
        lamports: u64
    ) -> Result<()> {
        self.emulator.fund(key,owner,lamports).await
    }

    async fn list(&self) -> Result<AccountDescriptorList> {
        self.emulator.list().await
    }

}