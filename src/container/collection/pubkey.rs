//!
//! Arbitrary pubkey collection (Array-based, opcode-restricted)
//! 

use super::meta::*;
use cfg_if::cfg_if;
use kaizen::container::Container;
use kaizen::container::Containers;
use kaizen::error::ErrorCode;
use kaizen::prelude::*;
use kaizen::result::Result;
use kaizen_macros::{container, Meta};
use solana_program::pubkey::Pubkey;

pub type PubkeyCollection<'info, 'refs> =
    PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>;
pub type PubkeyCollectionReference<'info, 'refs> =
    PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionMetaInterface<'info>>;

pub struct PubkeyCollectionInterface<'info, 'refs, M>
where
    M: PubkeyCollectionMetaTrait,
{
    meta: M,
    pub container: Option<PubkeyCollectionStore<'info, 'refs>>,
}

impl<'info, 'refs, M> PubkeyCollectionInterface<'info, 'refs, M>
where
    M: PubkeyCollectionMetaTrait,
{
    pub fn try_new(meta: M) -> Result<Self> {
        Ok(Self {
            meta,
            container: None,
        })
    }

    pub fn data_len_min() -> usize {
        M::min_data_len()
    }

    pub fn try_create<'i, 'r>(
        &mut self,
        ctx: &ContextReference<'i, 'r, '_, '_>,
        allocation_args: &AccountAllocationArgs<'i, 'r, '_>,
        data_type: Option<u32>,
        container_type: Option<u32>,
    ) -> Result<()> {
        // let collection_store = PubkeyCollectionStore::try_allocate(ctx, allocation_args, 0)?;
        let collection_store = PubkeyCollectionStore::try_allocate(ctx, allocation_args, 0)?;
        self.meta
            .try_create(collection_store.pubkey(), data_type, container_type)?;
        collection_store.try_init(container_type)?;
        Ok(())
    }

    pub fn try_load(&mut self, ctx: &ContextReference<'info, 'refs, '_, '_>) -> Result<()> {
        if let Some(account_info) = ctx.locate_index_account(self.meta.pubkey()) {
            let container = PubkeyCollectionStore::try_load(account_info)?;
            self.container = Some(container);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotFound))
        }
    }

    pub fn try_create_with_meta<'ctx, 'r>(
        ctx: &ContextReference<'ctx, 'r, '_, '_>,
        allocation_args: &AccountAllocationArgs<'ctx, 'r, '_>,
        data: &'info mut PubkeyCollectionMeta,
        data_type: Option<u32>,
        container_type: Option<u32>,
    ) -> Result<PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionMetaInterface<'info>>> {
        let mut collection = PubkeyCollectionInterface::<
            'info,
            'refs,
            PubkeyCollectionMetaInterface<'info>,
        >::try_from_meta(data)?;
        collection.try_create(ctx, allocation_args, data_type, container_type)?;
        Ok(collection)
    }

    pub fn try_load_from_meta(
        ctx: &ContextReference<'info, 'refs, '_, '_>,
        data: &'info mut PubkeyCollectionMeta,
    ) -> Result<PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionMetaInterface<'info>>> {
        let mut collection = PubkeyCollectionInterface::<
            'info,
            'refs,
            PubkeyCollectionMetaInterface<'info>,
        >::try_from_meta(data)?;
        collection.try_load(ctx)?;
        Ok(collection)
    }

    pub fn try_from_meta(
        data: &'info mut PubkeyCollectionMeta,
    ) -> Result<PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionMetaInterface<'info>>> {
        let meta = PubkeyCollectionMetaInterface::new(data);
        PubkeyCollectionInterface::<PubkeyCollectionMetaInterface>::try_new(meta)
    }

    pub fn try_create_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<
        PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>,
    > {
        PubkeyCollectionInterface::<PubkeyCollectionSegmentInterface>::try_new(
            PubkeyCollectionSegmentInterface::new(segment),
        )
    }

    pub fn try_load_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<
        PubkeyCollectionInterface<'info, 'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>,
    > {
        PubkeyCollectionInterface::<PubkeyCollectionSegmentInterface>::try_new(
            PubkeyCollectionSegmentInterface::new(segment),
        )
    }

    pub fn len(&self) -> usize {
        self.meta.get_len() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.meta.get_len() == 0
    }

    pub fn try_insert_container<'i, 'r, C: Container<'i, 'r>>(&mut self, target: &C) -> Result<()> {
        self.try_insert_pubkey(target.pubkey())?;
        Ok(())
    }

    pub fn try_insert_pubkey(&mut self, key: &Pubkey) -> Result<()> {
        if let Some(container) = &self.container {
            let seq = self.meta.advance_sequence();
            container.try_insert(seq, key)?;
            let len = self.meta.get_len();
            self.meta.set_len(len + 1);
            // self.sync_rent(ctx, rent_collector)
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
        }
    }

    pub fn try_remove(&mut self, record: &PubkeySequence) -> Result<()> {
        {
            if self.container.is_none() {
                return Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded));
            }

            self.container.as_ref().unwrap().try_remove(record)?;
        }

        // let meta = self.meta_mut()?;
        let len = self.meta.get_len();
        self.meta.set_len(len - 1);

        // self.sync_rent(ctx, rent_collector);

        Ok(())
    }

    // pub fn as_slice(&self) -> Result<&[PubkeyMeta]> {
    //     if let Some(container) = &self.container {
    //         Ok(container.as_slice())
    //     } else {
    //         Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
    //     }
    // }

    // pub fn as_slice_mut(&mut self) -> Result<&mut [PubkeyMeta]> {
    //     if let Some(container) = &mut self.container {
    //         Ok(container.as_slice_mut())
    //     } else {
    //         Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
    //     }
    // }

    pub fn sync_rent(
        &self,
        ctx: &ContextReference<'info, 'refs, '_, '_>,
        rent_collector: &kaizen::rent::RentCollector<'info, 'refs>,
    ) -> kaizen::result::Result<()> {
        // TODO: @alpha - transfer out excess rent
        if let Some(container) = &self.container {
            ctx.sync_rent(container.account(), rent_collector)?;
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
        }
    }
}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyCollectionStoreMeta {
    pub version: u32,
    pub container_type: u32,
}

#[container(Containers::OrderedCollection)]
pub struct PubkeyCollectionStore<'info, 'refs> {
    pub meta: RefCell<&'info mut PubkeyCollectionStoreMeta>,
    pub records: Array<'info, 'refs, PubkeyMeta>,
}

impl<'info, 'refs> PubkeyCollectionStore<'info, 'refs> {
    pub fn try_init(&self, container_type: Option<u32>) -> Result<()> {
        let mut meta = self.meta.borrow_mut();
        meta.set_version(1);
        meta.set_container_type(container_type.unwrap_or(0u32));
        Ok(())
    }

    fn try_insert(&self, seq: u32, key: &Pubkey) -> Result<()> {
        let record = unsafe { self.records.try_allocate(false)? };
        record.set_seq(seq);
        record.key = *key;
        Ok(())
    }

    // fn try_insert<C : Container<'info,'refs>>(&self, seq: u32, key: &Pubkey) -> Result<()> {
    //     let record = unsafe { self.records.try_allocate(false)? };
    //     record.set_seq(seq);
    //     record.key = *key;
    //     Ok(())
    // }

    pub fn try_remove(&self, sequence: &PubkeySequence) -> Result<()> {
        let records: &[PubkeySequence] = self.records.as_struct_slice();
        match records.binary_search(sequence) {
            Ok(idx) => {
                unsafe {
                    self.records.try_remove_at(idx, true)?;
                }
                Ok(())
            }
            Err(_idx) => Err(error_code!(ErrorCode::PubkeyCollectionRecordNotFound)),
        }
    }

    pub fn as_slice(&self) -> &'info [PubkeyMeta] {
        self.records.as_slice()
    }

    pub fn as_slice_mut(&mut self) -> &'info mut [PubkeyMeta] {
        self.records.as_slice_mut()
    }
}

// ~~~

cfg_if! {
    if #[cfg(not(target_os = "solana"))] {
        use kaizen::error;
        use futures::future::join_all;
        use solana_program::instruction::AccountMeta;
        use kaizen::container::{AccountAggregatorInterface,AsyncAccountAggregatorInterface};

        impl<'info,'refs, M> PubkeyCollectionInterface<'info,'refs, M>
        where M : PubkeyCollectionMetaTrait
        {

            pub async fn get_pubkey_at(&self, idx: usize) -> Result<Pubkey> {
                let container = load_container::<PubkeyCollectionStore>(self.meta.pubkey()).await?;

                if idx >= self.len() {
                    log_trace!("idx: {} self.len(): {} self.records.len(): {}",idx,self.len(),container.as_ref().unwrap().records.len());
                }
                assert!(idx < self.len());

                if let Some(container) = container {
                    let pubkey = container.records.get_at(idx).key;
                    Ok(pubkey)
                } else {
                    // Err(error_code!(ErrorCode::PubkeyCollectionMissing))
                    Err(error!("Error: missing collection container {}", self.meta.pubkey()))
                }
            }

            pub async fn get_pubkey_range(&self, range: std::ops::Range<usize>) -> Result<Vec<Pubkey>> {
                let container = load_container::<PubkeyCollectionStore>(self.meta.pubkey()).await?;
                let mut pubkeys = Vec::new();

                for idx in range {
                    if idx >= self.len() {
                        log_trace!("idx: {} self.len(): {} self.records.len(): {}",idx,self.len(),container.as_ref().unwrap().records.len());
                    }
                    assert!(idx < self.len());
                    if let Some(container) = &container {
                        let pubkey = container.records.get_at(idx).key;
                        pubkeys.push(pubkey);
                    } else {
                        return Err(error!("Error: missing collection container {}", self.meta.pubkey()));
                    }
                }

                Ok(pubkeys)
            }

            pub async fn load_reference_at(&self, idx: usize) -> Result<Arc<AccountDataReference>> {
                let pubkey = self.get_pubkey_at(idx).await?;
                let transport = Transport::global()?;
                // Err(error!("Error: missing account {} in collection {}",pubkey,self.meta.pubkey()))
                match transport.lookup(&pubkey).await? {
                    Some(reference) => Ok(reference),
                    None => Err(error!("Error: missing account {} in collection {}",pubkey,self.meta.pubkey()))
                }
            }

            pub async fn load_reference_range(&self, range: std::ops::Range<usize>) -> Result<Vec<Arc<AccountDataReference>>> {
                let transport = Transport::global()?;
                let mut list = Vec::new();

                let pubkeys = self.get_pubkey_range(range.clone()).await?;
                let mut lookups = Vec::new();
                for pubkey in pubkeys.iter() {
                    lookups.push(transport.lookup(pubkey));
                }

                let mut idx = range.start;
                let results = join_all(lookups).await;
                for result in results {
                    match result? {
                        Some(reference) => list.push(reference.clone()),
                        None => return Err(error!("Error: missing account {} in collection {}",pubkeys[idx],self.meta.pubkey()))
                    }
                    idx += 1;
                }

                Ok(list)
            }

            pub async fn load_container_at<'this,C>(&self, idx: usize)
            -> Result<ContainerReference<'this,C>>
            where C: kaizen::container::Container<'this,'this>
            {
                let reference = self.load_reference_at(idx).await?;
                let container = reference.try_into_container::<C>()?;
                Ok(container)
            }

            pub async fn load_container_range<'this,C>(&self, range: std::ops::Range<usize>)
            -> Result<Vec<Option<ContainerReference<'this,C>>>>
            where C: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                let mut list = Vec::new();

                let pubkeys = self.get_pubkey_range(range.clone()).await?;
                let mut lookups = Vec::new();
                for pubkey in pubkeys.iter() {
                    lookups.push(transport.lookup(pubkey));
                }

                let results = join_all(lookups).await;

                let mut idx = range.start;
                for result in results {
                    match result? {
                        Some(reference) => {
                            let container = match reference.try_into_container::<C>() {
                                Ok(container) => Some(container),
                                Err(_) => {
                                    None
                                }
                            };
                            list.push(container)
                        },
                        None => return Err(error!("Error: missing account {} in collection {}",pubkeys[idx],self.meta.pubkey()))
                    }
                    idx += 1;
                }

                Ok(list)
            }

            pub async fn load_container_range_strict<'this,C>(&self, range: std::ops::Range<usize>)
            -> Result<Vec<ContainerReference<'this,C>>>
            where C: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                let mut list = Vec::new();

                let pubkeys = self.get_pubkey_range(range.clone()).await?;
                let mut lookups = Vec::new();
                for pubkey in pubkeys.iter() {
                    lookups.push(transport.lookup(pubkey));
                }

                let results = join_all(lookups).await;

                let mut idx = range.start;
                for result in results {
                    match result? {
                        Some(reference) => {
                            let container = reference.try_into_container::<C>()?;
                            list.push(container)
                        },
                        None => return Err(error!("Error: missing account {} in collection {}",pubkeys[idx],self.meta.pubkey()))
                    }
                    idx += 1;
                }

                Ok(list)
            }

            // pub async fn load_container_range_strict<'this,C>(&self, range: std::ops::Range<usize>)
            // -> Result<Vec<ContainerReference<'this,C>>>
            // where C: kaizen::container::Container<'this,'this>
            // {
            //     let transport = Transport::global()?;
            //     let mut list = Vec::new();

            //     for idx in range {
            //         let pubkey = self.get_pubkey_at(idx).await?;
            //         match transport.lookup(&pubkey).await? {
            //             Some(reference) => {
            //                 list.push(reference.try_into_container::<C>()?)
            //             },
            //             None => return Err(error!("Error: missing account {} in collection {}",pubkey,self.meta.pubkey()))
            //         }
            //     }

            //     Ok(list)
            // }

            pub async fn get_aggregator_cache_pubkeys(&self) -> Result<Vec<Pubkey>> {
                Ok(vec![*self.meta.pubkey()])
            }

            pub async fn reload(&self) -> Result<Option<Arc<AccountDataReference>>>  {
                reload_reference(self.meta.pubkey()).await
            }

            pub fn invalidate(&self) -> Result<()> {
                Transport::global()?.purge(Some(self.meta.pubkey()))?;
                Ok(())
            }

            pub async fn find_container<'this,C>(&self)
            -> Result<Option<ContainerReference<'this,C>>>
            where C: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;

                if let Some(container) = load_container::<PubkeyCollectionStore>(self.meta.pubkey()).await? {
                    let pubkeys = container.records.as_slice();
                    for entry in pubkeys.iter() {
                        if let Some(reference) = transport.lookup(&entry.key).await? {
                            if let Ok(container) = reference.try_into_container::<C>() {
                                return Ok(Some(container))
                            }
                        }
                    }
                }

                Ok(None)
            }

            // pub async fn collect_pubkeys(&self) -> Result<Vec<Pubkey>> {
            //     let container = load_container::<PubkeyCollectionStore>(self.meta.pubkey()).await?;
            //     if let Some(container) = container {
            //         let pubkeys = container
            //             .as_slice()
            //             .iter()
            //             .map(|r|r.get_key())
            //             .collect::<Vec<Pubkey>>();
            //         Ok(pubkeys)
            //     } else {
            //         // Err(error_code!(ErrorCode::PubkeyCollectionMissing))
            //         Err(error!("Error: missing collection container {}", self.meta.pubkey()))
            //     }
            // }

            // pub async fn load_references(&self) -> Result<Vec<Arc<AccountDataReference>>> {
            //     let pubkeys = self.collect_pubkeys().await?;
            //     let mut references = Vec::new();
            //     let transport = Transport::global()?;
            //     for pubkey in pubkeys.iter() {
            //         if let Some(reference) = transport.lookup(pubkey).await? {
            //             references.push(reference);
            //         }
            //     }

            //     Ok(references)
            // }

            // pub async fn load_containers<'this, T>(&self)
            // -> Result<Vec<ContainerReference<'this, T>>>
            // where T: kaizen::container::Container<'this,'this>
            // {
            //     let references = self.load_references().await?;
            //     let mut containers = Vec::new();
            //     for reference in references.iter() {
            //         if reference.container_type() == T::container_type() {
            //             let container = reference.try_into_container::<T>()?;
            //             containers.push(container);
            //         }
            //     }

            //     Ok(containers)
            // }

        }

        // impl<'info,'refs, M> PubkeyCollectionInterface<'info,'refs, M>
        // where M : PubkeyCollectionMetaTrait
        // {
        //     pub fn aggregator(&self) -> Result<PubkeyCollectionAsyncAccountAggregator> {
        //         let metas = vec![AccountMeta::new(*self.meta.pubkey(), false)];
        //         Ok(PubkeyCollectionAsyncAccountAggregator::new(metas))
        //     }

        //     pub fn aggregator(&self) -> Result<PubkeyCollectionAsyncAccountAggregator> {
        //         let metas = vec![AccountMeta::new(*self.meta.pubkey(), false)];
        //         Ok(PubkeyCollectionAsyncAccountAggregator::new(metas))
        //     }
        // }

        impl<'info,'refs,M> AccountAggregatorInterface for PubkeyCollectionInterface<'info,'refs,M>
        where M : PubkeyCollectionMetaTrait {
            type Aggregator = PubkeyCollectionAsyncAccountAggregator;

            fn aggregator(&self) -> Result<Arc<Self::Aggregator>> {
                let pubkeys = vec![*self.meta.pubkey()];
                Ok(Arc::new(PubkeyCollectionAsyncAccountAggregator::new(pubkeys)))
            }
        }

        pub struct PubkeyCollectionAsyncAccountAggregator {
            pubkeys: Vec<Pubkey>,
        }

        impl PubkeyCollectionAsyncAccountAggregator {
            pub fn new(pubkeys: Vec<Pubkey>) -> PubkeyCollectionAsyncAccountAggregator {
                PubkeyCollectionAsyncAccountAggregator {
                    pubkeys
                }
            }
        }

        #[workflow_async_trait]
        impl AsyncAccountAggregatorInterface for PubkeyCollectionAsyncAccountAggregator//PubkeyCollectionInterface<'info,'refs,M>
        // impl<'info,'refs,M> AccountAggregator for GenericAccountAggregator//PubkeyCollectionInterface<'info,'refs,M>
        // where T : Copy + Eq + PartialEq + Ord + 'info
        // where M : PubkeyCollectionMetaTrait
        {
            type Key = Pubkey;
            async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
                if key.is_some() {
                    return Err(error_code!(ErrorCode::NotImplemented));
                }

                let metas = self
                    .pubkeys
                    .iter()
                    .map(|pubkey|
                        AccountMeta::new(*pubkey, true)
                    ).collect();

                Ok(metas)
            }

            async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
                if key.is_some() {
                    return Err(error_code!(ErrorCode::NotImplemented));
                }
                let metas = self
                    .pubkeys
                    .iter()
                    .map(|pubkey|
                        AccountMeta::new(*pubkey, true)
                    ).collect();

                Ok(metas)
            }

        }
    }
}
