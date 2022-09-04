use workflow_allocator::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
// use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use workflow_allocator::container::Containers;
use workflow_allocator::container::*;
use borsh::*;
use serde::*;

#[derive(Meta)]
#[repr(packed)]
pub struct IdentityProxyMeta {
    version : u32,
    identity_pubkey : Pubkey,
}

// #[derive(Debug)]
#[container(Containers::IdentityProxy)]
pub struct IdentityProxy<'info,'refs> {
    pub meta : RefCell<&'info mut IdentityProxyMeta>,
    pub store : SegmentStore<'info,'refs>,
}

impl<'info, 'refs> IdentityProxy<'info, 'refs> {
    pub fn init(&self, pubkey : &Pubkey) -> Result<()> {
        let mut meta = self.meta.borrow_mut();
        meta.version = 1;
        meta.identity_pubkey = *pubkey;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum DataType {
    Authority        = 0x00000001,
    PGPPubkey        = 0x00000002,
    UserTypes        = 0xf0000000,
}

const FLAG_READONLY : u32 = 0x00000001;

#[derive(Meta, Copy, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct IdentityRecordStore {
    pub data_type : u32,
    pub flags : u32,
    pub pubkey : Pubkey,
}

impl Into<IdentityRecord> for &IdentityRecordStore {
    fn into(self) -> IdentityRecord {
        IdentityRecord {
            data_type: self.get_data_type(),
            flags: self.get_flags(),
            pubkey: self.get_pubkey(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Op {
    CreateRecords(Vec<IdentityRecordStore>),
    CreateCollections(Vec<u32>),
    // ChangeEntryFlags(u32),
    // ChangeDataFlags(u32),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Instr {
    Ops(Vec<Op>)
}

impl Instr {
    pub fn get_collection_count(&self) -> usize {
        let mut count = 0;
        match self {
            Instr::Ops(ops) => {
                for op in ops.iter() {
                    if let Op::CreateCollections(vec) = op {
                        count += vec.len();
                    }
                }
            }
        }
        count
    }
}
// impl Instr {
//     pub fn get_records<'instr>(&'instr self) -> Vec<&'instr IdentityRecordStore> {
//         let mut records = Vec::new();
//         match self {
//             Instr::Ops(ops) => {
//                 for op in ops.iter() {
//                     if let Op::CreateRecords(vec) = op {
//                         vec.iter().map(|record| records.push(record));
//                     }
//                 }
//             }
//         }

//         records
//     }
// }

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct IdentityRecord {
    pub data_type : u32,
    pub flags : u32,
    pub pubkey : Pubkey,
}

impl PartialEq for IdentityRecord {
    fn eq(&self, other: &Self) -> bool {
        self.data_type == other.data_type
//        self.data_flags == other.data_flags
        && self.pubkey == other.pubkey
    }
}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct IdentityMeta {
    pub version : u32,
    // pub payload_len : u32,
    pub pda_sequence : u64,
    pub reserved_for_future_flags : u32,
}

#[container(Containers::Identity)]
pub struct Identity<'info,'refs> {
    pub meta : RefCell<&'info mut IdentityMeta>,
    pub store : SegmentStore<'info,'refs>,
    // ---
    #[segment(reserve(Array::<IdentityRecord>::calculate_data_len(5)))]
    pub records : Array<'info,'refs, IdentityRecord>,
    pub collections : Array<'info,'refs, CollectionMeta>,
}

impl<'info,'refs> std::fmt::Debug for Identity<'info,'refs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Identity {{ {} }}",self.pubkey())?;
        Ok(())
    }
}

impl<'info, 'refs> Identity<'info, 'refs> {

    pub fn init(&self) -> Result<()> {
        let mut meta = self.meta.try_borrow_mut()?;
        meta.set_version(1);
        meta.set_pda_sequence(0);
        meta.set_reserved_for_future_flags(0);
        Ok(())
    }

    pub fn pda_sequence(&self) -> Result<u64> {
        let meta = self.meta.try_borrow()?;
        Ok(meta.pda_sequence)
    }

    pub fn advance_pda_sequence(&self) -> Result<()> {
        let mut meta = self.meta.try_borrow_mut()?;
        // TODO: handle eventual overflow using second element
        let seq = meta.get_pda_sequence();
        meta.set_pda_sequence(seq + 1);
        Ok(())
    }

    /// Insert IdentityEntry into the entry list
    // pub fn try_insert_entry(&mut self, entry : &IdentityRecord) -> Result<()> {
    //     let new_entry = self.records.try_allocate_volatile(false)?;
    //     *new_entry = *entry;
    //     Ok(())
    // }

    // Insert Authority Pubkey as IdentityEntry into the entry list
    pub fn try_insert_authority(&mut self, pubkey: &Pubkey)-> Result<()>{
        let record = IdentityRecord {
            data_type : DataType::Authority as u32,
            flags : 0,
            pubkey:pubkey.clone(),
        };
        unsafe { self.records.try_insert(&record) }
        // self.try_insert_entry(&entry)
    }

    /// Remove entry from the identity entry list
    pub unsafe fn try_remove_entry(&mut self, target : &IdentityRecord) -> Result<()> {
        // let entries = self.try_get_entries()?;
        for idx in 0..self.records.len() {
            let entry = self.records.get_at(idx);
            if entry == target {
                if entry.flags & FLAG_READONLY != 0 {
                    return Err(program_error_code!(ErrorCode::ReadOnlyAccess));
                }
                self.records.try_remove_at(idx,true)?;
            }
        }

        Err(program_error_code!(ErrorCode::EntryNotFound))
    }

    /// Check if identity has an authority pubkey in the list
    pub fn try_has_authority(&self, pubkey: &Pubkey) -> Result<bool> {
        // let entries = self.try_get_entries()?;
        for entry in self.records.iter() {
            if entry.data_type == (DataType::Authority as u32) && entry.pubkey == *pubkey {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // pub fn has_collection(&self, pubkey : &Pubkey) -> bool {
    //     for record in self.records.iter() {
    //         if record.data_type == (DataType::Collection as u32) && record.pubkey == *pubkey {
    //             return true;
    //         }
    //     }

    //     false
    // }

    /// Create a new identity container and the corresponding identity proxy account
    pub fn create(ctx:&ContextReference) -> ProgramResult { //Result<()> {

        let mut records : Vec<IdentityRecordStore> = Vec::new();
        let mut collections : Vec<u32> = Vec::new();
        if ctx.instruction_data.len() > 0 {
            match Instr::try_from_slice(&ctx.instruction_data)? {
                Instr::Ops(ops) => {
                    for op in ops {
                        match op {
                            Op::CreateRecords(src) => {
                                records.extend_from_slice(&src);
                            },
                            Op::CreateCollections(src) => {
                                collections.extend_from_slice(&src);
                            },
                        }
                    }
                }
            }
        }
        // let records = instr.get_records();

        let allocation_args = AccountAllocationArgs::default();
        let proxy = IdentityProxy::try_allocate(ctx, &allocation_args, 0)?;

        let data_len = 
            (1 + records.len()) * std::mem::size_of::<IdentityRecord>() +
            collections.len() * std::mem::size_of::<CollectionMeta>();
        let mut identity = Identity::try_allocate(ctx, &allocation_args, data_len)?;
        
        identity.init()?;
        proxy.init(identity.pubkey())?;
        identity.try_insert_authority(ctx.authority.key)?;

        for record in records.iter() {
            let record : IdentityRecord = record.into();
            unsafe { identity.records.try_insert(&record)?; }
        }

        for idx in 0..collections.len() {
            let collection_data_type = collections[idx];
            let allocation_args = AccountAllocationArgs::default();
            let collection_store = CollectionStore::<Pubkey>::try_allocate(ctx, &allocation_args, 0)?;
            collection_store.try_init(collection_data_type)?;
            let collection = unsafe { identity.collections.try_allocate(false)? };
            collection.init(collection_store.pubkey(), collection_data_type);
        }

        Ok(())
    }

    // pub fn locate_collection

    pub fn locate_collection_pubkeys(&self, data_type : u32) -> Option<Vec<Pubkey>> {
        for idx in 0..self.collections.len() {
            let collection = &self.collections[idx];
            if collection.get_data_type() == data_type {
                return Some(vec![collection.get_pubkey()]);
            }
        }
        None
    }

    pub fn locate_collection(&self, ctx:&'refs Rc<Box<Context<'info,'refs,'_,'_>>>, data_type : u32) -> Result<CollectionStore<'info,'refs, Pubkey>> {
        for idx in 0..self.collections.len() {
            let collection = &self.collections[idx];
            if collection.get_data_type() == data_type {
                let pubkey = collection.get_pubkey();
                let collection_account = ctx.locate_index_account(&pubkey).ok_or(program_error_code!(ErrorCode::CollectionAccountNotFound))?;
                let collection_store = CollectionStore::<Pubkey>::try_load(collection_account)?;
                return Ok(collection_store);
            }
        }
        Err(program_error_code!(ErrorCode::CollectionDataTypeNotFound))
    }

    // TODO: testing sandbox
    /// Register a separate authority with an identity and create a new proxy account for the authority being registered
    pub fn try_register_authority_with_identity(ctx:&ContextReference) -> Result<()> {


        let identity = ctx.try_identity()?;

        // the incoming PDA should have 0 sequence derivation from the target wallet
        let _foreign_wallet_address = if ctx.handler_accounts.len() != 1 {
            return Err(program_error_code!(ErrorCode::IdentityMissingForeignAuthority));
        } else {
            &ctx.handler_accounts[0]
        };

        // TODO: generate PDA dynamically or validate incoming PDA
        // ! WARNING this derivation is not correct (testing) 
        let allocation_args = AccountAllocationArgs::default();
        let proxy_account = ctx.create_pda(IdentityProxy::initial_data_len(), &allocation_args)?;
        let proxy = IdentityProxy::try_create(proxy_account)?;

        proxy.init(identity.pubkey())?;

        Ok(()) 
    }

}

/// Returns pubkey of the identity proxy given program_id and the authority (wallet address)
pub fn find_identity_proxy_pubkey(program_id: &Pubkey, authority: &Pubkey) -> Result<Pubkey> {
    let bytes = "proxy".as_bytes();
    let seed_suffix = bytes.to_vec();
    // let seeds = vec![program_id.as_ref(), authority.as_ref(), seed_suffix.as_ref()];
    let seeds = vec![authority.as_ref(), seed_suffix.as_ref()];
    let (address, _bump_seed) = Pubkey::find_program_address(
        &seeds[..],
        program_id
    );
    Ok(address)
}

declare_handlers!(Identity::<'info,'refs>,[
    Identity::create
]);


#[cfg(not(target_arch = "bpf"))]
pub mod client {
    use crate::emulator::Simulator;

    use super::*;

    pub async fn locate_identity_pubkey(transport : &Arc<Transport>, program_id : &Pubkey, authority : &Pubkey) -> Result<Option<Pubkey>> {

        let proxy_pubkey = super::find_identity_proxy_pubkey(program_id, authority)?;
        log_trace!("proxy_pubkey: {}", proxy_pubkey);
        if let Some(proxy_ref) = transport.lookup(&proxy_pubkey).await? {
            log_trace!("got proxy account {}", proxy_pubkey);

            let mut proxy_account_data = proxy_ref.account_data.lock()?;
            let proxy_account_info = proxy_account_data.into_account_info();
            let proxy = IdentityProxy::try_load(&proxy_account_info)?;
            let identity_pubkey = proxy.meta.borrow().get_identity_pubkey();
            log_trace!("got identity pubkey {}", identity_pubkey);

            Ok(Some(identity_pubkey))
        } else {
            log_trace!("can not lookup proxy account {}", proxy_pubkey);
            Ok(None)
        }
        
    }

    // pub async fn load_identity(program_id: &Pubkey, authority : &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    pub async fn load_identity(program_id: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let transport = workflow_allocator::transport::Transport::global()?;
        let authority = transport.get_authority_pubkey()?;
        log_trace!("found authority: {}", authority);
        if let Some(identity_pubkey) = locate_identity_pubkey(&transport, program_id, &authority).await? {
            log_trace!("found identity pubkey: {}", identity_pubkey);
            Ok(transport.lookup(&identity_pubkey).await?)
        } else {
            log_trace!("ERROR: identity pubkey not found!");
            Ok(None)
        }
    }

    pub async fn create_identity(
        program_id: &Pubkey,
        authority: &Pubkey,
        interface_id: usize,
        handler_id : usize,
        instructions : Instr,
    ) -> Result<Arc<AccountDataReference>> {

        let instruction_data = instructions.try_to_vec()?;

        let transport = workflow_allocator::transport::Transport::global()?;

        let builder = InstructionBuilder::new(program_id, interface_id, handler_id as u16)
            .with_authority(authority)
            .with_account_templates_with_custom_suffixes(&["proxy"]) 
            .with_account_templates(1 + instructions.get_collection_count())
            .with_sequence(0u64) 
            .with_instruction_data(&instruction_data)
            .seal()?;

        let instruction : Instruction = builder.try_into()?;
        transport.execute(&instruction).await?;

        let identity = load_identity(program_id).await?;

        match identity {
            Some(identity) => Ok(identity),
            None => Err(workflow_allocator::error!("Error creating identity").into())
        }

    }


    pub async fn create_identity_for_unit_tests(
        // transport : &Arc<Transport>,
        simulator : &Simulator,
        authority : &Pubkey,
        program_id : &Pubkey,

    ) -> Result<Pubkey> {

        // Identity::min
        // AccountData::new_static_with_size(key,owner, )
        // let emulator = transport.emulator();
        // let simulator = emulator.clone().downcast_arc::<Simulator>().unwrap();

        let config = InstructionBuilderConfig::new(program_id.clone())
            .with_authority(authority)
            .with_sequence(0u64);

        let builder = InstructionBuilder::new_with_config_for_testing(&config)
            .with_account_templates_with_custom_suffixes(&["proxy"])
            .with_account_templates(2)
            .seal()?;

        let accounts = builder.template_accounts();
        // let proxy = accounts[0].clone(); // PDA0
        let identity = accounts[1].clone();

    
        simulator.execute_handler(builder,|ctx:&ContextReference| {
            log_trace!("create identity");
            Identity::create(ctx)?;
            Ok(())
        }).await?;

        Ok(identity.pubkey)


    }

}



#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    // use async_std;
    use super::*;
    // use workflow_allocator::prelude::*;
    use workflow_allocator::emulator::Simulator;
    use workflow_allocator::result::Result;
 
    #[async_std::test]
    async fn identity_init() -> Result<()> {
        workflow_allocator::container::registry::init()?;

        let program_id = generate_random_pubkey();
        let simulator = Simulator::try_new_for_testing()?.with_mock_accounts(program_id).await?;

        let config = InstructionBuilderConfig::new(simulator.program_id())
            .with_authority(&simulator.authority())
        //     .with_identity(&identity)
            .with_sequence(0u64);

        let builder = InstructionBuilder::new_with_config_for_testing(&config)
            .with_account_templates_with_custom_suffixes(&["proxy"]) // [proxy, identity]
            .with_account_templates(1) // [proxy, identity]
            // .with_account_templates(2) // [proxy, identity]
            .seal()?;

        let accounts = builder.template_accounts();
        let proxy = accounts[0].clone(); // PDA0
        let identity = accounts[1].clone();

        simulator.execute_handler(builder,|ctx:&ContextReference| {
            log_trace!("create identity");
            Identity::create(ctx)?;
            Ok(())
        }).await?;


        let proxy_pubkey = find_identity_proxy_pubkey(&simulator.program_id(), &simulator.authority())?;
        log_trace!("validating proxy pubkey: {} vs {}", proxy.pubkey,proxy_pubkey);
        assert_eq!(proxy.pubkey, proxy_pubkey);


        let config = config.with_identity(&identity.pubkey);

        // load test container
        let builder = InstructionBuilder::new_with_config_for_testing(&config)
            // .with_identity
            // .with_handler_accounts(&[
            //     test_container_account
            // ])
            //.with_account_templates(1)
            .seal()?;
        
        simulator.execute_handler(builder,|ctx:&ContextReference| {
            log_trace!("testing authority presense in the identity");
            let identity = ctx.try_identity()?;
            assert!(identity.try_has_authority(ctx.authority.key)?);
            Ok(())
        }).await?;

        Ok(())
    }
}

