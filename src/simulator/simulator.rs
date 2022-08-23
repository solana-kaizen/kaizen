use std::sync::Arc;
use solana_program::entrypoint::ProcessInstruction;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use workflow_allocator::store;
use workflow_allocator::result::Result;
use workflow_allocator::accounts::{ AccountData, AccountDataReference };
use workflow_allocator::builder::{ InstructionBuilder, InstructionBuilderConfig };

use super::mockdata::InProcMockData;
use super::emulator::Emulator;

use workflow_allocator::context::SimulationHandlerFn;


pub struct Simulator {
    // lamports: u64,

    pub inproc_mock_data : Option<InProcMockData>,

    // #[wasm_bindgen(skip)]
    // #[derivative(Debug="ignore")]
    pub store: Arc<dyn store::Store>, 

    pub emulator: Emulator,
    // MemoryStore,
}

// declare_async_rwlock!(Simulator, SimulatorInner);

// #[wasm_bindgen]
impl Simulator {

    // pub fn store(&self) -> Arc<Store> {
    //     self.store
    // }

    pub fn new(store: Arc<dyn store::Store>) -> Simulator {
        let executor = Emulator::new(store.clone());
        Simulator {
            store,
            emulator: executor,
            inproc_mock_data : None,
        }
    }

    
    pub fn try_new_with_store() -> Result<Simulator> {
        let store = Arc::new(store::MemoryStore::new_local()?);
        let executor = Emulator::new(store.clone());
        let simulator = Simulator {
            store,
            emulator: executor,
            inproc_mock_data : None,
        };
        Ok(simulator)
    }



    // #[wasm_bindgen(constructor)]
    pub fn try_new_for_testing() -> Result<Simulator> {
        let store = Arc::new(store::MemoryStore::new_local()?);
        let executor = Emulator::new(store.clone());


        {
            // let mut store_inner = store.inner_mut().unwrap();
            // let store = self.store.;
            // let identity_account_data = Arc::new(RwLock::new(identity_account_data));
        }

        let inproc_mock_data = Some(InProcMockData::new());

        let simulator = Simulator {//}::new_with_inner(SimulatorInner {
            store,
            emulator: executor,
            inproc_mock_data,
            // lamports,
            // authority,
            // identity,
            // program_id,
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

    
    pub async fn execute_entrypoint(
        &self,
        program_id: &Pubkey,
        accounts: &[AccountMeta],
        instruction_data: &[u8],

        entrypoint: ProcessInstruction,
    ) -> Result<()> {
        self.emulator.execute_entrypoint(program_id, accounts, instruction_data, entrypoint).await
    }

    pub async fn execute_handler(
        &self,
        builder: InstructionBuilder,
        handler: SimulationHandlerFn,
    ) -> Result<()> {
        self.emulator.execute_handler(builder,handler).await
    }


}