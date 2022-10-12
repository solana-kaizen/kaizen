use cfg_if::cfg_if;
// use solana_program::rent::Rent;
// use solana_program::pubkey::Pubkey;
// use workflow_allocator_macros::{Meta, container};
use workflow_allocator_macros::Meta;
use crate::address::ProgramAddressData;
use crate::container::Container;
// use crate::context::ContextReference;
// use crate::error;
// use crate::error_code;
// use std::rc::Rc;
// use crate::error::ErrorCode;
// use borsh::{BorshDeserialize, BorshSerialize};
use crate::result::Result;
// use crate::container::segment::Segment;
// use crate::identity::*;
use workflow_allocator::prelude::*;
use workflow_allocator::error::ErrorCode;
use workflow_allocator::container;
// use workflow_allocator::container::Containers;
// use workflow_allocator::container::keys::Ts;

// use super::TsPubkey;
// use super::Container;

trait CollectionMeta {
    fn min_data_len() -> usize;
    fn try_init(&self, seed : &[u8], container_type : Option<u32>) -> Result<()>;
    fn get_seed<'data>(&'data self) -> Result<&'data [u8]>;
    fn get_len(&self) -> Result<u64>;
    fn set_len(&self, _len: u64) -> Result<()>;
    fn get_container_type(&self) -> Result<Option<u32>>;

}


#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct AccountCollectionMeta {
    collection_seed : u64,
    collection_len : u64,
    collection_container_type : u32,
}

impl AccountCollectionMeta {

    fn try_init(&self, seed_src : &[u8], container_type : Option<u32>) -> Result<()> {
        // TODO check that len, seed and container_type are blank
        self.set_len(0);
        self.set_collection_container_type(container_type.unwrap_or(0u32));
        // let seed = u64::from_le_bytes(seed_src[0..8].try_into().unwrap());
        let mut seed_dst = [0u8; 8];
        seed_dst.clone_from_slice(&seed_src[0..]);
        let seed = u64::from_be_bytes(seed_dst);
        self.set_collection_seed(seed);
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<AccountCollectionMeta>()
    }

    fn get_seed<'data>(&'data self) -> Result<&'data [u8]> {
        Ok(unsafe { std::mem::transmute(self.get_collection_seed().to_le()) })
    }

    fn get_len(&self) -> Result<u64> {
        Ok(self.get_collection_len())
    }

    fn set_len(&self, len : u64) -> Result<()> {
        self.set_collection_len(len);
        Ok(())
    }

    fn get_container_type(&self) -> Result<Option<u32>> {
        let container_type = self.get_collection_container_type();
        if container_type == 0 {
            Ok(None)
        } else {
            Ok(Some(container_type))
        }
    }

}

pub struct AccountCollectionMetaReference<'info> {
    data : &'info mut AccountCollectionMeta
}

// impl AccountCollectionMeta {
//     pub fn get_seed_as_bytes(&self) -> [u8;8] {
//         unsafe { std::mem::transmute(self.get_collection_seed().to_le()) }
//     }
// }

impl<'info> AccountCollectionMetaReference<'info> {

    pub fn new(data : &'info mut AccountCollectionMeta) -> Self {
        Self { data }
    }

    pub fn data_ref<'data>(&'data self) -> Result<&'data AccountCollectionMeta> {
        // Ok(self.segment.as_struct_ref::<AccountCollectionMeta>())
        Ok(self.data)
    }

    pub fn data_mut<'data>(&'data self) -> Result<&'data mut AccountCollectionMeta> {
        // Ok(self.segment.as_struct_mut::<AccountCollectionMeta>())
        Ok(self.data)
    }

}


impl<'info> CollectionMeta for AccountCollectionMetaReference<'info> {
    fn try_init(&self, seed : &[u8], container_type : Option<u32>) -> Result<()> {
        self.data_ref()?.try_init(seed,container_type)
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<AccountCollectionMeta>()
    }

    fn get_seed<'data>(&'data self) -> Result<&'data [u8]> {
        self.data_ref()?.get_seed()
    }
    
    fn get_len(&self) -> Result<u64> {
        self.data_ref()?.get_len()
    }
    
    fn set_len(&self, len : u64) -> Result<()> {
        self.data_ref()?.set_len(len)
    }
    
    fn get_container_type(&self) -> Result<Option<u32>> {
        self.data_ref()?.get_container_type()
    }

}


pub struct AccountCollectionMetaSegment<'info,'refs> {
    segment : Rc<Segment<'info,'refs>>
}

impl<'info,'refs> AccountCollectionMetaSegment<'info,'refs> {
    pub fn new(segment : Rc<Segment<'info,'refs>>) -> Self {
        Self { segment }
    }

    pub fn data_ref<'data>(&'data self) -> Result<&'data AccountCollectionMeta> {
        Ok(self.segment.as_struct_ref::<AccountCollectionMeta>())
    }

    pub fn data_mut<'data>(&'data self) -> Result<&'data mut AccountCollectionMeta> {
        Ok(self.segment.as_struct_mut::<AccountCollectionMeta>())
    }
}

impl<'info,'refs> CollectionMeta for AccountCollectionMetaSegment<'info,'refs> {

    fn try_init(&self, seed : &[u8], container_type : Option<u32>) -> Result<()> {
        self.data_ref()?.try_init(seed,container_type)
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<AccountCollectionMeta>()
    }

    fn get_seed<'data>(&'data self) -> Result<&'data [u8]> {
        self.data_ref()?.get_seed()
    }
    
    fn get_len(&self) -> Result<u64> {
        self.data_ref()?.get_len()
    }
    
    fn set_len(&self, len : u64) -> Result<()> {
        self.data_ref()?.set_len(len)
    }
    
    fn get_container_type(&self) -> Result<Option<u32>> {
        self.data_ref()?.get_container_type()
    }

}

pub struct AccountCollection<'info, M>{
    pub domain : &'info [u8],
    //meta : Rc<RefCell<&'info mut T>>,
    meta : M,
    // meta : Box<dyn CollectionMeta>,
    // pub account : &'refs AccountInfo<'info>,
    // pub external_meta : Option<&'info mut AccountCollectionMeta>,
    // pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
}


impl<'info,M> AccountCollection<'info,M>
where M: CollectionMeta
{
    fn new(domain:&'info [u8], meta:M)->Self{
        Self { domain, meta}
    }
    pub fn len(&self) -> Result<usize> {
        // self.meta().unwrap().get_len() as usize
        Ok(self.meta.get_len()? as usize)
    }

    // pub fn account(&self) -> &'refs AccountInfo<'info> {
    //     self.account
    // }

    // pub fn meta<'meta>(&'meta self) -> Result<&'meta AccountCollectionMeta> {
    //     if let Some(external_meta) = &self.external_meta {
    //         return Ok(external_meta);
    //     } else if let Some(segment) = &self.segment_meta {
    //         Ok(segment.as_struct_ref::<AccountCollectionMeta>())
    //     } else {
    //         Err(ErrorCode::AccountCollectionMissingMeta.into())
    //     }
    // }

    // pub fn meta_mut<'meta>(&'meta mut self) -> Result<&'meta mut AccountCollectionMeta> {
    //     if let Some(external_meta) = &mut self.external_meta {
    //         return Ok(external_meta);
    //     } else if let Some(segment) = &self.segment_meta {
    //         Ok(segment.as_struct_mut::<AccountCollectionMeta>())
    //     } else {
    //         Err(ErrorCode::AccountCollectionMissingMeta.into())
    //     }
    // }

    pub fn data_len_min() -> usize { std::mem::size_of::<AccountCollectionMeta>() }

    pub fn try_from_meta(
        data : &'info mut AccountCollectionMeta,
        account_info : &AccountInfo<'info>,
    ) -> Result<AccountCollection<'info, AccountCollectionMetaReference<'info>>> {

        // let m = M
        let reference = AccountCollectionMetaReference::new(data);
        // let r : &dyn M = &dyn reference;
        // let trait_object: Box<dyn M> = Box::new(reference) as Box<dyn M>;

        //let meta : M = reference;
        Ok(AccountCollection::<AccountCollectionMetaReference>::new(
            account_info.key.as_ref(),
            reference,
            // meta : Rc::new(reference), //AccountCollectionMetaReference::new(data)
            // account: account_info,
            // segment_meta : None,
            // external_meta : Some(meta),
        ))
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, '_>>
    ) -> Result<AccountCollection<'info, AccountCollectionMetaSegment<'info, 'info>>> {
        Ok(AccountCollection {
            domain : segment.account().key.as_ref(),
            meta : AccountCollectionMetaSegment::new(segment)
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, '_>>
    ) -> Result<AccountCollection<'info, AccountCollectionMetaSegment<'info, 'info>>> {
        Ok(AccountCollection {
            domain : segment.account().key.as_ref(),
            meta : AccountCollectionMetaSegment::new(segment)
        })
    }

    // pub fn try_create(&mut self, _ctx: &ContextReference, data_type : u32) -> Result<()> {
    pub fn try_init(&mut self, container_type : u32, seed : u64) -> Result<()> {
        // let data_type = self.meta().get_data_type();
        self.meta.try_init(seed,Some(container_type));
        // let meta = self.meta_mut()?;
        // meta.set_len(0);
        // meta.set_container_type(container_type);
        // meta.set_seed(seed);

        Ok(())
        // Ok(collection_store)
    }

    pub fn try_load<'refs,T>(&self, ctx: &ContextReference<'info,'refs,'_,'_>, suffix : &str, index: u64, bump_seed : u8) 
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {
        let meta = self.meta.clone()?;
        assert!(index < meta.get_len());
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

    pub fn get_seed_at(&self, meta: &AccountCollectionMeta, idx : u64) -> Vec<u8> {
        let domain = self.account().key.as_ref();
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(idx.to_le()) };
        [domain, &meta.get_seed_as_bytes(),&index_bytes].concat()
    }

    // pub fn try_create_and_insert<T>(
    pub fn try_create_pda<'refs,T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        // suffix : &str,
        seed_bump : u8,
        // tpl_program_address_data : ProgramAddressData,

        // allocation_args : &AccountAllocationArgs<'info,'refs,'_>,
        data_len : Option<usize>,
    )
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {
        // let domain = self.account().key.as_ref();

        let meta = self.meta()?;
        if T::container_type() != meta.get_container_type() {
            return Err(error_code!(ErrorCode::ContainerTypeMismatch));
        }
        let next_index = meta.get_len() + 1;
        // let index_bytes: [u8; 8] = unsafe { std::mem::transmute(next_index.to_le()) };

        let mut program_address_data_bytes = self.get_seed_at(meta,next_index);
        program_address_data_bytes.push(seed_bump);

        // let program_address_data_bytes : Vec<u8> = [domain, &meta.get_seed_as_bytes(),&index_bytes,&[seed_bump]].concat();
        let tpl_program_address_data = ProgramAddressData::from_bytes(program_address_data_bytes.as_slice());

        let pda = Pubkey::create_program_address(
            &[tpl_program_address_data.seed],
            //&[suffix.as_bytes(),&index_bytes,&[bump_seed]],
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
            // user_seed,
            tpl_program_address_data,
            tpl_account_info,
            false
        )?;

        self.meta_mut()?.set_len(next_index);

        let container = T::try_create(account_info)?;
        Ok(container)



    }

    pub fn try_insert_pda<'refs,T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        // suffix : &str,
        bump : u8,
        container: &T
    )
    -> Result<()>
    where T : Container<'info,'refs>
    {
        // let user_seed = self.account().key.as_ref();

        let meta = self.meta()?;
        if T::container_type() != meta.get_container_type() {
            return Err(error_code!(ErrorCode::ContainerTypeMismatch));
        }
        let next_index = meta.get_len() + 1;

        let program_address_data_bytes = self.get_seed_at(meta,next_index);
        // program_address_data_bytes.push(seed_bump);

        // let index_bytes: [u8; 8] = unsafe { std::mem::transmute(next_index.to_le()) };

        let pda = Pubkey::create_program_address(
            &[&program_address_data_bytes,&[bump]], //user_seed,suffix.as_bytes(),&index_bytes,&[seed_bump]],
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

        self.meta_mut()?.set_len(next_index);

        Ok(())
    }

    
}

cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {

        use futures::{stream::FuturesOrdered, StreamExt};

        impl<'info,M> AccountCollection<'info,M> 
        where M: CollectionMeta
        {
            pub fn get_pda_at(&self, program_id : &Pubkey, idx : u64) -> Result<(Pubkey, u8)> {
                let (address, bump) = Pubkey::find_program_address(
                    &[&self.get_seed_at(self.meta()?, idx)], //domain,&meta.get_seed_as_bytes(),&index_bytes],
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
// ~~~

// cfg_if! {
//     if #[cfg(not(target_arch = "bpf"))] {
//         use async_trait::async_trait;
//         use workflow_allocator::container::AccountAggregator;
//         use solana_program::instruction::AccountMeta;

//         #[async_trait(?Send)]
//         impl<'info,'refs,T> AccountAggregator for Collection<'info,'refs,T> 
//         where T : Copy + Eq + PartialEq + Ord + 'info
//         {
//             type Key = T;
//             async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
//                 if key.is_some() {
//                     return Err(error_code!(ErrorCode::NotImplemented));
//                 }
//                 let meta = self.meta()?;
//                 Ok(vec![AccountMeta::new(meta.get_pubkey(), false)])
//             }

//             async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
//                 if key.is_some() {
//                     return Err(error_code!(ErrorCode::NotImplemented));
//                 }
//                 let meta = self.meta()?;
//                 Ok(vec![AccountMeta::new_readonly(meta.get_pubkey(), false)])
//             }
        
//         }
//     }
// }
