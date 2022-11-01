use workflow_allocator::prelude::*;
use std::cell::RefCell;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
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
    CreateCollections(Vec<(u32,Option<u32>)>),
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
        && self.pubkey == other.pubkey
    }
}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct IdentityMeta {
    pub version : u32,
    pub pda_sequence : u64,
    pub reserved_for_future_flags : u32,
    pub referrer : Pubkey,
    pub creation_date : Date,
}

#[container(Containers::Identity)]
pub struct Identity<'info,'refs> {
    pub meta : RefCell<&'info mut IdentityMeta>,
    pub store : SegmentStore<'info,'refs>,
    // ---
    pub records : Array<'info,'refs, IdentityRecord>,
    pub collections : Array<'info,'refs, PubkeyCollectionMeta>,
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

    pub fn referrer(&self) -> Result<Pubkey> {
        let meta = self.meta.try_borrow()?;
        Ok(meta.referrer)
    }

    // Insert Authority Pubkey as IdentityEntry into the entry list
    pub fn try_insert_authority(&mut self, pubkey: &Pubkey)-> Result<()>{
        let record = IdentityRecord {
            data_type : DataType::Authority as u32,
            flags : 0,
            pubkey:pubkey.clone(),
        };
        unsafe { self.records.try_insert(&record) }
    }

    /// Remove entry from the identity entry list
    pub unsafe fn try_remove_entry(&mut self, target : &IdentityRecord) -> Result<()> {
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
        for entry in self.records.iter() {
            if entry.data_type == (DataType::Authority as u32) && entry.pubkey == *pubkey {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn create(ctx:&ContextReference) -> ProgramResult {
        let mut records : Vec<IdentityRecordStore> = Vec::new();
        let mut collection_data_types : Vec<(u32,Option<u32>)> = Vec::new();
        if ctx.instruction_data.len() > 0 {
            match Instr::try_from_slice(&ctx.instruction_data)? {
                Instr::Ops(ops) => {
                    for op in ops {
                        match op {
                            Op::CreateRecords(src) => {
                                records.extend_from_slice(&src);
                            },
                            Op::CreateCollections(src) => {
                                collection_data_types.extend_from_slice(&src);
                            },
                        }
                    }
                }
            }
        }

        let allocation_args = AccountAllocationArgs::new_with_payer(AddressDomain::Authority, AllocationPayer::Authority);
        let proxy = IdentityProxy::try_allocate(ctx, &allocation_args, 0)?;

        let data_len = 
            (1 + records.len()) * std::mem::size_of::<IdentityRecord>() +
            collection_data_types.len() * std::mem::size_of::<PubkeyCollectionMeta>();

        let mut identity = Identity::try_allocate(ctx, &allocation_args, data_len)?;
        identity.init()?;
        proxy.init(identity.pubkey())?;
        identity.try_insert_authority(ctx.authority.key)?;

        for record in records.iter() {
            let record : IdentityRecord = record.into();
            unsafe { identity.records.try_insert(&record)?; }
        }

        for (data_type,container_type) in collection_data_types.iter() {
            let allocation_args = AccountAllocationArgs::new_with_payer(AddressDomain::Authority, AllocationPayer::Authority);
            let collection_meta = unsafe { identity.collections.try_allocate(false)? };

            PubkeyCollectionReference::try_create_with_meta(
                ctx, 
                &allocation_args,
                collection_meta, 
                Some(*data_type),
                *container_type
            )?;
        }

        Ok(())
    }

    pub fn has_collection(&self, data_type : u32) -> bool {
        let collections = self.collections.as_slice_mut();
        for collection_meta in collections.iter_mut() {
            if collection_meta.get_data_type() == data_type {
                return true;
            }
        }
        false
    }

    pub fn locate_collection(&self, data_type : u32) -> Result<PubkeyCollectionReference<'info,'refs>> { //PubkeyCollection<'info,'refs>> {
        let collections = self.collections.as_slice_mut();
        for collection_meta in collections.iter_mut() {
            if collection_meta.get_data_type() == data_type {
                let collection = PubkeyCollectionReference::try_from_meta(collection_meta)?;
                return Ok(collection);
            }
        }
        Err(program_error_code!(ErrorCode::PubkeyCollectionDataTypeNotFound))
    }

    pub fn load_collection(&self, ctx: &ContextReference<'info,'refs,'_,'_>, data_type : u32) -> Result<PubkeyCollectionReference<'info,'refs>> { //PubkeyCollection<'info,'refs>> {
        let mut collection = self.locate_collection(data_type)?;
        collection.try_load(ctx)?;
        Ok(collection)
    }

    pub fn locate_collection_root(&self, data_type : u32) -> Option<Pubkey> {
        for idx in 0..self.collections.len() {
            let collection = &self.collections[idx];
            if collection.get_data_type() == data_type {
                return Some(collection.get_pubkey());
            }
        }
        None
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

        let allocation_args = AccountAllocationArgs::new(AddressDomain::Authority);
        let proxy_account = ctx.try_create_pda(IdentityProxy::initial_data_len(), &allocation_args)?;
        let proxy = IdentityProxy::try_create(proxy_account)?;

        proxy.init(identity.pubkey())?;

        Ok(()) 
    }

}

/// Returns pubkey of the identity proxy given program_id and the authority (wallet address)
pub fn find_identity_proxy_pubkey(program_id: &Pubkey, authority: &Pubkey) -> Result<Pubkey> {
    let bytes = "proxy".as_bytes();
    let seed_suffix = bytes.to_vec();
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
