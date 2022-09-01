use solana_program::pubkey::Pubkey;
use workflow_allocator_macros::{Meta, container};
// use std::rc::Rc;
// use crate::error::ErrorCode;
// use borsh::{BorshDeserialize, BorshSerialize};
use crate::result::Result;
// use crate::container::segment::Segment;
// use crate::identity::*;
use workflow_allocator::prelude::*;
use workflow_allocator::container::Containers;



#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct Collection {
    pubkey: Pubkey,
    count : u64,
    data_type : u32,
}

impl Collection {
    pub fn init(&mut self, pubkey : &Pubkey, data_type : u32) {
        self.set_pubkey(*pubkey);
        self.set_data_type(data_type);
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
    records : MappedArray<'info, 'refs, T>,
    // _t_ : std::marker::PhantomData<T>,

}

impl<'info, 'refs, T> CollectionStore<'info, 'refs, T> where T : Copy + Eq + PartialEq + Ord + 'static {

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
                unsafe { self.records.try_remove_at(idx,true,false)?; }
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