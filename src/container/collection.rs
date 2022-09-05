use cfg_if::cfg_if;
use solana_program::pubkey::Pubkey;
use workflow_allocator_macros::{Meta, container};
use crate::context::ContextReference;
// use std::rc::Rc;
// use crate::error::ErrorCode;
// use borsh::{BorshDeserialize, BorshSerialize};
use crate::result::Result;
// use crate::container::segment::Segment;
// use crate::identity::*;
use workflow_allocator::prelude::*;
use workflow_allocator::error::ErrorCode;
use workflow_allocator::container::Containers;
use workflow_allocator::container::AccountAggregator;
use async_trait::async_trait;

// use super::TsPubkey;
// use super::Container;


#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct CollectionMeta {
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

pub struct Collection<'info,'refs, T> 
where T : Copy + Eq + PartialEq + Ord 
{
    pub external_meta : Option<&'info mut CollectionMeta>,
    pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
    pub container : Option<CollectionStore<'info,'refs, T>>,
    // _t_ : std::marker::PhantomData<T>,
}


impl<'info,'refs, T> Collection<'info,'refs, T> 
where T : Copy + Eq + PartialEq + Ord + 'info + 'refs
{
    pub fn meta<'meta>(&'meta self) -> Result<&'meta CollectionMeta> {
        if let Some(external_meta) = &self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_ref::<CollectionMeta>())
        } else {
            Err(ErrorCode::CollectionMissingMeta.into())
        }
    }

    pub fn meta_mut<'meta>(&'meta mut self) -> Result<&'meta mut CollectionMeta> {
        if let Some(external_meta) = &mut self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_mut::<CollectionMeta>())
        } else {
            Err(ErrorCode::CollectionMissingMeta.into())
        }
    }

    pub fn data_len_min() -> usize { std::mem::size_of::<CollectionMeta>() }

    pub fn try_from_meta(meta : &'info mut CollectionMeta) -> Result<Self> {
        Ok(Collection {
            segment_meta : None,
            external_meta : Some(meta),
            container : None
        })
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Collection<'info,'refs,T>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(Collection {
            segment_meta : Some(segment),
            external_meta : None,
            container : None
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Collection<'info,'refs,T>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(Collection {
            segment_meta : Some(segment),
            external_meta : None,
            container : None
        })
    }

    pub fn try_create(&mut self, ctx: &ContextReference, data_type : u32) -> Result<()> {
        // let data_type = self.meta().get_data_type();
        let allocation_args = AccountAllocationArgs::default();
        let collection_store = CollectionStore::<T>::try_allocate(ctx, &allocation_args, 0)?;
        collection_store.try_init(data_type)?;
        let meta = self.meta_mut()?;
        meta.set_data_type(data_type);
        meta.set_pubkey(*collection_store.pubkey());

        Ok(())
        // Ok(collection_store)
    }

    pub fn try_load<'ctx>(&mut self, ctx:&'ctx ContextReference<'info,'refs,'_,'_>) -> Result<()> {

        let meta = self.meta()?;
        if let Some(account_info) = ctx.locate_index_account(&meta.pubkey) {
            // let container = CollectionStore::<'info,'refs,T>::try_load(account_info)?;
            let container = CollectionStore::<T>::try_load(account_info)?;
            self.container = Some(container);
            Ok(())
        } else {
            Err(ErrorCode::CollectionNotFound.into())
        }
    }

    pub fn try_insert<'t>(&mut self, record: &'t T) -> Result<()> {
        if let Some(container) = &self.container {
            container.try_insert(record)?;
            let meta = self.meta_mut()?;
            let count = meta.get_count();
            meta.set_count(count + 1);
            Ok(())
        } else {
            Err(ErrorCode::CollectionNotLoaded.into())
        }
    }

    // pub fn try_remove(&'info mut self, record: &T) -> Result<()> {
    pub fn try_remove<'t : 'info>(&mut self, record: &'t T) -> Result<()> {
        {
            if self.container.is_none() {
                return Err(ErrorCode::CollectionNotLoaded.into());
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
            Err(ErrorCode::CollectionNotLoaded.into())
        }
    }

    pub fn as_slice_mut(&mut self) -> Result<&mut [T]> {
        if let Some(container) = &mut self.container {
            Ok(container.as_slice_mut())
        } else {
            Err(ErrorCode::CollectionNotLoaded.into())
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
            Err(ErrorCode::CollectionNotLoaded.into())
        }
    }


}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct CollectionStoreMeta {
    pub version : u32,
    pub data_type : u32,
}

#[container(Containers::Collection)]
pub struct CollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq {
    pub meta : RefCell<&'info mut CollectionStoreMeta>,
    pub records : Array<'info, 'refs, T>,
    // _t_ : std::marker::PhantomData<T>,

}

impl<'info, 'refs, T> CollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq + Ord {

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

    pub fn try_insert(&self, record: &T) -> Result<()> {
        unsafe { self.records.try_insert(record)?; }
        Ok(())
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

        #[async_trait(?Send)]
        impl<'info,'refs,T> AccountAggregator for Collection<'info,'refs,T> 
        where T : Copy + Eq + PartialEq + Ord + 'info
        {
            type Key = T;
            async fn locate_account_pubkeys(&self, _: &Self::Key) -> Result<Vec<Pubkey>> {
                let meta = self.meta()?;
                Ok(vec![meta.get_pubkey()])
            }
        
        }
    }
}
