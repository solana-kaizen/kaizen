use cfg_if::cfg_if;
use solana_program::pubkey::Pubkey;
use workflow_allocator_macros::{Meta, container};
use workflow_allocator::error;
use workflow_allocator::error::ErrorCode;
use workflow_allocator::container::Containers;
use workflow_allocator::container::Container;
use workflow_allocator::result::Result;
use workflow_allocator::prelude::*;
use super::meta::*;

pub type PubkeyCollection<'info,'refs> = PubkeyCollectionInterface<'info,'refs, PubkeyCollectionSegmentInterface<'info,'refs>>;
pub type PubkeyCollectionReference<'info,'refs> = PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>;

pub struct PubkeyCollectionInterface<'info,'refs, M> 
where M : PubkeyCollectionMetaTrait 
{
    meta : M,
    pub container : Option<PubkeyCollectionStore<'info,'refs>>,
}


impl<'info,'refs, M> PubkeyCollectionInterface<'info,'refs, M> 
where M : PubkeyCollectionMetaTrait
{
    pub fn try_new(
        meta:M,
    )->Result<Self> {
        Ok(Self { meta, container : None })
    }

    pub fn data_len_min() -> usize { M::min_data_len() }

    pub fn try_create<'i,'r>(
        &mut self,
        ctx: &ContextReference<'i,'r,'_,'_>,
        allocation_args: &AccountAllocationArgs<'i,'r,'_>,
        data_type : Option<u32>,
        container_type : Option<u32>
    ) -> Result<()> {
        let collection_store = PubkeyCollectionStore::try_allocate(ctx, allocation_args, 0)?;
        self.meta.try_create(collection_store.pubkey(), data_type, container_type)?;
        Ok(())
    }

    pub fn try_load(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
    ) -> Result<()> {

        if let Some(account_info) = ctx.locate_index_account(self.meta.pubkey()) {
            let container = PubkeyCollectionStore::try_load(account_info)?;
            self.container = Some(container);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotFound))
        }
    }

    pub fn try_create_with_meta<'ctx,'r>(
        ctx: &ContextReference<'ctx,'r,'_,'_>,
        allocation_args: &AccountAllocationArgs<'ctx,'_,'_>,
        data : &'info mut PubkeyCollectionMeta,
        data_type : Option<u32>,
        container_type : Option<u32>,

    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {


        let mut collection = PubkeyCollectionInterface::<'info,'refs,PubkeyCollectionMetaInterface<'info>>::try_from_meta(data)?;
        collection.try_create(ctx, allocation_args, data_type, container_type)?;
        Ok(collection)

    }

    pub fn try_load_from_meta(
        ctx: &ContextReference<'info,'refs,'_,'_>,
        data : &'info mut PubkeyCollectionMeta,
    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {

        let mut collection = PubkeyCollectionInterface::<'info,'refs,PubkeyCollectionMetaInterface<'info>>::try_from_meta(data)?;
        collection.try_load(ctx)?;
        Ok(collection)

    }

    pub fn try_from_meta<'ctx,'r>(
        data : &'info mut PubkeyCollectionMeta,
    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {

        let meta = PubkeyCollectionMetaInterface::new(data);
        PubkeyCollectionInterface::<PubkeyCollectionMetaInterface>::try_new(
            meta,
        )
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>,
    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>> {
        PubkeyCollectionInterface::<PubkeyCollectionSegmentInterface>::try_new(
            PubkeyCollectionSegmentInterface::new(
                segment
            ),
        )
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>,
    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>> {
        PubkeyCollectionInterface::<PubkeyCollectionSegmentInterface>::try_new(
            PubkeyCollectionSegmentInterface::new(
                segment,
            ),
        )
    }

    pub fn len(&self) -> usize {
        self.meta.get_len() as usize
    }

    pub fn try_insert_container<'i,'r, C : Container<'i,'r>>(&mut self, target: &C) -> Result<()> {
        self.try_insert_pubkey(target.pubkey())?;
        Ok(())
    }

    pub fn try_insert_pubkey(
        &mut self,
        key: &Pubkey
    ) -> Result<()> {
        if let Some(container) = &self.container {
            let seq = self.meta.advance_sequence();
            container.try_insert(seq,key)?;
            let len = self.meta.get_len();
            self.meta.set_len(len + 1);
            // self.sync_rent(ctx, rent_collector)
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
        }
    }

    pub fn try_remove(
        &mut self,
        record: &PubkeySequence
    ) -> Result<()> {
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
        ctx: &ContextReference<'info,'_,'_,'_>,
        rent_collector : &workflow_allocator::rent::RentCollector<'info,'refs>,
    ) -> workflow_allocator::result::Result<()> {
        // TODO: @alpha - transfer out excess rent
        if let Some(container) = &self.container {
            ctx.sync_rent(container.account(),rent_collector)?;
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
        }
    }


}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyCollectionStoreMeta {
    pub version : u32,
    pub container_type : u32,
}

#[container(Containers::OrderedCollection)]
pub struct PubkeyCollectionStore<'info, 'refs>{
    pub meta : RefCell<&'info mut PubkeyCollectionStoreMeta>,
    pub records : Array<'info, 'refs, PubkeyMeta>,
}

impl<'info, 'refs> PubkeyCollectionStore<'info, 'refs> {

    pub fn try_init(&self, container_type : Option<u32>) -> Result<()> {
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

    pub fn try_remove(&self, sequence : &PubkeySequence) -> Result<()> {
        let records: &[PubkeySequence] = self.records.as_struct_slice();
        match records.binary_search(sequence) {
            Ok(idx) => {
                unsafe { self.records.try_remove_at(idx,true)?; }
                Ok(())
            },
            Err(_idx) => {
                Err(error_code!(ErrorCode::PubkeyCollectionRecordNotFound))
            }
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
    if #[cfg(not(target_arch = "bpf"))] {
        use async_trait::async_trait;
        use solana_program::instruction::AccountMeta;
        use workflow_allocator::container::AccountAggregator;

        impl<'info,'refs, M> PubkeyCollectionInterface<'info,'refs, M> 
        where M : PubkeyCollectionMetaTrait
        {

            pub async fn get_pubkey_at(&self, idx: usize) -> Result<Pubkey> {
                let container = load_container::<PubkeyCollectionStore>(self.meta.pubkey()).await?;

                if idx >= self.len() {
                    log_trace!("idx: {}",idx);
                    log_trace!("self.len(): {}",self.len());
                    log_trace!("self.records.len(): {}",container.as_ref().unwrap().records.len());
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

            pub async fn load_reference_at(&self, idx: usize) -> Result<Arc<AccountDataReference>> {
                let pubkey = self.get_pubkey_at(idx).await?;
                let transport = Transport::global()?;
                match transport.lookup(&pubkey).await? {
                    Some(reference) => Ok(reference),
                    None => Err(error!("Error: missing account {} in collection {}",pubkey,self.meta.pubkey()))
                }
            }

            pub async fn collect_pubkeys(&self) -> Result<Vec<Pubkey>> {
                let container = load_container::<PubkeyCollectionStore>(self.meta.pubkey()).await?;
                if let Some(container) = container {
                    let pubkeys = container
                        .as_slice()
                        .iter()
                        .map(|r|r.get_key())
                        .collect::<Vec<Pubkey>>();
                    Ok(pubkeys)
                } else {
                    // Err(error_code!(ErrorCode::PubkeyCollectionMissing))
                    Err(error!("Error: missing collection container {}", self.meta.pubkey()))
                }
            }

            pub async fn load_references(&self) -> Result<Vec<Arc<AccountDataReference>>> {
                let pubkeys = self.collect_pubkeys().await?;
                let mut references = Vec::new();
                let transport = Transport::global()?;
                for pubkey in pubkeys.iter() {
                    if let Some(reference) = transport.lookup(pubkey).await? {
                        references.push(reference);
                    }
                }

                Ok(references)
            }

            pub async fn load_containers<'this, T>(&self)
            -> Result<Vec<ContainerReference<'this, T>>>
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let references = self.load_references().await?;
                let mut containers = Vec::new();
                for reference in references.iter() {
                    if reference.container_type() == T::container_type() {
                        let container = reference.try_into_container::<T>()?;
                        containers.push(container);
                    }
                }

                Ok(containers)
            }

        }        


        // #[async_trait(?Send)]
        #[workflow_async_trait]
        impl<'info,'refs,M> AccountAggregator for PubkeyCollectionInterface<'info,'refs,M> 
        // where T : Copy + Eq + PartialEq + Ord + 'info
        where M : PubkeyCollectionMetaTrait
        {
            type Key = Pubkey;
            async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
                if key.is_some() {
                    return Err(error_code!(ErrorCode::NotImplemented));
                }
                // let meta = self.meta()?;
                Ok(vec![AccountMeta::new(*self.meta.pubkey(), false)])
            }

            async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
                if key.is_some() {
                    return Err(error_code!(ErrorCode::NotImplemented));
                }
                // let meta = self.meta()?;
                Ok(vec![AccountMeta::new_readonly(*self.meta.pubkey(), false)])
            }
        
        }
    }
}
