//!
//! [`Instruction`] builder, used to dynamically generate Solana OS program instructions.
//!

use crate::accounts::{Access, IsSigner, SeedBump, SeedSuffix};
use crate::error;
use crate::payload::Payload;
use crate::result::*;
use crate::sequencer::Sequencer;
use crate::user::User;
use std::sync::{Mutex, MutexGuard};
// use crate::transport::load_container;
use kaizen::address::AddressDomain;
use kaizen::container::{
    AccountAggregatorInterface, AsyncAccountAggregatorInterface,
    AsyncPdaCollectionAccessorInterface, AsyncPdaCollectionCreatorInterface,
    PdaCollectionAccessorInterface, PdaCollectionCreatorInterface,
};
use kaizen::context::{HandlerFn, HandlerFnCPtr};
use kaizen::identity::program::Identity;
use kaizen::prelude::*;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use workflow_log::{log_warning, style};

pub enum Gather {
    Authority,
    Identity,
    All,
}

pub fn find_interface_id(program_fn: HandlerFn, handlers: &[HandlerFn]) -> usize {
    handlers
        .iter()
        .position(|&hfn| std::ptr::eq(hfn as HandlerFnCPtr, program_fn as HandlerFnCPtr))
        .expect("handler is not registered")
}

// ~~~

// this is currently for testing only
#[derive(Debug)]
pub struct InstructionBuilderConfig {
    pub authority: Option<AccountMeta>,
    pub identity: Option<AccountMeta>,
    pub program_id: Pubkey,
    pub suffix_seed_seq: Option<Arc<Mutex<u64>>>,
    pub sequencer: Option<Sequencer>,
}

impl InstructionBuilderConfig {
    pub fn new(program_id: Pubkey) -> Self {
        InstructionBuilderConfig {
            authority: None,
            identity: None,
            program_id,
            suffix_seed_seq: None,
            sequencer: None,
        }
    }

    pub fn with_authority(mut self, authority: &Pubkey) -> Self {
        self.authority = Some(AccountMeta::new(*authority, true));
        self
    }

    pub fn with_identity(mut self, identity: &Pubkey) -> Self {
        self.identity = Some(AccountMeta::new(*identity, false));
        self
    }

    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.suffix_seed_seq = Some(Arc::new(Mutex::new(sequence)));
        self
    }

    pub fn with_sequencer(mut self, sequencer: &Sequencer) -> Self {
        self.sequencer = Some(sequencer.clone());
        self
    }
}

pub type GenericTemplateAccountDescriptor = (IsSigner, Access, AddressDomain, SeedSuffix);
pub type CollectionTemplateAccountDescriptor = (AccountMeta, SeedBump);

/// # Helper for creating a structured program Instruction
/// Structured program instructions are used by the Workflow Allocator framework.
///
/// Accounts and instruction data are serialized into the Solana instruction buffer
/// and deserialized into the [crate::context::Context] during the program execution.
#[derive(Debug)]
pub struct Inner {
    /// Operation authority (user wallet account)
    pub authority: Option<AccountMeta>,
    /// Operation identity (user identity account)
    pub identity: Option<AccountMeta>,
    /// Program id for instruction execution
    pub program_id: Pubkey,
    /// Program interface receiving the instruction (should be `SomeStruct::program`)
    pub interface_id: u16,
    /// Program handler functoin receiving the instruction (should be `SomeStruct::handler_fn`)
    pub handler_id: u16,

    /// List of included system accounts (if templates exist System Program account is added automatically)
    system_accounts: Vec<AccountMeta>,
    /// List of accounts used for token operations (SOL/SPL token accounts)
    token_accounts: Vec<AccountMeta>,
    /// List of accounts used for indexing purposes (BTree accounts)
    index_accounts: Vec<AccountMeta>,
    /// List of accounts used by collections
    collection_accounts: Vec<AccountMeta>,
    /// List of accounts intended for the handler function (specific to the current operation)
    handler_accounts: Vec<AccountMeta>,
    /// Instruction buffer received by the handler function (excludes the [Payload] header)
    handler_instruction_data: Vec<u8>,

    /// List of template accounts used for PDA creation (The instruction buffer Payload header will  contain the corresponding PDA seed information)
    generic_template_accounts: Vec<AccountMeta>,
    /// List of seeds used for PDA creation
    generic_template_address_data: Vec<Vec<u8>>,
    /// Chunk of serialized PDA seed data (used for Instruction Buffer assembly)
    generic_template_instruction_data: Vec<u8>,

    /// List of collection accounts used for PDA creation
    collection_template_accounts: Vec<AccountMeta>,
    /// List of bumps used for PDA creation
    collection_template_address_data: Vec<Vec<u8>>,
    /// Chunk of serialized PDA seed data (used for Instruction Buffer assembly)
    collection_template_instruction_data: Vec<u8>,

    // signals if the Instruction Builder has been sealed (meaning the final Instruction Buffer has been integrated)
    is_sealed: bool,
    // starting value for the PDA seed sequence used in this operation
    suffix_seed_seq: u64,

    sequencer: Option<Sequencer>,

    // TODO
    generic_template_account_descriptors: Vec<GenericTemplateAccountDescriptor>,
    collection_template_account_descriptors: Vec<CollectionTemplateAccountDescriptor>,

    // Reference to an external seed that should be used during PDA creation (allowing PDA seed
    // sequence value to be tracked in an external object such as `InstructionBufferConfig`)
    // track_suffix_seed_seq : Option<Rc<RefCell<u64>>>,
    track_suffix_seed_seq: Option<Arc<Mutex<u64>>>,
}

pub struct InstructionBuilder {
    inner: Arc<Mutex<Inner>>,
}

impl InstructionBuilder {
    pub fn inner(&self) -> MutexGuard<Inner> {
        self.inner
            .lock()
            .expect("Unable to lock instruction builder")
    }

    fn with_inner_with_result<F: FnMut(MutexGuard<Inner>) -> Result<()>>(
        self: Arc<Self>,
        mut cb: F,
    ) -> Result<Arc<Self>> {
        cb(self.inner())?;
        Ok(self)
    }

    fn with_inner<F: FnMut(MutexGuard<Inner>)>(self: Arc<Self>, mut cb: F) -> Arc<Self> {
        cb(self.inner());
        self
    }

    pub fn is_sealed(&self) -> bool {
        self.inner().is_sealed
    }

    pub fn program_id(&self) -> Pubkey {
        self.inner().program_id
    }

    pub fn identity_pubkey(&self) -> Option<Pubkey> {
        self.inner().identity.as_ref().map(|m| m.pubkey) //.clone()
                                                         // match self.inner().identity.as_ref()
    }

    /// Creates a default [InstructionBuilder] copying Authority, optional Identity and an optional PDA seed sequence
    /// from the InstructionBuilderConfig
    ///
    /// The receiver Interface and Program Instruction are set to zero.
    ///
    /// This function is used for testing purposes.
    pub fn new_with_config_for_testing(
        config: &InstructionBuilderConfig,
    ) -> Arc<InstructionBuilder> {
        Self::new_with_config(config, 0, 0u16)
    }

    /// Creates a default InstructionBuilder copying Authority, optional Identity and an optional PDA seed sequence
    /// from the InstructionBuilderConfig
    pub fn new_with_config<T: Into<u16>>(
        config: &InstructionBuilderConfig,
        interface_id: usize,
        program_instruction: T,
    ) -> Arc<InstructionBuilder> {
        // let track_suffix_seed_seq = match config.suffix_seed_seq {
        //     Some(ref) =>
        // }

        let track_suffix_seed_seq = config.suffix_seed_seq.clone();
        let suffix_seed_seq = match &track_suffix_seed_seq {
            None => 0u64,
            Some(suffix_seed_seq_refcell) => *suffix_seed_seq_refcell.lock().unwrap(),
        };

        let inner = Inner {
            authority: config.authority.clone(),
            identity: config.identity.clone(),
            program_id: config.program_id,

            interface_id: interface_id as u16,
            handler_id: program_instruction.into(),
            system_accounts: Vec::new(),
            token_accounts: Vec::new(),
            index_accounts: Vec::new(),
            collection_accounts: Vec::new(),
            handler_accounts: Vec::new(),
            handler_instruction_data: Vec::new(),
            generic_template_accounts: Vec::new(),
            generic_template_address_data: Vec::new(),
            generic_template_instruction_data: Vec::new(),
            collection_template_accounts: Vec::new(),
            collection_template_address_data: Vec::new(),
            collection_template_instruction_data: Vec::new(),

            is_sealed: false,
            suffix_seed_seq, // : 0u64,
            sequencer: config.sequencer.clone(),
            generic_template_account_descriptors: Vec::new(),
            collection_template_account_descriptors: Vec::new(),

            track_suffix_seed_seq,
        };

        Arc::new(InstructionBuilder {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub fn new_for_testing(program_id: &Pubkey) -> Arc<InstructionBuilder> {
        Self::new(program_id, 0, 0u16)
    }

    pub fn new<T: Into<u16>>(
        program_id: &Pubkey,
        interface_id: usize,
        handler_id: T,
    ) -> Arc<InstructionBuilder> {
        let inner = Inner {
            authority: None,
            identity: None,
            program_id: *program_id,

            interface_id: interface_id as u16,
            handler_id: handler_id.into(),
            system_accounts: Vec::new(),
            token_accounts: Vec::new(),
            index_accounts: Vec::new(),
            collection_accounts: Vec::new(),
            handler_accounts: Vec::new(),
            handler_instruction_data: Vec::new(),
            generic_template_accounts: Vec::new(),
            generic_template_address_data: Vec::new(),
            generic_template_instruction_data: Vec::new(),
            collection_template_accounts: Vec::new(),
            collection_template_address_data: Vec::new(),
            collection_template_instruction_data: Vec::new(),

            is_sealed: false,
            suffix_seed_seq: 0u64,
            sequencer: None,
            generic_template_account_descriptors: Vec::new(),
            collection_template_account_descriptors: Vec::new(),

            track_suffix_seed_seq: None,
        };

        Arc::new(InstructionBuilder {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    // pub fn generic_template_accounts<'this>(&'this self) -> &'this Vec<AccountMeta> {
    pub fn generic_template_accounts(&self) -> Vec<AccountMeta> {
        // &self.generic_template_accounts
        self.inner().generic_template_accounts.clone()
    }

    // pub fn generic_template_account_at<'this>(&'this self, idx : usize) -> &'this AccountMeta {
    pub fn generic_template_account_at(&self, idx: usize) -> AccountMeta {
        self.inner().generic_template_accounts[idx].clone()
    }

    pub fn generic_template_pubkey_at(&self, idx: usize) -> Pubkey {
        self.inner().generic_template_accounts[idx].pubkey
    }

    pub fn collection_template_accounts(&self) -> Vec<AccountMeta> {
        // &self.generic_template_accounts
        self.inner().generic_template_accounts.clone()
    }

    pub fn collection_template_account_at(&self, idx: usize) -> AccountMeta {
        self.inner().collection_template_accounts[idx].clone()
    }

    pub fn collection_template_pubkey_at(&self, idx: usize) -> Pubkey {
        self.inner().collection_template_accounts[idx].pubkey
    }

    // pub fn template_accounts(&self) -> Vec<AccountMeta> {
    //     self.template_accounts.clone()
    // }

    pub fn payload(&self) -> Payload {
        let inner = self.inner();

        let collection_data_offset =
            std::mem::size_of::<Payload>() + inner.generic_template_instruction_data.len();

        let instruction_data_offset =
            collection_data_offset + inner.collection_template_instruction_data.len();
        // std::mem::size_of::<Payload>()
        // + self.generic_template_instruction_data.len()

        //  log_trace!("* * * INSTRUCTION DATA OFFSET {}", instruction_data_offset);

        let mut flags: u16 = 0;
        if inner.identity.is_some() {
            flags |= crate::payload::PAYLOAD_HAS_IDENTITY_ACCOUNT;
        }

        Payload {
            version: Payload::version(),

            flags,

            system_accounts_len: inner.system_accounts.len() as u8,
            token_accounts_len: inner.token_accounts.len() as u8,
            index_accounts_len: inner.index_accounts.len() as u8,
            collection_accounts_len: inner.collection_accounts.len() as u8,
            generic_template_accounts_len: inner.generic_template_accounts.len() as u8,
            collection_template_accounts_len: inner.collection_template_accounts.len() as u8,
            // NOTE: handler accounts are the remaining accounts supplied to the program
            collection_data_offset: collection_data_offset as u16,
            instruction_data_offset: instruction_data_offset as u16,

            interface_id: inner.interface_id,
            handler_id: inner.handler_id,
        }
    }

    pub fn with_system_program_account(self: Arc<Self>) -> Arc<Self> {
        self.inner()
            .system_accounts
            .push(AccountMeta::new(solana_sdk::system_program::id(), false));
        self
    }

    pub fn with_system_accounts(self: Arc<Self>, system_accounts: &[AccountMeta]) -> Arc<Self> {
        self.inner()
            .system_accounts
            .extend_from_slice(system_accounts);
        self
    }

    pub fn with_token_accounts(self: Arc<Self>, token_accounts: &[AccountMeta]) -> Arc<Self> {
        self.inner()
            .token_accounts
            .extend_from_slice(token_accounts);
        self
    }

    pub fn with_index_accounts(self: Arc<Self>, index_accounts: &[AccountMeta]) -> Arc<Self> {
        self.inner()
            .index_accounts
            .extend_from_slice(index_accounts);
        self
    }

    pub fn with_collection_accounts(self: Arc<Self>, index_accounts: &[AccountMeta]) -> Arc<Self> {
        self.inner()
            .index_accounts
            .extend_from_slice(index_accounts);
        self
    }

    pub fn with_handler_accounts(self: Arc<Self>, handler_accounts: &[AccountMeta]) -> Arc<Self> {
        self.inner()
            .handler_accounts
            .extend_from_slice(handler_accounts);
        self
    }

    pub fn with_instruction_data(self: Arc<Self>, instruction_data: &[u8]) -> Arc<Self> {
        self.inner()
            .handler_instruction_data
            .extend(instruction_data);
        self
    }

    pub fn with_user(self: Arc<Self>, user: &User) -> Arc<Self> {
        let (authority, identity, sequencer) =
            user.builder_args().expect("User record is not ready");

        log_trace!("authority: {:?}, identity: {:?}", authority, identity);

        self.with_authority(&authority)
            .with_identity(&identity)
            .with_sequencer(&sequencer)
    }

    pub fn with_authority(self: Arc<Self>, authority: &Pubkey) -> Arc<Self> {
        self.inner().authority = Some(AccountMeta::new(*authority, true));
        self
    }

    pub fn with_identity(self: Arc<Self>, identity: &Pubkey) -> Arc<Self> {
        self.inner().identity = Some(AccountMeta::new(*identity, false));
        self
    }

    pub fn with_sequencer(self: Arc<Self>, sequencer: &Sequencer) -> Arc<Self> {
        self.with_inner(|mut inner| {
            inner.suffix_seed_seq = sequencer.get();
            inner.sequencer = Some(sequencer.clone());
        })
    }

    pub fn with_sequence(self: Arc<Self>, seq: u64) -> Arc<Self> {
        self.with_inner(|mut inner| {
            assert!(inner.track_suffix_seed_seq.is_none());
            inner.suffix_seed_seq = seq;
        })
    }

    pub fn sequence(&self) -> u64 {
        self.inner().suffix_seed_seq
    }

    // fn encode_template_instruction_data(&self) -> Vec<u8> {
    fn encode_template_instruction_data(data: &[Vec<u8>]) -> Vec<u8> {
        let mut template_address_data = Vec::new();
        // for data in self.generic_template_address_data.iter() {
        for data in data.iter() {
            let bytes = data.to_vec();
            let len = bytes.len();
            // log_trace!(" - - - > processing address data len: {}", len);
            // TODO remove address check
            assert!(len < 0xff);
            // log_trace!("{}:{}",style("TPL DATA PACKET LEN:").white().on_red(),len);
            template_address_data.push(len as u8);
            // log_trace!("*** PACKAGING LENGTH: {:?}", len as u8);
            // log_trace!("*** PACKAGING BYTES: {:?}", bytes);
            template_address_data.extend(bytes);
        }
        template_address_data
    }

    pub fn instruction_data(&self) -> Vec<u8> {
        let inner = self.inner();
        let mut data = Vec::new();
        data.extend(&inner.generic_template_instruction_data);
        data.extend(&inner.collection_template_instruction_data);
        data.extend(&inner.handler_instruction_data);
        data
    }

    // pub fn try_accounts(self : &Arc<Self>) -> Result<Vec<AccountMeta>> {
    pub fn try_accounts(&self) -> Result<Vec<AccountMeta>> {
        let inner = self.inner();

        let mut vec = Vec::new();

        if let Some(authority) = &inner.authority {
            vec.push(authority.clone());
        }

        if let Some(identity) = &inner.identity {
            vec.push(identity.clone());

            if inner.authority.is_none() {
                return Err(error!("InstructionBuilder::try_account(): missing authority - required when using identity"));
            }
        }

        vec.extend_from_slice(&inner.system_accounts);
        vec.extend_from_slice(&inner.token_accounts);
        vec.extend_from_slice(&inner.index_accounts);
        vec.extend_from_slice(&inner.collection_accounts);
        vec.extend_from_slice(&inner.generic_template_accounts);
        vec.extend_from_slice(&inner.collection_template_accounts);
        vec.extend_from_slice(&inner.handler_accounts);
        Ok(vec)
    }

    pub fn with_generic_account_templates(self: Arc<Self>, n: usize) -> Arc<Self> {
        self.with_inner(|mut inner| {
            for _ in 0..n {
                inner.generic_template_account_descriptors.push((
                    IsSigner::NotSigner,
                    Access::Write,
                    AddressDomain::Default,
                    SeedSuffix::Sequence,
                ))
            }
        })
    }

    // pub fn with_account_templates_with_custom_suffixes(self: Arc<Self>, suffixes : &[&str]) -> Arc<Self> {
    pub fn with_generic_account_templates_with_custom_suffixes(
        self: Arc<Self>,
        suffixes: &[&[u8]],
    ) -> Arc<Self> {
        self.with_inner(|mut inner| {
            for s in suffixes.iter() {
                inner.generic_template_account_descriptors.push((
                    IsSigner::NotSigner,
                    Access::Write,
                    AddressDomain::Default,
                    SeedSuffix::Custom(s.to_vec()),
                ))
            }
        })
    }

    // pub fn with_account_templates_with_custom_domains_and_suffixes(self: Arc<Self>, suffixes : &[(AddressDomain,&str)]) -> Arc<Self> {
    // pub fn with_account_templates_with_custom_seeds(self: Arc<Self>, suffixes : &[(AddressDomain,&str)]) -> Arc<Self> {
    pub fn with_generic_account_templates_with_seeds(
        self: Arc<Self>,
        suffixes: &[(AddressDomain, &[u8])],
    ) -> Arc<Self> {
        self.with_inner(|mut inner| {
            for (domain, suffix) in suffixes {
                inner.generic_template_account_descriptors.push((
                    IsSigner::NotSigner,
                    Access::Write,
                    domain.clone(),
                    SeedSuffix::Custom(suffix.to_vec()),
                ))
            }
        })
    }

    pub fn with_generic_account_templates_with_custom_suffixes_prefixed(
        self: Arc<Self>,
        prefix: &[u8],
        suffixes: &[&[u8]],
    ) -> Arc<Self> {
        self.with_inner(|mut inner| {
            for s in suffixes.iter() {
                let mut suffix = prefix.to_vec();
                suffix.extend_from_slice(s);
                inner.generic_template_account_descriptors.push((
                    IsSigner::NotSigner,
                    Access::Write,
                    AddressDomain::Default,
                    SeedSuffix::Custom(suffix.to_vec()),
                ))
            }
        })
    }

    pub fn with_generic_custom_account_templates_and_seeds(
        self: Arc<Self>,
        templates: &[(IsSigner, Access, AddressDomain, SeedSuffix)],
    ) -> Arc<Self> {
        self.inner()
            .generic_template_account_descriptors
            .extend(templates.to_vec());
        self
    }

    pub async fn with_collection_template<A>(
        self: Arc<Self>,
        pda_collection: &A,
    ) -> Result<Arc<Self>>
    where
        A: PdaCollectionCreatorInterface,
    {
        self.with_collection_templates(pda_collection, 1).await
    }

    pub async fn with_collection_templates<A>(
        self: Arc<Self>,
        pda_collection: &A,
        number_of_accounts: usize,
    ) -> Result<Arc<Self>>
    where
        A: PdaCollectionCreatorInterface,
    {
        for _ in 0..number_of_accounts {
            let collection_account_descriptors = pda_collection
                .creator(&self.program_id(), 1)?
                .writable_accounts_meta()
                .await?;
            self.inner()
                .collection_template_account_descriptors
                .extend_from_slice(&collection_account_descriptors);
        }
        Ok(self)
    }

    pub async fn with_collection_index<A>(
        self: Arc<Self>,
        pda_collection_accessor: &A,
        idx: usize,
    ) -> Result<Arc<Self>>
    where
        A: PdaCollectionAccessorInterface,
    {
        self.with_collection_index_range(pda_collection_accessor, idx..idx + 1)
            .await
    }

    pub async fn with_collection_index_range<A>(
        self: Arc<Self>,
        pda_collection_accessor: &A,
        range: std::ops::Range<usize>,
    ) -> Result<Arc<Self>>
    where
        A: PdaCollectionAccessorInterface,
    {
        let list = pda_collection_accessor
            .accessor(&self.program_id(), range)?
            .writable_accounts_meta()
            .await?;
        Ok(self.with_collection_accounts(&list))
    }

    #[inline(always)]
    pub async fn with_writable_account_aggregator<A>(
        self: Arc<Self>,
        aggregator: &A,
    ) -> Result<Arc<Self>>
    where
        A: AccountAggregatorInterface,
    {
        let list = aggregator
            .aggregator()?
            .writable_account_metas(None)
            .await?;
        Ok(self.with_index_accounts(&list))
    }

    #[inline(always)]
    pub async fn with_readonly_account_aggregator<A>(
        self: Arc<Self>,
        aggregator: &A,
    ) -> Result<Arc<Self>>
    where
        A: AccountAggregatorInterface,
    {
        let list = aggregator
            .aggregator()?
            .readonly_account_metas(None)
            .await?;
        Ok(self.with_index_accounts(&list))
    }

    #[inline(always)]
    pub async fn with_account_aggregators<A>(
        self: Arc<Self>,
        accessors: &[(bool, A)],
    ) -> Result<Arc<Self>>
    where
        A: AccountAggregatorInterface,
    {
        let mut list = Vec::new();
        for (writable, accessor) in accessors.iter() {
            let aggregator_list = if *writable {
                accessor.aggregator()?.writable_account_metas(None).await?
            } else {
                accessor.aggregator()?.readonly_account_metas(None).await?
            };
            list.extend_from_slice(&aggregator_list);
        }
        Ok(self.with_index_accounts(&list))
    }

    #[inline(always)]
    pub async fn with_async_account_aggregators<A>(
        self: Arc<Self>,
        aggregators: &[(bool, Arc<A>)],
    ) -> Result<Arc<Self>>
    where
        A: AsyncAccountAggregatorInterface,
    {
        let mut list = Vec::new();
        for (writable, aggregator) in aggregators.iter() {
            let aggregator_list = if *writable {
                aggregator.writable_account_metas(None).await?
            } else {
                aggregator.readonly_account_metas(None).await?
            };
            list.extend_from_slice(&aggregator_list);
        }
        Ok(self.with_index_accounts(&list))
    }

    // #[inline(always)]
    // pub async fn with_writable_account_aggregator_for_key<A>(self, key: &<A as AccountAggregator>::Key, aggregator : &A) -> Result<Self>
    // where A: AccountAggregator
    // {
    //     let list = aggregator.writable_account_metas(Some(key)).await?;
    //     Ok(self.with_index_accounts(&list))
    // }

    // #[inline(always)]
    // pub async fn with_readonly_account_aggregator_for_key<A>(self, key: &<A as AccountAggregator>::Key, aggregator : &A) -> Result<Self>
    // where A: AccountAggregator
    // {
    //     let list = aggregator.readonly_account_metas(Some(key)).await?;
    //     Ok(self.with_index_accounts(&list))
    // }

    pub async fn with_identity_collections(
        self: Arc<Self>,
        collections: &[(bool, u32)],
    ) -> Result<Arc<Self>> {
        match self.identity_pubkey().as_ref() {
            Some(pubkey) => {
                // TODO handle processing of concurrent requests!
// Ok(self)
                let identity = load_reference(pubkey).await?;
                match identity {
                    Some(identity) => {

                        let aggregators = {
                            let identity = identity.try_into_container::<Identity>()?;
                            let mut aggregators = Vec::new();
                            for (writable,data_type) in collections.iter() {
                                let collection = identity.locate_collection(*data_type)?;
                                aggregators.push((*writable,collection.aggregator()?));
                            }
                            aggregators

                        };

                        Ok(self.with_async_account_aggregators(&aggregators).await?)
                    },
                    None => {
                        Err(error!("InstructionBuilder::with_identity_collection() missing on-chain identity account"))
                    }
                }
            },
            None => {
                Err(error!("InstructionBuilder::with_identity_collection() missing identity record (please use with_identity())"))
            }
        }
    }

    pub fn seal(self: Arc<Self>) -> Result<Arc<Self>> {
        self.with_inner_with_result(|mut inner| {
            // let mut inner = self.inner();

            if inner.is_sealed {
                return Err(error!(
                    "InstructionBuilder::seal(): has already been invoked"
                ));
            }
            inner.is_sealed = true;

            if !inner.generic_template_account_descriptors.is_empty() {
                if let Some(sequencer) = &inner.sequencer {
                    sequencer.advance(inner.generic_template_account_descriptors.len());
                    // log_trace!("Advancing instruction builder sequencer {}", sequencer.get());
                } else {
                    log_warning!(
                        "{}",
                        style("\nWARNING: InstructionBuilder::seal()? - missing sequencer!\n")
                            .red()
                    );
                }
            } else if inner.collection_template_account_descriptors.is_empty() {
                // if both template sets are empty, there is nothing to do
                return Ok(());
            }

            // if we have templates, automatically include system account if not added by the user
            if !inner
                .system_accounts
                .iter()
                .any(|meta| meta.pubkey == solana_sdk::system_program::id())
            {
                inner
                    .system_accounts
                    .insert(0, AccountMeta::new(solana_sdk::system_program::id(), false));
            }

            let generic_template_account_descriptors =
                inner.generic_template_account_descriptors.clone();
            for (is_signer, is_writable, domain, seed_suffix) in
                generic_template_account_descriptors.iter()
            {
                let domain_seed =
                    domain.get_seed(inner.authority.as_ref(), inner.identity.as_ref())?;

                let mut seeds = vec![domain_seed.as_slice()];

                let seed_suffix = match seed_suffix {
                    SeedSuffix::Blank => {
                        vec![]
                    }
                    SeedSuffix::Sequence => {
                        inner.suffix_seed_seq += 1;
                        let bytes: [u8; 8] = inner.suffix_seed_seq.to_le_bytes();
                        // unsafe { std::mem::transmute(inner.suffix_seed_seq.to_le()) };
                        let mut bytes = bytes.to_vec();

                        while let 0 = bytes[bytes.len() - 1] {
                            bytes.pop();
                        }
                        // loop {
                        //     match bytes[bytes.len() - 1] {
                        //         0 => {
                        //             bytes.pop();
                        //         }
                        //         _ => break,
                        //     }
                        // }

                        bytes
                    }
                    SeedSuffix::Custom(seed_suffix) => seed_suffix.clone(),
                };

                seeds.push(&seed_suffix[..]);

                let (pda, seed_bump) = Pubkey::find_program_address(&seeds[..], &inner.program_id);

                let descriptor = match is_writable {
                    Access::Write => AccountMeta::new(pda, (*is_signer).into()),
                    Access::Read => AccountMeta::new_readonly(pda, (*is_signer).into()),
                };

                let seed_bump = &[seed_bump];
                seeds.push(seed_bump);
                seeds.remove(0);

                inner.generic_template_accounts.push(descriptor);
                inner.generic_template_address_data.push(seeds.concat());
            }

            let collection_template_account_descriptors =
                inner.collection_template_account_descriptors.clone();
            for (meta, bump) in collection_template_account_descriptors.iter() {
                inner.collection_template_accounts.push(meta.clone());
                inner.collection_template_address_data.push(vec![*bump]);
            }

            inner.generic_template_instruction_data =
                Self::encode_template_instruction_data(&inner.generic_template_address_data);
            inner.collection_template_instruction_data =
                Self::encode_template_instruction_data(&inner.collection_template_address_data);

            match &inner.track_suffix_seed_seq {
                None => {}
                Some(suffix_seed_seq_refcell) => {
                    let mut suffix_seed_seq = suffix_seed_seq_refcell.lock().unwrap();
                    *suffix_seed_seq = inner.suffix_seed_seq;
                }
            }

            Ok(())
        })
        // Ok(self)
    }

    pub fn generic_templates(self: Arc<Self>) -> Vec<AccountMeta> {
        self.inner().generic_template_accounts.to_vec()
    }

    pub fn collection_templates(self: Arc<Self>) -> Vec<AccountMeta> {
        self.inner().collection_template_accounts.to_vec()
    }

    pub fn try_into(self: Arc<Self>) -> Result<solana_program::instruction::Instruction> {
        if !self.is_sealed() {
            return Err(error!("InstructionBuilder is not sealed!"));
        }

        let program_id = self.program_id();

        let mut instruction_data: Vec<u8> = Vec::new();
        instruction_data.extend(self.payload().to_vec());
        instruction_data.extend(self.instruction_data().to_vec());
        let accounts = self.try_accounts()?;

        log_info!("PROGRAM ID: {}", program_id);
        let instruction = solana_program::instruction::Instruction {
            program_id,
            data: instruction_data,
            accounts,
        };
        log_info!("INSTRUCTION: {:?}", instruction);
        Ok(instruction)
    }

    pub fn gather_accounts(
        &self,
        gather: Option<Gather>,
        first: Option<&Pubkey>,
    ) -> Result<Vec<Pubkey>> {
        let inner = self.inner();

        let mut vec = Vec::new();
        // by default, first template account is always at the
        // first position in the list unless `first` is supplied
        vec.extend_from_slice(&inner.generic_template_accounts);
        vec.extend_from_slice(&inner.token_accounts);
        vec.extend_from_slice(&inner.index_accounts);
        vec.extend_from_slice(&inner.collection_accounts);
        vec.extend_from_slice(&inner.collection_template_accounts);
        vec.extend_from_slice(&inner.handler_accounts);

        let (authority, identity) = match gather {
            None => (false, false),
            Some(Gather::Authority) => (true, false),
            Some(Gather::Identity) => (false, true),
            Some(Gather::All) => (true, true),
        };

        if authority {
            if let Some(authority) = &inner.authority {
                vec.push(authority.clone());
            }
        }

        if identity {
            if let Some(identity) = &inner.identity {
                vec.push(identity.clone());
            }
        }

        let mut vec = vec.iter().map(|account| account.pubkey).collect::<Vec<_>>();

        if let Some(first) = first {
            let index = vec.iter().position(|pubkey| pubkey == first).unwrap();
            vec.remove(index);
            vec.insert(0, *first);
        }

        Ok(vec)
    }
}

// impl TryFrom<&InstructionBuilder> for solana_program::instruction::Instruction {
//     type Error = crate::error::Error;

//     fn try_from(builder: &InstructionBuilder) -> std::result::Result<Self,Self::Error> {

//         if !builder.is_sealed() {
//             return Err(error!("InstructionBuilder is not sealed!"));
//         }

//         let program_id = builder.program_id();

//         let mut instruction_data: Vec<u8> = Vec::new();
//         instruction_data.extend(builder.payload().to_vec());
//         instruction_data.extend(builder.instruction_data().to_vec());
//         let accounts = builder.try_accounts()?;

//         let instruction = solana_program::instruction::Instruction {
//             program_id,
//             data : instruction_data,
//             accounts,
//         };

//         Ok(instruction)
//     }
// }
