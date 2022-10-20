use std::cmp::Ordering;

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

// use super::PubkeyReference;
// use super::Container;

pub type PubkeyCollection<'info,'refs> = PubkeyCollectionInterface<'info,'refs, PubkeyCollectionSegmentInterface<'info,'refs>>;
pub type PubkeyCollectionReference<'info,'refs> = PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>;


#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyCollectionMeta {
    pubkey: Pubkey,
    collection_len : u64,
    sequence : u64,
    data_type : u32,
    container_type : u32,
}

// fn v<V:Into<u32>>(v:Option<V>) -> u32 {
//     match v {
//         Some(v) => v.into(),
//         None => 0,
//     }
// }

impl PubkeyCollectionMeta {
    // pub fn try_create<V : Into<u32>>(
    pub fn try_create(
        &mut self,
        pubkey : &Pubkey,
        data_type : Option<u32>,
        container_type : Option<u32>,
    ) -> Result<()> {
        self.pubkey = *pubkey;
        // let data_type : u32 = data_type.unwrap_or(0 as V);
        // self.set_data_type(data_type.unwrap_or(0u32).into());
        // self.set_data_type(v(data_type));
        self.set_data_type(data_type.unwrap_or(0));
        self.set_container_type(container_type.unwrap_or(0));
        Ok(())
    }

    pub fn advance_sequence(&mut self) -> u32 {
        let seq = self.get_sequence() + 1;
        self.set_sequence(seq);
        seq as u32
    }

}

pub trait PubkeyCollectionMetaTrait {
    fn try_create(
        &mut self,
        pubkey : &Pubkey,
        data_type : Option<u32>,
        container_type : Option<u32>,
    ) -> Result<()>;
    fn try_load(&mut self) -> Result<()>;
    fn min_data_len() -> usize;
    fn pubkey<'key>(&'key self) -> &'key Pubkey;
    fn get_len(&self) -> u64;
    fn set_len(&mut self, count: u64);
    fn advance_sequence(&mut self) -> u32;
    fn get_data_type(&self) -> Option<u32>;
    fn get_container_type(&self) -> Option<u32>;
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


pub struct PubkeyCollectionMetaInterface<'data> {
    data : &'data mut PubkeyCollectionMeta,
}

impl<'data> PubkeyCollectionMetaInterface<'data> {

    pub fn new(
        data : &'data mut PubkeyCollectionMeta,
    ) -> Self {
        Self { 
            data,
        }
    }

    pub fn data_ref<'t>(&'t self) -> &'t PubkeyCollectionMeta {
        self.data
    }

    pub fn data_mut<'t>(&'t mut self) -> &'t mut PubkeyCollectionMeta {
        self.data
    }
}



impl<'info> PubkeyCollectionMetaTrait for PubkeyCollectionMetaInterface<'info> {
    fn try_create(
        &mut self,
        pubkey : &Pubkey,
        data_type : Option<u32>,
        container_type : Option<u32>,
    ) -> Result<()> {
        self.data_mut().try_create(pubkey,data_type,container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        // self.seed = self.data_ref().get_seed();
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<PubkeyCollectionMeta>()
    }

    fn pubkey<'key>(&'key self) -> &'key Pubkey {
        &self.data_ref().pubkey
    }
    
    fn get_len(&self) -> u64 {
        self.data_ref().get_collection_len()
    }
    
    fn set_len(&mut self, len : u64) {
        self.data_mut().set_collection_len(len);
    }
    
    fn advance_sequence(&mut self) -> u32 {
        self.data_mut().advance_sequence()
    }

    fn get_data_type(&self) -> Option<u32> {
        let data_type = self.data_ref().get_data_type();
        if data_type == 0 {
            None
        } else {
            Some(data_type)
        }
    }

    fn get_container_type(&self) -> Option<u32> {
        let container_type = self.data_ref().get_container_type();
        if container_type == 0 {
            None
        } else {
            Some(container_type)
        }
    }

}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


pub struct PubkeyCollectionSegmentInterface<'info,'refs> {
    segment : Rc<Segment<'info,'refs>>,
}

impl<'info,'refs> PubkeyCollectionSegmentInterface<'info,'refs> {
    pub fn new(
        segment : Rc<Segment<'info,'refs>>,
    ) -> Self {
        Self {
            segment,
        }
    }

    pub fn data_ref<'data>(&'data self) -> &'data PubkeyCollectionMeta {
        self.segment.as_struct_ref::<PubkeyCollectionMeta>()
    }

    pub fn data_mut<'data>(&'data self) -> &'data mut PubkeyCollectionMeta {
        self.segment.as_struct_mut::<PubkeyCollectionMeta>()
    }
}

impl<'info,'refs> PubkeyCollectionMetaTrait for PubkeyCollectionSegmentInterface<'info,'refs> {

    fn try_create(
        &mut self,
        pubkey : &Pubkey,
        data_type : Option<u32>,
        container_type : Option<u32>,
    ) -> Result<()> {
        self.data_mut().try_create(pubkey,data_type,container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<PubkeyCollectionMeta>()
    }

    fn pubkey<'key>(&'key self) -> &'key Pubkey {
        &self.data_ref().pubkey
    }

    fn get_len(&self) -> u64 {
        self.data_ref().get_collection_len()
    }
    
    fn set_len(&mut self, len : u64) {
        self.data_mut().set_collection_len(len)
    }
    
    fn advance_sequence(&mut self) -> u32 {
        self.data_mut().advance_sequence()
    }

    fn get_data_type(&self) -> Option<u32> {
        let data_type = self.data_ref().get_data_type();
        if data_type == 0 {
            None
        } else {
            Some(data_type)
        }
    }

    fn get_container_type(&self) -> Option<u32> {
        let container_type = self.data_ref().get_container_type();
        if container_type == 0 {
            None
        } else {
            Some(container_type)
        }
    }

}


// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyReference {
    seq : u32,
    pub key : Pubkey
}


// impl From<(u32, Pubkey)> for PubkeyReference {
//     fn from((seq, key): (u32, Pubkey)) -> Self {
//         PubkeyReference { seq, key }
//     }
// }

// impl From<(Instant, &Pubkey)> for PubkeyReference {
//     fn from((ts, key): (Instant, &Pubkey)) -> Self {
//         PubkeyReference { ts : ts.0, key : *key }
//     }
// }

// ~

impl Ord for PubkeyReference {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.seq, &self.key).cmp(&(other.seq, &other.key))
    }
}

impl PartialOrd for PubkeyReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PubkeyReference {
    fn eq(&self, other: &Self) -> bool {
        (self.seq, &self.key) == (other.seq, &other.key)
    }
}

impl Eq for PubkeyReference { }

// impl CollectionMeta {
//     pub fn init(&mut self, pubkey : &Pubkey, data_type : u32) {
//         self.set_pubkey(*pubkey);
//         self.set_data_type(data_type);
//         self.set_count(0);
//     }
// }



pub struct PubkeyCollectionInterface<'info,'refs, M> 
where M : PubkeyCollectionMetaTrait 
// where T : Copy + Eq + PartialEq + Ord 
{
    meta : M,
    // pub external_meta : Option<&'info mut OrderedCollectionMeta>,
    // pub external_meta : Option<&'refs mut PubkeyCollectionMeta>,
    // pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
    pub container : Option<PubkeyCollectionStore<'info,'refs>>,
    // _t_ : std::marker::PhantomData<T>,
}


impl<'info,'refs, M> PubkeyCollectionInterface<'info,'refs, M> 
where M : PubkeyCollectionMetaTrait
// where T : Copy + Eq + PartialEq + Ord + 'info + 'refs
{


    pub fn try_new(
        meta:M,
        // container : Option<PubkeyCollectionStore<'info,'refs>>
    )->Result<Self> {
        // meta.try_create()?; // seed,container_type)?;
        Ok(Self { meta, container : None })
    }

    // fn try_create_impl(
    //     mut meta:M,
    // )->Result<Self> {
    //     meta.try_create()?; // seed,container_type)?;
    //     Ok(Self { meta, container : None })
    // }

    // fn try_load_impl(
    //     mut meta:M,
    // )->Result<Self> {
    //     meta.try_load()?;
    //     Ok(Self { meta, container : None })
    // }

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

    // pub fn try_load<'i,'r>(
    pub fn try_load(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,//<'i,'r,'_,'_>
    ) -> Result<()> {

        // let meta = self.meta()?;
        if let Some(account_info) = ctx.locate_index_account(self.meta.pubkey()) {
            // let container = CollectionStore::<'info,'refs,T>::try_load(account_info)?;
            let container = PubkeyCollectionStore::try_load(account_info)?;
            self.container = Some(container);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotFound))
        }
    }

    // pub fn try_load<'i,'r>(
    //     &mut self,
    //     ctx: &ContextReference<'i,'r,'_,'_>,
    //     // allocation_args: &AccountAllocationArgs<'i,'r,'_>,
    //     // data_type : Option<u32>,
    //     // container_type : Option<u32>
    // ) -> Result<()> {
    //     // let collection_store = PubkeyCollectionStore::try_allocate(ctx, allocation_args, 0)?;
    //     // self.meta.try_create(collection_store.pubkey(), data_type, container_type)?;
    //     // Ok(())

    //     let account_info = ctx.locate_index_account(self.meta.pubkey())
    //         .ok_or(error_code!(ErrorCode::AccountCollectionNotFound))?;
    //     let collection_store = PubkeyCollectionStore::try_load(account_info)?;
    //     self.container = Some(collection_store);
    //     Ok(())
    // }


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
        // ctx: &ContextReference<'ctx,'r,'_,'_>,
        // allocation_args: &AccountAllocationArgs<'ctx,'_,'_>,
        data : &'info mut PubkeyCollectionMeta,
        // data_type : Option<u32>,
        // container_type : Option<u32>,

    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {


        let mut collection = PubkeyCollectionInterface::<'info,'refs,PubkeyCollectionMetaInterface<'info>>::try_from_meta(data)?;
        collection.try_load(ctx)?;//, allocation_args, data_type, container_type)?;
        Ok(collection)

    }
    // pub fn try_create_from_meta<'ctx,'r>(
    //     ctx: &ContextReference<'ctx,'r,'_,'_>,
    //     allocation_args: &AccountAllocationArgs<'ctx,'_,'_>,
    //     data : &'info mut PubkeyCollectionMeta,
    //     data_type : Option<u32>,
    //     container_type : Option<u32>,

    // ) -> Result<PubkeyCollectionInterface<'ctx,'r, PubkeyCollectionMetaInterface<'ctx>>> {

    //     let collection_store = PubkeyCollectionStore::try_allocate(ctx, allocation_args, 0)?;
    //     collection_store.try_init(container_type)?;
    //     let mut meta = PubkeyCollectionMetaInterface::new(data);
    //     meta.try_create(collection_store.pubkey(), data_type, container_type)?;
    //     PubkeyCollectionInterface::<PubkeyCollectionMetaInterface>::try_new(
    //         meta,
    //         Some(collection_store)
    //     )
    // }

    pub fn try_from_meta<'ctx,'r>(
        // ctx: &ContextReference<'ctx,'r,'_,'_>,
        // allocation_args: &AccountAllocationArgs<'ctx,'_,'_>,
        data : &'info mut PubkeyCollectionMeta,
        // data_type : Option<u32>,
        // container_type : Option<u32>,

    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {

        let meta = PubkeyCollectionMetaInterface::new(data);
        PubkeyCollectionInterface::<PubkeyCollectionMetaInterface>::try_new(
            meta,
            // None
        )
    }

    // pub fn try_load_from_meta<'ctx>(
    //     ctx: &ContextReference<'ctx,'_,'_,'_>,
    //     // ctx: &ContextReference<'info,'refs,'_,'_>,
    //     data : &'info mut PubkeyCollectionMeta,
    //     // account_info : &AccountInfo<'info>,
    //     // seed : &'static [u8],
    //     // container_type : Option<u32>,
    // ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {
    // // ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionMetaInterface<'info>>> {

    //     let meta = PubkeyCollectionMetaInterface::new(
    //         data,
            
    //     );


    //     let account_info = ctx.locate_index_account(meta.pubkey())
    //         .ok_or(error_code!(ErrorCode::AccountCollectionNotFound))?;
    //     let collection_store = PubkeyCollectionStore::try_load(account_info)?;
        

    //     PubkeyCollectionInterface::<PubkeyCollectionMetaInterface>::try_new(meta,
    //         Some(collection_store)
    //         // account_info.key.as_ref(),
    //         // PubkeyCollectionMetaInterface::new(
    //         //     data,
    //         // )
    //     )
    // }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>,
    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>> {
        PubkeyCollectionInterface::<PubkeyCollectionSegmentInterface>::try_new(
            PubkeyCollectionSegmentInterface::new(
                segment
            ),
            // None
        )
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>,
            // seed : &'static [u8],
            // container_type : Option<u32>,
    ) -> Result<PubkeyCollectionInterface<'info,'refs, PubkeyCollectionSegmentInterface<'info, 'refs>>> {
        PubkeyCollectionInterface::<PubkeyCollectionSegmentInterface>::try_new(
            PubkeyCollectionSegmentInterface::new(
                segment,
            ),
            // None
        )
    }

    // pub fn try_create(
    //     &mut self,
    //     // seed : &[u8],
    //     // container_type : Option<u32>,
    // ) -> Result<()> {
    //     self.meta.try_create()//seed, container_type)
    // }

    pub fn len(&self) -> usize {
        self.meta.get_len() as usize
    }



// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // pub fn meta<'meta>(&'meta self) -> Result<&'meta PubkeyCollectionMeta> {
    //     if let Some(external_meta) = &self.external_meta {
    //         return Ok(external_meta);
    //     } else if let Some(segment) = &self.segment_meta {
    //         Ok(segment.as_struct_ref::<PubkeyCollectionMeta>())
    //     } else {
    //         Err(ErrorCode::OrderedCollectionMissingMeta.into())
    //     }
    // }

    // pub fn meta_mut<'meta>(&'meta mut self) -> Result<&'meta mut PubkeyCollectionMeta> {
    //     if let Some(external_meta) = &mut self.external_meta {
    //         return Ok(external_meta);
    //     } else if let Some(segment) = &self.segment_meta {
    //         Ok(segment.as_struct_mut::<PubkeyCollectionMeta>())
    //     } else {
    //         Err(ErrorCode::OrderedCollectionMissingMeta.into())
    //     }
    // }

    // pub fn data_len_min() -> usize { std::mem::size_of::<PubkeyCollectionMeta>() }

    // pub fn try_from_meta(meta : &'info mut OrderedCollectionMeta) -> Result<Self> {
    // pub fn try_from_meta(meta : &'refs mut PubkeyCollectionMeta) -> Result<Self> {
    //     Ok(OrderedCollectionInterface {
    //         segment_meta : None,
    //         external_meta : Some(meta),
    //         container : None
    //     })
    // }

    // pub fn try_create_from_segment(
    //     segment : Rc<Segment<'info, 'refs>>
    // ) -> Result<OrderedCollectionInterface<'info,'refs,T>> {
    //     // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
    //     Ok(OrderedCollectionInterface {
    //         segment_meta : Some(segment),
    //         external_meta : None,
    //         container : None
    //     })
    // }

    // pub fn try_load_from_segment(
    //         segment : Rc<Segment<'info, 'refs>>
    // ) -> Result<OrderedCollectionInterface<'info,'refs,T>> {
    //     // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
    //     Ok(OrderedCollectionInterface {
    //         segment_meta : Some(segment),
    //         external_meta : None,
    //         container : None
    //     })
    // }


    // pub fn try_load<'ctx>(&mut self, ctx:&'ctx ContextReference<'info,'refs,'_,'_>) -> Result<()> {
    // pub fn try_load(&mut self, ctx: &ContextReference<'info,'refs,'_,'_>) -> Result<()> {
    // pub fn try_load<'i,'r>(&mut self, ctx: &ContextReference<'i,'r,'_,'_>) -> Result<()> {

    // pub fn try_insert<'t>(&mut self, record: &'t T) -> Result<()> {
    // pub fn try_insert(&mut self, record: &PubkeyReference) -> Result<()> {
    pub fn try_insert(&mut self, key: &Pubkey) -> Result<()> {
        if let Some(container) = &self.container {
            
            let seq = self.meta.advance_sequence();
            container.try_insert(seq,key)?;
            let len = self.meta.get_len();
            self.meta.set_len(len + 1);
            
            Ok(())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
        }
    }

    // pub fn try_remove(&'info mut self, record: &T) -> Result<()> {
    // pub fn try_remove<'t : 'info>(&mut self, record: &'t T) -> Result<()> {
    pub fn try_remove(&mut self, record: &PubkeyReference) -> Result<()> {
        {
            if self.container.is_none() {
                return Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded));
            }

            self.container.as_ref().unwrap().try_remove(record)?;
        }

        // let meta = self.meta_mut()?;
        let len = self.meta.get_len();
        self.meta.set_len(len - 1);
        Ok(())
    }

    pub fn as_slice(&self) -> Result<&[PubkeyReference]> {
        if let Some(container) = &self.container {
            Ok(container.as_slice())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
        }
    }

    pub fn as_slice_mut(&mut self) -> Result<&mut [PubkeyReference]> {
        if let Some(container) = &mut self.container {
            Ok(container.as_slice_mut())
        } else {
            Err(error_code!(ErrorCode::PubkeyCollectionNotLoaded))
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
    pub records : Array<'info, 'refs, PubkeyReference>,
}

impl<'info, 'refs> PubkeyCollectionStore<'info, 'refs> {

    pub fn try_init(&self, container_type : Option<u32>) -> Result<()> {
        let mut meta = self.meta.borrow_mut();
        meta.set_version(1);
        meta.set_container_type(container_type.unwrap_or(0u32));
        Ok(())
    }

    // pub fn data_type(&self) -> u32 {
    //     self.meta.borrow().get_data_type()
    // }

    // pub fn try_insert(&self, record: &T) -> Result<()> {
    //     unsafe { self.records.try_insert(record)?; }
    //     Ok(())
    // }

    // fn try_insert(&self, reference: &PubkeyReference) -> Result<()> {
    fn try_insert(&self, seq: u32, key: &Pubkey) -> Result<()> {

        let record = unsafe { self.records.try_allocate(false)? };
        record.set_seq(seq);
        record.key = *key;
        // match self.records.binary_search(reference) {
        //     Ok(_) => {
        //         Err(error_code!(ErrorCode::PubkeyCollectionCollision))
        //     },
        //     Err(idx) => {
        //         log_trace!("###################################### = idx {} / {}",idx,self.records.len());
        //         // log_trace!("###################################### = rec {}",record);
        //         Ok(unsafe { self.records.try_insert_at(idx,reference)? })
        //         // Ok(())
        //     }
        // }
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

    pub fn try_remove(&self, reference: &PubkeyReference) -> Result<()> {
        match self.records.binary_search(reference) {
            Ok(idx) => {
                unsafe { self.records.try_remove_at(idx,true)?; }
                Ok(())
            },
            Err(_idx) => {
                Ok(())
            }
        }
    }

    pub fn as_slice(&self) -> &'info [PubkeyReference] {
        self.records.as_slice()
    }

    pub fn as_slice_mut(&mut self) -> &'info mut [PubkeyReference] {
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
