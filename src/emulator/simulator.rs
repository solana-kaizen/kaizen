use std::sync::Arc;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use workflow_allocator::store;
use workflow_allocator::result::Result;
use workflow_allocator::accounts::{ AccountData, AccountDataReference };
use workflow_allocator::builder::{ InstructionBuilder, InstructionBuilderConfig };
use workflow_allocator::context::SimulationHandlerFn;
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
        let inproc_mock_data = Some(InProcMockData::new());

        let simulator = Simulator {//}::new_with_inner(SimulatorInner {
            store,
            emulator,
            inproc_mock_data,
        };

        Ok(simulator)
    }

    
    pub async fn with_mock_accounts(mut self) -> Result<Self> {

        let lamports = crate::utils::u64sol_to_lamports(500_000_000);

        let mock_data = InProcMockData::new();

        // let mock = self.inproc.as_ref().unwrap().expect("inproc mock data not initialized").cloned();
        
        let authority = AccountData::new_static(
            mock_data.authority.clone(),
            mock_data.program_id.clone(),
        ).with_lamports(lamports);
        self.store.store(&Arc::new(AccountDataReference::new(authority))).await?;//.await?;
        // let authority_account_data = Arc::new(RwLock::new(authority_account_data));
        //map.write()?.insert(authority.clone(),authority_account_data);
        
        let identity = AccountData::new_static(
            mock_data.identity.clone(),
            mock_data.program_id.clone(),
        );
        // .with_lamports(lamports);
        self.store.store(&Arc::new(AccountDataReference::new(identity))).await?;//map.write()?.insert(identity.clone(),identity_account_data);
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

    pub fn identity(&self) -> Pubkey {
        self.inproc_mock_data().identity
    }

    pub fn new_instruction_builder_config(
        &self,
    ) -> InstructionBuilderConfig {

        let InProcMockData { program_id, authority, identity} = self.inproc_mock_data();
        // self.inproc.expect("simulator is missing inproc mock data");

        // let inner = self.inner().unwrap();
        let config = InstructionBuilderConfig::new(
            program_id.clone(),
        )
        .with_authority(authority)
        .with_identity(identity);

        config
    }

    pub fn new_instruction_builder(
        &self,
    ) -> InstructionBuilder {
        let InProcMockData { program_id, authority, identity} = self.inproc_mock_data();

        // let inner = self.inner().unwrap();

        let builder = InstructionBuilder::new(
            program_id.clone(),
            0,
            0u16,
        )
        .with_authority(authority)
        .with_identity(identity);

        builder
    }
    
    pub async fn execute_handler(
        &self,
        builder: InstructionBuilder,
        handler: SimulationHandlerFn,
    ) -> Result<()> {
        self.emulator.clone().execute_handler(builder,handler).await
    }
}

#[async_trait]
impl EmulatorInterface for Simulator {
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        self.emulator.lookup(pubkey).await
    }
    async fn execute(
        &self,
        instruction : &Instruction,
    ) -> Result<ExecutionResponse> {
        self.emulator.execute(instruction).await
    }
    async fn fund(
        &self,
        key: &Pubkey,
        owner: &Pubkey,
        lamports: u64
    ) -> Result<()> {
        self.emulator.fund(key,owner,lamports).await
    }
}