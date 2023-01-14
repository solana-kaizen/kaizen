use crate::accounts::AccountDescriptorList;
use kaizen::accounts::{AccountData, AccountDataReference};
use kaizen::builder::{InstructionBuilder, InstructionBuilderConfig};
use kaizen::context::SimulationHandlerFn;
use kaizen::result::Result;
use kaizen::store;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

use super::emulator::Emulator;
use super::interface::{EmulatorConfig, EmulatorInterface, ExecutionResponse};
use super::mockdata::InProcMockData;
use async_trait::async_trait;

pub struct Simulator {
    pub inproc_mock_data: Option<InProcMockData>,
    pub store: Arc<dyn store::Store>,
    pub emulator: Arc<Emulator>,
}

impl Simulator {
    pub fn new(store: &Arc<dyn store::Store>) -> Simulator {
        let emulator = Arc::new(Emulator::new(store.clone()));
        Simulator {
            store: store.clone(),
            emulator,
            inproc_mock_data: None,
        }
    }

    pub fn try_new_with_store() -> Result<Simulator> {
        let store = Arc::new(store::MemoryStore::new_local()?);
        let emulator = Arc::new(Emulator::new(store.clone()));
        let simulator = Simulator {
            store,
            emulator,
            inproc_mock_data: None,
        };
        Ok(simulator)
    }

    pub fn try_new_for_testing() -> Result<Simulator> {
        let store = Arc::new(store::MemoryStore::new_local()?);
        let emulator = Arc::new(Emulator::new(store.clone()));

        let simulator = Simulator {
            store,
            emulator,
            inproc_mock_data: None,
        };

        Ok(simulator)
    }

    pub async fn with_mock_accounts(
        mut self,
        program_id: Pubkey,
        authority: Option<Pubkey>,
    ) -> Result<Self> {
        let lamports = crate::utils::u64sol_to_lamports(500_000_000);

        let authority = match authority {
            Some(authority) => authority,
            // FIXME should user always supply a pubkey?
            None => Pubkey::from_str("42bML5qB3WkMwfa2cosypjUrN7F2PLQm4qhxBdRDyW7f")?, //generate_random_pubkey();
        };

        let authority_account_data =
            AccountData::new_static(authority.clone(), program_id.clone()).with_lamports(lamports);

        self.store
            .store(&Arc::new(AccountDataReference::new(authority_account_data)))
            .await?;

        let mock_data = InProcMockData::new(
            &authority,
            // &identity,
            &program_id,
        );

        self.inproc_mock_data = Some(mock_data);
        Ok(self)
    }

    pub fn inproc_mock_data<'simulator>(&'simulator self) -> &'simulator InProcMockData {
        &self
            .inproc_mock_data
            .as_ref()
            .expect("simulator missing inproc mock account data")
    }

    pub fn program_id(&self) -> Pubkey {
        self.inproc_mock_data().program_id
    }

    pub fn authority(&self) -> Pubkey {
        self.inproc_mock_data().authority
    }

    pub fn new_instruction_builder_config(&self) -> InstructionBuilderConfig {
        let InProcMockData {
            program_id,
            authority,
        } = self.inproc_mock_data();
        let config = InstructionBuilderConfig::new(program_id.clone()).with_authority(authority);

        config
    }

    pub fn new_instruction_builder(&self) -> Arc<InstructionBuilder> {
        let InProcMockData {
            program_id,
            authority,
        } = self.inproc_mock_data();

        let builder = InstructionBuilder::new(&program_id, 0, 0u16).with_authority(authority);

        builder
    }

    pub async fn execute_handler(
        &self,
        builder: Arc<InstructionBuilder>,
        handler: SimulationHandlerFn,
    ) -> Result<()> {
        self.emulator
            .clone()
            .execute_handler(builder, handler)
            .await
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
                // Update: due to potential of client-side
                // deadlocks, client-side ContainerReferences
                // now replicate data during try_into_container() call.
                Ok(Some(reference.replicate()?))
            }
            None => {
                return Ok(None);
            }
        }
    }

    async fn execute(
        &self,
        authority: &Pubkey,
        instruction: &Instruction,
    ) -> Result<ExecutionResponse> {
        self.emulator.execute(authority, instruction).await
    }

    async fn fund(&self, key: &Pubkey, owner: &Pubkey, lamports: u64) -> Result<()> {
        self.emulator.fund(key, owner, lamports).await
    }

    async fn list(&self) -> Result<AccountDescriptorList> {
        self.emulator.list().await
    }

    async fn configure(&self, config: EmulatorConfig) -> Result<()> {
        self.emulator.configure(config).await
    }
}
