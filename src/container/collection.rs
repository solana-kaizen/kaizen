use solana_program::pubkey::Pubkey;
use workflow_allocator_macros::{Meta, container};
// use std::rc::Rc;
// use crate::error::ErrorCode;
// use borsh::{BorshDeserialize, BorshSerialize};
use crate::result::Result;
// use crate::container::segment::Segment;
// use crate::identity::*;
use workflow_allocator::prelude::*;
use workflow_allocator::error::ErrorCode;
use workflow_allocator::container::Containers;

// use super::Container;



#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct CollectionMeta {
    pubkey: Pubkey,
    count : u64,
    data_type : u32,
}

impl CollectionMeta {
    pub fn init(&mut self, pubkey : &Pubkey, data_type : u32) {
        self.set_pubkey(*pubkey);
        self.set_data_type(data_type);
    }
}

pub struct Collection<'info,'refs, T> where T : Copy + Eq + PartialEq + Ord {
    pub segment : Rc<Segment<'info,'refs>>,
    // pub meta : &'refs mut CollectionMeta,
    _t_ : std::marker::PhantomData<T>,
    pub container : Option<CollectionStore<'info,'refs, T>>,
}

impl<'info,'refs, T> Collection<'info,'refs, T> 
where T : Copy + Eq + PartialEq + Ord + 'info
{
    pub fn data_len_min() -> usize { std::mem::size_of::<CollectionMeta>() }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Collection<'info,'refs,T>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(Collection {
            segment,
            container : None,
            // meta,
            _t_ : std::marker::PhantomData,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Collection<'info,'refs,T>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(Collection {
            segment,
            container : None,
            // meta,
            _t_ : std::marker::PhantomData,
        })
    }

    pub fn meta<'meta>(&'meta self) -> &'meta mut CollectionMeta {
        self.segment.as_struct_mut_ref::<CollectionMeta>()
    }

    pub fn init(&self, data_type : u32) {
        self.meta().set_data_type(data_type);
        // Ok(())
    }

    pub fn try_create(&self, ctx: &Rc<Context<'info,'refs,'_,'_>>, data_type : u32) -> Result<()> {
        // let data_type = self.meta().get_data_type();
        self.meta().set_data_type(data_type);
        let allocation_args = AccountAllocationArgs::default();
        let collection_store = CollectionStore::<T>::try_allocate(ctx, &allocation_args, 0)?;
        collection_store.try_init(data_type)?;
        Ok(())
    }

    pub fn try_load(&mut self, ctx:&Rc<Context<'info,'refs,'_,'_>>) -> Result<()> {   //Result<CollectionStore<'_,'_,T>> {

        let meta = self.meta();
        if let Some(idx) = ctx.index_accounts.iter().position(|r|r.key==&meta.pubkey) {
            // let container = CollectionStore::<'info,'refs,T>::try_load(&ctx.index_accounts[idx])?;
            // let container : CollectionStore::<'info,'refs,T> = Container::<'info,'refs>::try_load(&ctx.index_accounts[idx])?;
            let container = CollectionStore::<'info,'refs,T>::try_load(&ctx.index_accounts[idx])?;
            // let container : CollectionStore::<'info,'refs,T> = CollectionStore::<'info,'refs,T>::try_load(&ctx.index_accounts[idx])?;
            self.container = Some(container);
            // Ok(collection)
            Ok(())
        } else {
            Err(ErrorCode::CollectionNotFound.into())
        }
    }

    pub fn try_insert<'t>(&self, record: &'t T) -> Result<()> {
        if let Some(container) = &self.container {
            container.try_insert(record)
        } else {
            Err(ErrorCode::CollectionNotFound.into())
        }
    }

    pub fn try_remove(&self, record: &T) -> Result<()> {
        if let Some(container) = &self.container {
            container.try_remove(record)
        } else {
            Err(ErrorCode::CollectionNotFound.into())
        }
    }

    pub fn as_slice(&self) -> Result<&[T]> {
        if let Some(container) = &self.container {
            Ok(container.as_slice())
        } else {
            Err(ErrorCode::CollectionNotFound.into())
        }
    }

    pub fn as_slice_mut(&mut self) -> Result<&mut [T]> {
        if let Some(container) = &mut self.container {
            Ok(container.as_slice_mut())
        } else {
            Err(ErrorCode::CollectionNotFound.into())
        }
    }

    // pub fn as_slice(&self) -> &[T] {
    //     self.records.as_slice()
    // }

    // pub fn as_slice_mut(&mut self) -> &mut [T] {
    //     self.records.as_slice_mut()
    // }


    // pub fn 
}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct CollectionStoreMeta {
    pub version : u32,
    pub data_type : u32,
}

// Recursive expansion of container! macro
// ========================================

// pub struct CollectionStore<'info, 'refs, T>
// where
//     T: Copy + Eq + PartialEq,
// {
//     __store__: workflow_allocator::container::segment::SegmentStore<'info, 'refs>,
//     pub meta: RefCell<&'info mut CollectionStoreMeta>,
//     records: MappedArray<'info, 'refs, T>,
// }
// impl<'info, 'refs, T> CollectionStore<'info, 'refs, T>
// where
//     T: Copy + Eq + PartialEq,
// {
//     pub fn try_allocate_default<'pid, 'instr>(
//         ctx: &std::rc::Rc<workflow_allocator::context::Context<'info, 'refs, 'pid, 'instr>>,
//         allocation_args: &workflow_allocator::context::AccountAllocationArgs<'info, 'refs>,
//     ) -> workflow_allocator::result::Result<Self> {
//         Ok(Self::try_allocate(ctx, allocation_args, 0)?)
//     }
//     pub fn try_allocate<'pid, 'instr>(
//         ctx: &std::rc::Rc<workflow_allocator::context::Context<'info, 'refs, 'pid, 'instr>>,
//         allocation_args: &workflow_allocator::context::AccountAllocationArgs<'info, 'refs>,
//         reserve_data_len: usize,
//     ) -> workflow_allocator::result::Result<Self> {
//         let data_len = Self::initial_data_len() + reserve_data_len;
//         let account_info = ctx.create_pda(data_len, allocation_args)?;
//         Ok(Self::try_create(account_info)?)
//     }
//     pub fn try_create(
//         account: &'refs solana_program::account_info::AccountInfo<'info>,
//     ) -> workflow_allocator::result::Result<CollectionStore<'info, 'refs, T>> {
//         let container_meta_offset =
//             std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
//         let segment_store_offset = std::mem::size_of::<
//             workflow_allocator::container::ContainerHeader,
//         >() + std::mem::size_of::<CollectionStoreMeta>();
//         let container_type: u32 = Containers::Collection as u32;
//         let layout = Self::layout();
//         let __store__ = workflow_allocator::container::segment::SegmentStore::try_create(
//             &account,
//             segment_store_offset,
//             &layout,
//         )?;
//         let meta: RefCell<&'info mut CollectionStoreMeta> = {
//             let mut data = __store__.account.data.borrow_mut();
//             let meta = unsafe { std::mem::transmute(&mut data[container_meta_offset]) };
//             RefCell::new(meta)
//         };
//         let segment = __store__.try_get_segment_at(1usize)?;
//         let records: MappedArray<'info, 'refs, T> = MappedArray::try_create_from_segment(segment)?;
//         {
//             let data = account.data.borrow_mut();
//             let header = unsafe {
//                 std::mem::transmute::<_, &mut workflow_allocator::container::ContainerHeader>(
//                     data.as_ptr(),
//                 )
//             };
//             header.container_type = container_type;
//         }
//         Ok(CollectionStore {
//             __store__,
//             meta,
//             records,
//         })
//     }
//     pub fn try_create_with_layout(
//         account: &'refs solana_program::account_info::AccountInfo<'info>,
//         layout: &workflow_allocator::container::segment::Layout<u16>,
//     ) -> workflow_allocator::result::Result<CollectionStore<'info, 'refs, T>> {
//         let container_meta_offset =
//             std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
//         let segment_store_offset = std::mem::size_of::<
//             workflow_allocator::container::ContainerHeader,
//         >() + std::mem::size_of::<CollectionStoreMeta>();
//         let container_type: u32 = Containers::Collection as u32;
//         let __store__ = workflow_allocator::container::segment::SegmentStore::try_create(
//             &account,
//             segment_store_offset,
//             &layout,
//         )?;
//         let meta: RefCell<&'info mut CollectionStoreMeta> = {
//             let mut data = __store__.account.data.borrow_mut();
//             let meta = unsafe { std::mem::transmute(&mut data[container_meta_offset]) };
//             RefCell::new(meta)
//         };
//         let segment = __store__.try_get_segment_at(1usize)?;
//         let records: MappedArray<'info, 'refs, T> = MappedArray::try_create_from_segment(segment)?;
//         {
//             let data = account.data.borrow_mut();
//             let header = unsafe {
//                 std::mem::transmute::<_, &mut workflow_allocator::container::ContainerHeader>(
//                     data.as_ptr(),
//                 )
//             };
//             header.container_type = container_type;
//         }
//         Ok(CollectionStore {
//             __store__,
//             meta,
//             records,
//         })
//     }
//     #[inline]
//     pub fn layout() -> workflow_allocator::container::segment::Layout<u16> {
//         workflow_allocator::container::segment::Layout::<u16>::from(
//             &CollectionStore::<T>::segments(),
//         )
//     }
//     #[inline]
//     pub fn initial_data_len() -> usize {
//         let container_meta_offset =
//             std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
//         let segment_store_offset = std::mem::size_of::<
//             workflow_allocator::container::ContainerHeader,
//         >() + std::mem::size_of::<CollectionStoreMeta>();
//         CollectionStore::<T>::layout().data_len() + segment_store_offset
//     }
//     #[inline]
//     pub fn sync_rent<'pid, 'instr>(
//         &self,
//         ctx: &std::rc::Rc<workflow_allocator::context::Context<'info, 'refs, 'pid, 'instr>>,
//         rent_collector: &workflow_allocator::rent::RentCollector<'info, 'refs>,
//     ) -> workflow_allocator::result::Result<()> {
//         ctx.sync_rent(self.account(), rent_collector)?;
//         Ok(())
//     }
//     #[inline]
//     pub fn purge<'pid, 'instr>(
//         &self,
//         ctx: &std::rc::Rc<workflow_allocator::context::Context<'info, 'refs, 'pid, 'instr>>,
//         rent_collector: &workflow_allocator::rent::RentCollector<'info, 'refs>,
//     ) -> workflow_allocator::result::Result<()> {
//         ctx.purge(self.account(), rent_collector)?;
//         Ok(())
//     }
//     #[inline]
//     pub fn account(&self) -> &'refs solana_program::account_info::AccountInfo<'info> {
//         self.__store__.account
//     }
//     #[inline]
//     pub fn pubkey(&self) -> &solana_program::pubkey::Pubkey {
//         self.__store__.account.key
//     }
// }
// impl<'info, 'refs, T> CollectionStore<'info, 'refs, T>
// where
//     T: Copy + Eq + PartialEq,
// {
//     fn segments() -> [usize; 1usize] {
//         [MappedArray::<T>::data_len_min()]
//     }
// }
// impl<'info, 'refs, T> workflow_allocator::container::Container<'info,'refs> for CollectionStore<'info, 'refs, T>
// where
//     T: Copy + Eq + PartialEq,
// {
//     type T = Self;
//     fn try_load(
//         account: &'refs solana_program::account_info::AccountInfo<'info>,
//     ) -> workflow_allocator::result::Result<CollectionStore<'info, 'refs, T>> {
//         let container_meta_offset =
//             std::mem::size_of::<workflow_allocator::container::ContainerHeader>();
//         let segment_store_offset = std::mem::size_of::<
//             workflow_allocator::container::ContainerHeader,
//         >() + std::mem::size_of::<CollectionStoreMeta>();
//         let container_type: u32 = Containers::Collection as u32;
//         let layout = Self::layout();
//         let __store__ = workflow_allocator::container::segment::SegmentStore::try_load(
//             &account,
//             segment_store_offset,
//         )?;
//         {
//             let data = account.data.borrow_mut();
//             let header = unsafe {
//                 std::mem::transmute::<_, &mut workflow_allocator::container::ContainerHeader>(
//                     data.as_ptr(),
//                 )
//             };
//             if header.container_type != container_type {
//                 return Err(workflow_allocator::error::Error::new()
//                     .with_program_code(
//                         workflow_allocator::error::ErrorCode::ContainerTypeMismatch as u32,
//                     )
//                     .with_source(file!(), line!()));
//             }
//         }
//         let meta: RefCell<&'info mut CollectionStoreMeta> = {
//             let mut data = __store__.account.data.borrow_mut();
//             let meta = unsafe { std::mem::transmute(&mut data[container_meta_offset]) };
//             RefCell::new(meta)
//         };
//         let segment = __store__.try_get_segment_at(1usize)?;
//         let records: MappedArray<'info, 'refs, T> = MappedArray::try_load_from_segment(segment)?;
//         Ok(CollectionStore {
//             __store__,
//             meta,
//             records,
//         })
//     }
// }
// #[cfg(not(any(target_arch = "bpf", target_arch = "wasm32")))]
// inventory::submit! {
//   workflow_allocator::container::registry::ContainerDeclaration::new(Containers::Collection as u32,"CollectionStore",)
// }
// #[cfg(target_arch = "wasm32")]
// #[macro_use]
// mod init_collectionstore {
//     use super::*;
//     #[cfg(target_arch = "wasm32")]
//     #[wasm_bindgen::prelude::wasm_bindgen]
//     pub fn container_declaration_register_collectionstore() -> workflow_allocator::result::Result<()>
//     {
//         let container_declaration =
//             workflow_allocator::container::registry::ContainerDeclaration::new(
//                 Containers::Collection as u32,
//                 "CollectionStore",
//             );
//         workflow_allocator::container::registry::register_container_declaration(
//             container_declaration,
//         )?;
//         Ok(())
//     }
// }


#[container(Containers::Collection)]
pub struct CollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq {
    pub meta : RefCell<&'info mut CollectionStoreMeta>,
    records : Array<'info, 'refs, T>,
    // _t_ : std::marker::PhantomData<T>,

}

impl<'info, 'refs, T> CollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq + Ord + 'info {

    // pub fn new(ctx:&Rc<Context>, data_type : u32) -> Result<CollectionStore<'info, 'refs, T>> {
        

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

    pub fn try_remove(&self, record: &T) -> Result<()> {
        let slice = self.records.as_slice();
        match slice.iter().position(|&r|r==*record) {
            Some(idx) => {
                unsafe { self.records.try_remove_at(idx,true)?; }
            },
            None => { }
        }
        Ok(())
    }

    pub fn as_slice(&self) -> &[T] {
        self.records.as_slice()
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self.records.as_slice_mut()
    }
}

// #[derive(Meta)]
// pub struct Collection<'data, T> where T : Copy {
//     identity_record : 
//     // data_type : &'data u32,
//     // pubkey : &'data Pubkey,
//     // count : &'data u64,
//     _t_ : std::marker::PhantomData<T>,
// }

// impl<'data,T> From<&'data IdentityRecord> for Collection<'data,T> where T : Copy {
//     fn from(r : &'data IdentityRecord) -> Self {
//         Self {
//             // data_type : &r.data_type,
//             // pubkey : &r.pubkey,
//             // count : &r.meta,
//             _t_ : std::marker::PhantomData,
//         }
//     }
// }

// pub struct Collection<'info,'refs,T> {
//     pub segment : Rc<Segment<'info,'refs>>,
//     _t_ : std::marker::PhantomData<T>,
// }

// impl<'info,'refs,T> Collection<'info,'refs,T>
// where T: Copy
// {
//     pub fn try_create_from_segment(
//         segment : Rc<Segment<'info, 'refs>>
//     ) -> Result<Collection<'info, 'refs, T>> {

//         segment.try_resize(std::mem::size_of::<CollectionMeta>(), false)?;
//         let collection = Collection { 
//             segment,
//             _t_ : std::marker::PhantomData,
//         };

//         Ok(collection)
//     }

//     pub fn try_load_from_segment(
//             segment : Rc<Segment<'info, 'refs>>
//     ) -> Result<Collection<'info, 'refs, T>> {

//         if segment.get_data_len() < std::mem::size_of::<CollectionMeta>() {
//             return Err(ErrorCode::CollectionMetaSegmentSizeTooSmall.into());
//         }

//         // let store = MappedArray {
//         //     account : segment.store.account,
//         //     segment : segment.clone(),
//         //     phantom : PhantomData,
//         //     // TODO: realloc_on_remove: false,
//         // };

//         Ok(Collection { store })
//     }

//     pub fn data_len_min() -> usize {
//         std::mem::size_of::<MappedArrayMeta>()
//     }    

// }

// #[derive(Debug)]
// pub struct Collection<'info, 'refs, T> 
// where T : Copy
// {
//     store : MappedArray<'info, 'refs, T>
//     // pub account : &'refs AccountInfo<'info>,
//     // pub segment : Rc<Segment<'info, 'refs>>,
//     // phantom: PhantomData<&'refs T>,
// }

// impl<'info, 'refs, T> Collection<'info, 'refs, T> 
// where T: Copy
// {

//     pub fn try_create_from_segment(
//         segment : Rc<Segment<'info, 'refs>>
//     ) -> Result<Collection<'info, 'refs, T>> {

//         let store = MappedArray::try_create_from_segment(segment)?;
//         // let store = Self::try_load_from_segment(segment)?;
//         // store.try_init_meta()?;

//         let collection = Collection { store };

//         Ok(collection)
//     }

//     pub fn try_load_from_segment(
//             segment : Rc<Segment<'info, 'refs>>
//     ) -> Result<Collection<'info, 'refs, T>> {

//         let collection = MappedArray::try_load_from_segment(segment)?;

//         // if segment.get_data_len() < mem::size_of::<MappedArrayMeta>() {
//         //     return Err(ErrorCode::MappedArraySegmentSizeTooSmall.into());
//         // }

//         // let store = MappedArray {
//         //     account : segment.store.account,
//         //     segment : segment.clone(),
//         //     phantom : PhantomData,
//         //     // TODO: realloc_on_remove: false,
//         // };

//         Ok(Collection { store })
//     }

//     pub fn data_len_min() -> usize {
//         std::mem::size_of::<MappedArrayMeta>()
//     }
// }