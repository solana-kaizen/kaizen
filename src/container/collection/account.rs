use cfg_if::cfg_if;
use crate::address::ProgramAddressData;
use crate::container::Container;
use crate::result::Result;
use workflow_allocator::prelude::*;
use workflow_allocator::error::ErrorCode;
use workflow_allocator::container;
use super::meta::*;

pub type PdaCollection<'info,'refs> = PdaCollectionInterface<'info, PdaCollectionSegmentInterface<'info,'refs>>;
pub type PdaCollectionReference<'info> = PdaCollectionInterface<'info, PdaCollectionMetaInterface<'info>>;

pub struct PdaCollectionInterface<'info, M>{
    pub domain : &'info [u8],
    meta : M,
}

impl<'info,M> PdaCollectionInterface<'info,M>
where M: CollectionMeta
{
    fn try_create_impl(
        domain:&'info [u8],
        mut meta:M,
        // seed : &[u8],
        // container_type : Option<u32>
    )->Result<Self> {
        meta.try_create()?; // seed,container_type)?;
        Ok(Self { domain, meta })
    }

    fn try_load_impl(
        domain:&'info [u8],
        mut meta:M,
    )->Result<Self> {
        meta.try_load()?;
        Ok(Self { domain, meta })
    }

    pub fn data_len_min() -> usize { M::min_data_len() }

    pub fn try_create_from_meta(
        data : &'info mut PdaCollectionMeta,
        account_info : &AccountInfo<'info>,
        seed : &'static [u8],
        container_type : Option<u32>,
    ) -> Result<PdaCollectionInterface<'info, PdaCollectionMetaInterface<'info>>> {

        PdaCollectionInterface::<PdaCollectionMetaInterface>::try_create_impl(
            account_info.key.as_ref(),
            PdaCollectionMetaInterface::new(
                data,
                seed,
                container_type,
            ),
        )
    }

    pub fn try_load_from_meta(
        data : &'info mut PdaCollectionMeta,
        account_info : &AccountInfo<'info>,
        seed : &'static [u8],
        container_type : Option<u32>,
    ) -> Result<PdaCollectionInterface<'info, PdaCollectionMetaInterface<'info>>> {

        PdaCollectionInterface::<PdaCollectionMetaInterface>::try_load_impl(
            account_info.key.as_ref(),
            PdaCollectionMetaInterface::new(
                data,
                seed,
                container_type,
            )
        )
    }

    // pub fn try_create_from_segment<'refs>(
    //     segment : Rc<Segment<'info, 'refs>>,
    // ) -> Result<PdaCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>> {
    //     PdaCollectionInterface::<PdaCollectionSegmentInterface>::try_load_impl(
    //         segment.account().key.as_ref(),
    //         PdaCollectionSegmentInterface::new(segment),
    //     )
    // }

    // pub fn try_load_from_segment<'refs>(
    //         segment : Rc<Segment<'info, 'refs>>
    // ) -> Result<PdaCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>> {
    //     PdaCollectionInterface::<PdaCollectionSegmentInterface>::try_load_impl(
    //         segment.account().key.as_ref(),
    //         PdaCollectionSegmentInterface::new(segment)
    //     )
    // }

    pub fn try_create_from_segment_with_collection_args<'refs>(
        segment : Rc<Segment<'info, 'refs>>,
        seed : &'static [u8],
        container_type : Option<u32>,
    ) -> Result<PdaCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>> {
        PdaCollectionInterface::<PdaCollectionSegmentInterface>::try_load_impl(
            segment.account().key.as_ref(),
            PdaCollectionSegmentInterface::new(
                segment,
                seed,
                container_type
            ),
        )
    }

    pub fn try_load_from_segment_with_collection_args<'refs>(
            segment : Rc<Segment<'info, 'refs>>,
            seed : &'static [u8],
            container_type : Option<u32>,
    ) -> Result<PdaCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>> {
        PdaCollectionInterface::<PdaCollectionSegmentInterface>::try_load_impl(
            segment.account().key.as_ref(),
            PdaCollectionSegmentInterface::new(
                segment,
                seed,
                container_type
            )
        )
    }

    pub fn try_create(
        &mut self,
        // seed : &[u8],
        // container_type : Option<u32>,
    ) -> Result<()> {
        self.meta.try_create()//seed, container_type)
    }

    pub fn len(&self) -> usize {
        self.meta.get_len() as usize
    }

    pub fn get_seed_at(&self, idx : u64) -> Vec<u8> {
        let domain = self.domain;
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(idx.to_le()) };
        [domain, &self.meta.get_seed(),&index_bytes].concat()
    }

    pub fn try_load_container<'refs,T>(&self, ctx: &ContextReference<'info,'refs,'_,'_>, suffix : &str, index: u64, bump_seed : u8) 
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {
        assert!(index < self.meta.get_len());
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(index.to_le()) };

        let pda = Pubkey::create_program_address(
            &[suffix.as_bytes(),&index_bytes,&[bump_seed]],
            ctx.program_id
        )?;

        if let Some(account_info) = ctx.locate_index_account(&pda) {
            let container = T::try_load(account_info)?;
            Ok(container)
        } else {
            Err(error_code!(ErrorCode::AccountCollectionNotFound))
        }
    }

    pub fn try_create_container<'refs,T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        seed_bump : u8,
        data_len : Option<usize>,
    )
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {
        if let Some(container_type) = self.meta.get_container_type() {
            if T::container_type() != container_type {
                return Err(error_code!(ErrorCode::ContainerTypeMismatch));
            }
        }

        let next_index = self.meta.get_len() + 1;
        let mut program_address_data_bytes = self.get_seed_at(next_index);
        program_address_data_bytes.push(seed_bump);
        let tpl_program_address_data = ProgramAddressData::from_bytes(program_address_data_bytes.as_slice());

        let pda = Pubkey::create_program_address(
            &[tpl_program_address_data.seed],
            ctx.program_id
        )?;

        let tpl_account_info = match ctx.locate_index_account(&pda) {
            Some(account_info) => account_info,
            None => {
                return Err(error_code!(ErrorCode::AccountCollectionNotFound))
            }
        };

        let data_len = match data_len {
            Some(data_len) => data_len,
            None => T::initial_data_len()
        };

        let allocation_args = AccountAllocationArgs::new(AddressDomain::None);

        let account_info = ctx.try_create_pda_with_args(
            data_len,
            &allocation_args,
            tpl_program_address_data,
            tpl_account_info,
            false
        )?;

        self.meta.set_len(next_index);

        let container = T::try_create(account_info)?;
        Ok(container)
    }

    pub fn try_insert_container<'refs,T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        bump : u8,
        container: &T
    )
    -> Result<()>
    where T : Container<'info,'refs>
    {
        if let Some(container_type) = self.meta.get_container_type() {
            if T::container_type() != container_type {
                return Err(error_code!(ErrorCode::ContainerTypeMismatch));
            }
        }

        let next_index = self.meta.get_len() + 1;
        let program_address_data_bytes = self.get_seed_at(next_index);
        let pda = Pubkey::create_program_address(
            &[&program_address_data_bytes,&[bump]],
            ctx.program_id
        )?;

        if container.pubkey() != &pda {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidAddress));
        }

        let account = match ctx.locate_index_account(&pda) {
            Some(account_info) => account_info,
            None => {
                return Err(error_code!(ErrorCode::AccountCollectionNotFound))
            }
        };

        if account.data_len() < std::mem::size_of::<u32>() {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidAccount))
        }

        if T::container_type() != container::try_get_container_type(account)? {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidContainerType))
        }

        self.meta.set_len(next_index);

        Ok(())
    }
    
}

cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {

        use futures::{stream::FuturesOrdered, StreamExt};
        use crate::container::interfaces::PdaCollectionBuilder;
        use async_trait::async_trait;

        #[async_trait(?Send)]
        impl<'info,M> PdaCollectionBuilder for PdaCollectionInterface<'info,M> 
        where M: CollectionMeta
        {
            async fn writable_account_meta(&self, program_id : &Pubkey) -> Result<(AccountMeta,u8)> {

                let idx = self.len() + 1;
                let (pubkey, bump) = self.get_pda_at(program_id, idx as u64)?;
                Ok((AccountMeta::new(pubkey, false),bump))
            }

            async fn writable_account_meta_range(&self, program_id : &Pubkey, items : usize) -> Result<Vec<(AccountMeta,u8)>> {

                let mut list = Vec::new();
                for idx in self.len()+1 ..= self.len()+items {
                    let (pubkey, bump) = self.get_pda_at(program_id, idx as u64)?;
                    list.push((AccountMeta::new(pubkey, false),bump))
                }
                Ok(list)
            }
        }

        impl<'info,M> PdaCollectionInterface<'info,M> 
        where M: CollectionMeta
        {
            pub fn get_pda_at(&self, program_id : &Pubkey, idx : u64) -> Result<(Pubkey, u8)> {
                let (address, bump) = Pubkey::find_program_address(
                    &[&self.get_seed_at(idx)],
                    program_id
                );

                Ok((address, bump))
            }

            #[inline(always)]
            pub fn get_pubkey_at(&self, program_id : &Pubkey, idx : usize) -> Result<Pubkey> {
                Ok(self.get_pda_at(program_id, idx as u64)?.0)
            }

            pub async fn load_container_at<'this,T>(&self, program_id: &Pubkey, idx: usize) 
            -> Result<Option<ContainerReference<'this,T>>> 
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                Ok(self.load_container_at_with_transport::<T>(program_id, idx, &transport).await?)
            }

            pub async fn load_container_at_with_transport<'this,T>(&self, program_id: &Pubkey, idx: usize, transport: &Arc<Transport>) 
            -> Result<Option<ContainerReference<'this,T>>> 
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let container_pubkey = self.get_pubkey_at(program_id, idx)?;
                Ok(load_container_with_transport::<T>(&transport,&container_pubkey).await?)
            }

            pub async fn load_container_range<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>) 
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                Ok(self.load_container_range_with_transport::<T>(program_id, range, &transport).await?)
            }

            pub async fn load_container_range_with_transport<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>, transport: &Arc<Transport>) 
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let mut futures = FuturesOrdered::new();
                for idx in range {
                    let f = self.load_container_at_with_transport::<T>(program_id, idx, &transport);
                    futures.push_back(f);
                }

                Ok(futures.collect::<Vec<_>>().await)
            }
        }
    }
}
