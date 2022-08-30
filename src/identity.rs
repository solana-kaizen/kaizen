use workflow_allocator::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
// use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use workflow_allocator::container::Containers;

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
}

const ENTRY_FLAG_READONLY : u32 = 0x00000001;
pub const DEFAULT_IDENTITY_RECORDS: usize = 5;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct IdentityEntry {
    pub data_type : u32,
    pub entry_flags : u32,
    pub data_flags : u32,
    pub pubkey : Pubkey,
}

impl PartialEq for IdentityEntry {
    fn eq(&self, other: &Self) -> bool {
        self.data_type == other.data_type
//        self.data_flags == other.data_flags
        && self.pubkey == other.pubkey
    }
}

#[derive(Debug, Copy, Clone)]
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
    #[segment(reserve(LinearStore::<IdentityEntry>::calculate_data_len(5)))]
    pub list : LinearStore<'info,'refs, IdentityEntry>,
}

impl<'info,'refs> std::fmt::Debug for Identity<'info,'refs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Identity {{ {} }}",self.pubkey())?;
        // FIXME dump identity info
        // let inner = self.0.try_lock();
        // match inner {
        //     Some(inner) => {
        //         write!(f, "Cache {{ size: {}, items: {}, capacity: {} }}", inner.size, inner.items, inner.capacity)?;
        //     },
        //     None => {
        //     }
        // }
        Ok(())
    }
}


impl<'info, 'refs> Identity<'info, 'refs> {

    pub fn init(&self) -> Result<()> {
        let mut meta = self.meta.try_borrow_mut()?;
        meta.version = 1;
        // meta.payload_len = 0;
        meta.pda_sequence = 0;
        meta.reserved_for_future_flags = 0;
        Ok(())
    }

    pub fn pda_sequence(&self) -> Result<u64> {
        let meta = self.meta.try_borrow()?;
        Ok(meta.pda_sequence)
    }

    pub fn advance_pda_sequence(&self) -> Result<()> {
        let mut meta = self.meta.try_borrow_mut()?;
        // TODO: handle eventual overflow using second element
        meta.pda_sequence = meta.pda_sequence + 1;
        Ok(())
    }

    /// Insert IdentityEntry into the entry list
    pub fn try_insert_entry(&mut self, entry : &IdentityEntry) -> Result<()> {
        let new_entry = self.list.volatile_try_insert(false)?;
        *new_entry = *entry;
        Ok(())
    }

    // Insert Authority Pubkey as IdentityEntry into the entry list
    pub fn try_add_authority(&mut self, pubkey: &Pubkey)-> Result<()>{
        let entry = IdentityEntry {
            data_type : DataType::Authority as u32,
            entry_flags : 0,
            data_flags : 0,
            pubkey:pubkey.clone()
        };
        self.try_insert_entry(&entry)
    }

    /// Remove entry from the identity entry list
    pub fn try_remove_entry(&mut self, target : &IdentityEntry) -> Result<()> {
        // let entries = self.try_get_entries()?;
        for idx in 0..self.list.len() {
            let entry = self.list.get_at(idx);
            if entry == target {
                if entry.entry_flags & ENTRY_FLAG_READONLY != 0 {
                    return Err(program_error_code!(ErrorCode::ReadOnlyAccess));
                }
                self.list.try_remove_at(idx,true,true)?;
            }
        }

        Err(program_error_code!(ErrorCode::EntryNotFound))
    }

    /// Check if identity has an authority pubkey in the list
    pub fn try_has_authority(&self, pubkey: &Pubkey) -> Result<bool> {
        // let entries = self.try_get_entries()?;
        for entry in self.list.iter() {
            if entry.data_type == (DataType::Authority as u32) && entry.pubkey == *pubkey {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Create a new identity container and the corresponding identity proxy account
    pub fn create(ctx:&Rc<Context>) -> ProgramResult { //Result<()> {
        let allocation_args = AccountAllocationArgs::default();
        let proxy_account = ctx.create_pda(IdentityProxy::initial_data_len(), &allocation_args)?;
        let proxy = IdentityProxy::try_create(proxy_account)?;

        let allocation_args = AccountAllocationArgs::default();
        let identity_account = ctx.create_pda(Identity::initial_data_len(), &allocation_args)?;
        let mut identity = Identity::try_create(identity_account)?;
        
        identity.init()?;
        proxy.init(identity.pubkey())?;
        identity.try_add_authority(ctx.authority.key)?;

        Ok(())
    }

    // TODO: testing sandbox
    /// Register a separate authority with an identity and create a new proxy account for the authority being registered
    pub fn try_register_authority_with_identity(ctx:&Rc<Context>) -> Result<()> {


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
    let seeds = vec![program_id.as_ref(), authority.as_ref(), seed_suffix.as_ref()];
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
    use super::*;

    pub async fn locate_identity_pubkey(transport : &Arc<Transport>, program_id : &Pubkey, authority : &Pubkey) -> Result<Option<Pubkey>> {

        let proxy_pubkey = super::find_identity_proxy_pubkey(program_id, authority)?;
        if let Some(proxy_ref) = transport.lookup(&proxy_pubkey).await? {
            let mut proxy_account_data = proxy_ref.account_data.write().await;
            let proxy_account_info = proxy_account_data.into_account_info();
            let proxy = IdentityProxy::try_load(&proxy_account_info)?;
            let identity_pubkey = proxy.meta.borrow().get_identity_pubkey();
            Ok(Some(identity_pubkey))
        } else {
            Ok(None)
        }
        
    }

    // pub async fn load_identity(program_id: &Pubkey, authority : &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    pub async fn load_identity(program_id: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let transport = workflow_allocator::transport::Transport::global()?;
        let authority = transport.get_authority_pubkey_impl()?;
        if let Some(identity_pubkey) = locate_identity_pubkey(&transport, program_id, &authority).await? {
            Ok(transport.lookup(&identity_pubkey).await?)
        } else {
            Ok(None)
        }
    }

    pub async fn create_identity(program_id: &Pubkey, authority: &Pubkey, interface_id: usize, handler_id : usize) -> Result<Arc<AccountDataReference>> {

        let transport = workflow_allocator::transport::Transport::global()?;

        let builder = InstructionBuilder::new(program_id, interface_id, handler_id as u16)
            .with_authority(authority)
            .with_account_templates_with_custom_suffixes(&["proxy"]) 
            .with_account_templates(1)
            .with_sequence(0u64) 
            .seal()?;

        let instruction : Instruction = builder.try_into()?;
        transport.execute(&instruction).await?;

        let identity = load_identity(program_id).await?;

        match identity {
            Some(identity) => Ok(identity),
            None => Err(workflow_allocator::error!("Error creating identity").into())
        }

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

        let simulator = Simulator::try_new_for_testing()?.with_mock_accounts().await?;

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

        simulator.execute_handler(builder,|ctx:&Rc<Context>| {
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
        
        simulator.execute_handler(builder,|ctx:&Rc<Context>| {
            log_trace!("testing authority presense in the identity");
            let identity = ctx.try_identity()?;
            assert!(identity.try_has_authority(ctx.authority.key)?);
            Ok(())
        }).await?;

        Ok(())
    }
}

