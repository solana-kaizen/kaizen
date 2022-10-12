use cfg_if::cfg_if;
use solana_program::pubkey::Pubkey;
use workflow_allocator_macros::{Meta, container};
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
use workflow_allocator::container::Containers;
// use workflow_allocator::container::keys::Ts;

// use super::TsPubkey;
// use super::Container;


#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct OrderedCollectionMeta {
    pubkey: Pubkey,
    count : u64,
    data_type : u32,
}

// impl CollectionMeta {
//     pub fn init(&mut self, pubkey : &Pubkey, data_type : u32) {
//         self.set_pubkey(*pubkey);
//         self.set_data_type(data_type);
//         self.set_count(0);
//     }
// }

pub struct OrderedCollection<'info,'refs, T> 
where T : Copy + Eq + PartialEq + Ord 
{
    // pub external_meta : Option<&'info mut OrderedCollectionMeta>,
    pub external_meta : Option<&'refs mut OrderedCollectionMeta>,
    pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
    pub container : Option<OrderedCollectionStore<'info,'refs, T>>,
    // _t_ : std::marker::PhantomData<T>,
}


impl<'info,'refs, T> OrderedCollection<'info,'refs, T> 
where T : Copy + Eq + PartialEq + Ord + 'info + 'refs
{
    pub fn meta<'meta>(&'meta self) -> Result<&'meta OrderedCollectionMeta> {
        if let Some(external_meta) = &self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_ref::<OrderedCollectionMeta>())
        } else {
            Err(ErrorCode::OrderedCollectionMissingMeta.into())
        }
    }

    pub fn meta_mut<'meta>(&'meta mut self) -> Result<&'meta mut OrderedCollectionMeta> {
        if let Some(external_meta) = &mut self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_mut::<OrderedCollectionMeta>())
        } else {
            Err(ErrorCode::OrderedCollectionMissingMeta.into())
        }
    }

    pub fn data_len_min() -> usize { std::mem::size_of::<OrderedCollectionMeta>() }

    // pub fn try_from_meta(meta : &'info mut OrderedCollectionMeta) -> Result<Self> {
    pub fn try_from_meta(meta : &'refs mut OrderedCollectionMeta) -> Result<Self> {
        Ok(OrderedCollection {
            segment_meta : None,
            external_meta : Some(meta),
            container : None
        })
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<OrderedCollection<'info,'refs,T>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(OrderedCollection {
            segment_meta : Some(segment),
            external_meta : None,
            container : None
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<OrderedCollection<'info,'refs,T>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(OrderedCollection {
            segment_meta : Some(segment),
            external_meta : None,
            container : None
        })
    }

    pub fn try_create(&mut self, ctx: &ContextReference<'info,'refs,'_,'_>, allocation_args: &AccountAllocationArgs<'info,'refs,'_>, data_type : u32) -> Result<()> {
    // pub fn try_create(&mut self, ctx: &ContextReference<'info,'refs,'_,'_>, allocation_args: &AccountAllocationArgs<'_,'_,'_>, data_type : u32) -> Result<()> {
    // pub fn try_create(&mut self, ctx: &ContextReference<'info,'refs,'_,'_>, allocation_args: &AccountAllocationArgs, data_type : u32) -> Result<()> {
    // pub fn try_create(&mut self, ctx: &ContextReference, allocation_args: &AccountAllocationArgs<'info,'refs,'_>, data_type : u32) -> Result<()> {
        // let data_type = self.meta().get_data_type();
        // let allocation_args = AccountAllocationArgs::new(AddressDomain::Identity);
        // let allocation_args = AccountAllocationArgs::new();
        let collection_store = OrderedCollectionStore::<T>::try_allocate(ctx, allocation_args, 0)?;
        collection_store.try_init(data_type)?;
        let meta = self.meta_mut()?;
        meta.set_data_type(data_type);
        meta.set_pubkey(*collection_store.pubkey());

        Ok(())
        // Ok(collection_store)
    }

    // pub fn try_load<'ctx>(&mut self, ctx:&'ctx ContextReference<'info,'refs,'_,'_>) -> Result<()> {
    pub fn try_load(&mut self, ctx: &ContextReference<'info,'refs,'_,'_>) -> Result<()> {

        let meta = self.meta()?;
        if let Some(account_info) = ctx.locate_index_account(&meta.pubkey) {
            // let container = CollectionStore::<'info,'refs,T>::try_load(account_info)?;
            let container = OrderedCollectionStore::<T>::try_load(account_info)?;
            self.container = Some(container);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::OrderedCollectionNotFound))
        }
    }

    // pub fn try_insert<'t>(&mut self, record: &'t T) -> Result<()> {
    pub fn try_insert(&mut self, record: &T) -> Result<()> {
        if let Some(container) = &self.container {
            container.try_insert(record)?;
            let meta = self.meta_mut()?;
            let count = meta.get_count();
            meta.set_count(count + 1);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::OrderedCollectionNotLoaded))
        }
    }

    // pub fn try_remove(&'info mut self, record: &T) -> Result<()> {
    pub fn try_remove<'t : 'info>(&mut self, record: &'t T) -> Result<()> {
        {
            if self.container.is_none() {
                return Err(error_code!(ErrorCode::OrderedCollectionNotLoaded));
            }

            self.container.as_ref().unwrap().try_remove(record)?;
        }

        let meta = self.meta_mut()?;
        let count = meta.get_count();
        meta.set_count(count - 1);
        Ok(())
    }

    pub fn as_slice(&self) -> Result<&[T]> {
        if let Some(container) = &self.container {
            Ok(container.as_slice())
        } else {
            Err(error_code!(ErrorCode::OrderedCollectionNotLoaded))
        }
    }

    pub fn as_slice_mut(&mut self) -> Result<&mut [T]> {
        if let Some(container) = &mut self.container {
            Ok(container.as_slice_mut())
        } else {
            Err(error_code!(ErrorCode::OrderedCollectionNotLoaded))
        }
    }

    pub fn sync_rent(//<'pid,'instr>(
        &self,
        ctx: &ContextReference<'info,'_,'_,'_>,//<'info,'refs,'pid,'instr>,
        rent_collector : &workflow_allocator::rent::RentCollector<'info,'refs>,
    ) -> workflow_allocator::result::Result<()> {
        // TODO: @alpha - transfer out excess rent
        if let Some(container) = &self.container {
            ctx.sync_rent(container.account(),rent_collector)?;
            Ok(())
        } else {
            Err(error_code!(ErrorCode::OrderedCollectionNotLoaded))
        }
    }


}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct OrderedCollectionStoreMeta {
    pub version : u32,
    pub data_type : u32,
}

#[container(Containers::OrderedCollection)]
pub struct OrderedCollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq {
    pub meta : RefCell<&'info mut OrderedCollectionStoreMeta>,
    pub records : Array<'info, 'refs, T>,
    // _t_ : std::marker::PhantomData<T>,

}

impl<'info, 'refs, T> OrderedCollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq + Ord {

    // pub fn new(ctx:&ContextReference, data_type : u32) -> Result<CollectionStore<'info, 'refs, T>> {
        

    // }

    pub fn try_init(&self, data_type : u32) -> Result<()> {
        let mut meta = self.meta.borrow_mut();
        meta.set_version(1);
        meta.set_data_type(data_type);
        Ok(())
    }

    pub fn data_type(&self) -> u32 {
        self.meta.borrow().get_data_type()
    }

    // pub fn try_insert(&self, record: &T) -> Result<()> {
    //     unsafe { self.records.try_insert(record)?; }
    //     Ok(())
    // }

    pub fn try_insert(&self, record: &T) -> Result<()> where T: 'info {
        match self.records.binary_search(record) {
            Ok(_) => {
                Err(error_code!(ErrorCode::OrderedCollectionCollision))
            },
            Err(idx) => {
                log_trace!("###################################### = idx {} / {}",idx,self.records.len());
                // log_trace!("###################################### = rec {}",record);
                Ok(unsafe { self.records.try_insert_at(idx,record)? })
                // Ok(())
            }
        }
    }

    // pub fn try_remove_with_linear_search(&self, record: &'info T) -> Result<()> {
    //     let slice = self.records.as_slice();
    //     match slice.iter().position(|&r|r==*record) {
    //         Some(idx) => {
    //             unsafe { self.records.try_remove_at(idx,true)?; }
    //         },
    //         None => { }
    //     }
    //     Ok(())
    // }

    pub fn try_remove(&self, record: &'info T) -> Result<()> {
        match self.records.binary_search(record) {
            Ok(idx) => {
                unsafe { self.records.try_remove_at(idx,true)?; }
                Ok(())
            },
            Err(_idx) => {
                Ok(())
            }
        }
    }

    pub fn as_slice(&self) -> &'info [T] {
        self.records.as_slice()
    }

    pub fn as_slice_mut(&mut self) -> &'info mut [T] {
        self.records.as_slice_mut()
    }
}



// ~~~

cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {
        use async_trait::async_trait;
        use workflow_allocator::container::AccountAggregator;
        use solana_program::instruction::AccountMeta;

        #[async_trait(?Send)]
        impl<'info,'refs,T> AccountAggregator for OrderedCollection<'info,'refs,T> 
        where T : Copy + Eq + PartialEq + Ord + 'info
        {
            type Key = T;
            async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
                if key.is_some() {
                    return Err(error_code!(ErrorCode::NotImplemented));
                }
                let meta = self.meta()?;
                Ok(vec![AccountMeta::new(meta.get_pubkey(), false)])
            }

            async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
                if key.is_some() {
                    return Err(error_code!(ErrorCode::NotImplemented));
                }
                let meta = self.meta()?;
                Ok(vec![AccountMeta::new_readonly(meta.get_pubkey(), false)])
            }
        
        }
    }
}
