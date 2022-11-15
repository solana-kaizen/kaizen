use cfg_if::cfg_if;
use crate::container::Container;
use crate::result::Result;
use kaizen::prelude::*;
use kaizen::error::ErrorCode;
use kaizen::container;
use super::meta::*;

pub type PdaCollection<'info,'refs> = PdaCollectionInterface<'info, PdaCollectionSegmentInterface<'info,'refs>>;
pub type PdaCollectionReference<'info> = PdaCollectionInterface<'info, PdaCollectionMetaInterface<'info>>;

pub struct PdaCollectionInterface<'info, M> {
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

    // pub fn get_seed_at(&self, idx : u64) -> Vec<u8> {
    //     let domain = self.domain;
    //     let index_bytes: [u8; 8] = unsafe { std::mem::transmute(idx.to_be()) };
    //     [domain, &self.meta.get_seed(),&index_bytes].concat()
    // }

    pub fn get_seed_at<'seed>(&'seed self, idx : &u64, suffix : Option<&'seed [u8]>) -> Vec<&'seed [u8]> {
        let index_bytes: &[u8;8] = unsafe { std::mem::transmute(idx as * const u64) };
        if let Some(suffix) = suffix {
            vec![self.domain, &self.meta.get_seed(), index_bytes, suffix]
        } else {
            vec![self.domain, &self.meta.get_seed(), index_bytes]
        }
    }

    // pub fn get_seed_at<'seed>(&'seed self, idx : &u64, bump : Option<u8>) -> Vec<&[u8]> {
    //     let index_bytes: &[u8;8] = unsafe { std::mem::transmute(idx as * const u64) };
    //     if let Some(bump) = bump {
    //         vec![self.domain, &self.meta.get_seed(), index_bytes, &[bump]]
    //     } else {
    //         vec![self.domain, &self.meta.get_seed(), index_bytes]
    //     }
    // }

    pub fn try_load_container<'refs,T>(&self, ctx: &ContextReference<'info,'refs,'_,'_>, suffix : &str, index: u64, bump_seed : u8) 
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {
        assert!(index < self.meta.get_len());
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(index.to_be()) };

        let pda = Pubkey::create_program_address(
            &[suffix.as_bytes(),&index_bytes,&[bump_seed]],
            ctx.program_id
        )?;

        if let Some(account_info) = ctx.locate_collection_account(&pda) {
            let container = T::try_load(account_info)?;
            Ok(container)
        } else {
            Err(error_code!(ErrorCode::AccountCollectionNotFound))
        }
    }

    pub fn try_create_container<'refs,T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        tpl_seed_suffix : &[u8],
        tpl_account_info : &'refs AccountInfo<'info>,
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
        // log_info!("pda collection creating index {} starting...",next_index);
        let tpl_seeds = self.get_seed_at(&next_index, Some(tpl_seed_suffix));
        
        let data_len = match data_len {
            Some(data_len) => data_len,
            None => T::initial_data_len()
        };
        let allocation_args = AccountAllocationArgs::new(AddressDomain::None);
        ctx.try_create_pda_with_args(
            data_len,
            &allocation_args,
            &tpl_seeds,
            tpl_account_info,
            false
        )?;

        self.meta.set_len(next_index);
        // log_info!("pda collection creating index {} ...done",next_index);
        let container = T::try_create(tpl_account_info)?;
        Ok(container)
    }

    pub fn try_insert_container<'refs,T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        seed_bump : u8,
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
        let seed_bump = &[seed_bump];
        let tpl_seeds = self.get_seed_at(&next_index,Some(seed_bump));
        let pda = Pubkey::create_program_address(
            &tpl_seeds,
            // &[&program_address_data_bytes,&[bump]],
            ctx.program_id
        )?;

        if container.pubkey() != &pda {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidAddress));
        }

        let account = ctx.locate_collection_account(&pda)
            .ok_or(error_code!(ErrorCode::AccountCollectionNotFound))?;
        //  {
        // let account = match ctx.locate_collection_account(&pda) {
        //     Some(account_info) => account_info,
        //     None => {
        //         return Err(error_code!(ErrorCode::AccountCollectionNotFound))
        //     }
        // };

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
    if #[cfg(not(target_os = "solana"))] {

        use futures::{stream::FuturesOrdered, StreamExt};
        use crate::container::interfaces::{
            PdaCollectionCreatorInterface,
            AsyncPdaCollectionCreatorInterface,
            PdaCollectionAccessorInterface,
            AsyncPdaCollectionAccessorInterface,
        };

        impl<'info,M> PdaCollectionInterface<'info,M> 
        where M: CollectionMeta
        {
            pub fn get_pda_at(&self, program_id : &Pubkey, idx : u64) -> Result<(Pubkey, u8)> {

                // log_trace!("find pda: {:?}",self.get_seed_at(idx));

                let (address, bump) = Pubkey::find_program_address(
                    &self.get_seed_at(&idx,None),
                    // &[&self.get_seed_at(idx)],
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
            where T: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                Ok(self.load_container_at_with_transport::<T>(program_id, idx, &transport).await?)
            }

            pub async fn load_container_at_with_transport<'this,T>(&self, program_id: &Pubkey, idx: usize, transport: &Arc<Transport>) 
            -> Result<Option<ContainerReference<'this,T>>> 
            where T: kaizen::container::Container<'this,'this>
            {
                let container_pubkey = self.get_pubkey_at(program_id, idx)?;
                Ok(load_container_with_transport::<T>(&transport,&container_pubkey).await?)
            }

            pub async fn load_container_range<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>) 
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                Ok(self.load_container_range_with_transport::<T>(program_id, range, &transport).await?)
            }

            pub async fn load_container_range_with_transport<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>, transport: &Arc<Transport>) 
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: kaizen::container::Container<'this,'this>
            {
                let mut futures = FuturesOrdered::new();
                for idx in range {
                    let f = self.load_container_at_with_transport::<T>(program_id, idx, &transport);
                    futures.push_back(f);
                }

                Ok(futures.collect::<Vec<_>>().await)
            }
        }

        // ~~~

        impl<'info,M> PdaCollectionCreatorInterface for PdaCollectionInterface<'info,M> 
        where M: CollectionMeta {
            type Creator = PdaCollectionCreator;
            fn creator(&self, program_id: &Pubkey, number_of_accounts : usize) -> Result<Arc<Self::Creator>> {
                
                let mut list = Vec::new();
                for idx in self.len()+1 ..= self.len()+number_of_accounts {
                    let (pubkey, bump) = self.get_pda_at(program_id, idx as u64)?;
                    list.push((pubkey,bump));
                }

                Ok(Arc::new(PdaCollectionCreator { list }))
            }
        }

        pub struct PdaCollectionCreator {
            list : Vec<(Pubkey, u8)>
        }

        #[workflow_async_trait]
        impl AsyncPdaCollectionCreatorInterface for PdaCollectionCreator
        {
            async fn writable_accounts_meta(&self) -> Result<Vec<(AccountMeta,u8)>> {

                let metas = self
                    .list
                    .iter()
                    .map(|(pubkey, bump):&(Pubkey,u8)| {
                        (AccountMeta::new(*pubkey, false),*bump)     
                    })
                    .collect();
                Ok(metas)
            }
        }

        // ~~~

        impl<'info,M> PdaCollectionAccessorInterface for PdaCollectionInterface<'info,M> 
        where M: CollectionMeta {
            type Accessor = PdaCollectionAccessor;
            fn accessor(&self, program_id: &Pubkey, index_range : std::ops::Range<usize>) -> Result<Arc<Self::Accessor>> {
                
                let mut list = Vec::new();
                for idx in index_range {
                    let (pubkey, _) = self.get_pda_at(program_id, idx as u64)?;
                    list.push(pubkey);
                }

                Ok(Arc::new(PdaCollectionAccessor { list }))
            }
        }

        pub struct PdaCollectionAccessor {
            list : Vec<Pubkey>
        }

        #[workflow_async_trait]
        impl AsyncPdaCollectionAccessorInterface for PdaCollectionAccessor
        {
            async fn writable_accounts_meta(&self) -> Result<Vec<AccountMeta>> {
                let metas = self
                    .list
                    .iter()
                    .map(|pubkey| {
                        AccountMeta::new(*pubkey, false)
                    })
                    .collect();
                Ok(metas)
            }
        }


    }
}
