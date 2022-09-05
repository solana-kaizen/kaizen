use workflow_log::*;
// use crate::accounts::LamportAllocation;
use crate::error;
use crate::error::*;
use crate::result::*;
use crate::error_code;
use crate::prelude::*;
use crate::program_error_code;
use crate::container::segment::*;
use crate::container::Containers;
#[cfg(not(target_arch = "bpf"))]
use crate::transport::Interface;
// use crate::enums::*;
// use crate::container::*;
// use std::cell::RefCell;
use std::cmp::Ordering;
use std::marker::PhantomData;
// use std::marker::PhantomData;
// use std::rc::Rc;
// pub use crate::segment::{Segment, SegmentStore, Layout};
// pub use crate::container::ContainerHeader;
// pub use solana_program::account_info::AccountInfo;

// #[cfg(not(target_arch = "bpf"))]
use workflow_log::style;

pub use workflow_allocator_macros::container;


pub const BPTREE_MAX_INDEX_ITEMS_CAPACITY: usize = 1024;
// pub const BPTREE_INDEX_ASSOC_THRESHOLD: usize = BPTREE_MAX_INDEX_ITEMS_CAPACITY / 4;
pub const BPTREE_MAX_VALUE_ITEMS_CAPACITY: usize = 1024;
// pub const BPTREE_VALUE_ASSOC_THRESHOLD: usize = BPTREE_MAX_VALUE_ITEMS_CAPACITY / 4;


// pub trait Key : Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display + Default { }
// pub trait Value : Default + Clone + Copy + std::fmt::Debug + std::fmt::Display { }


// pub fn calc_threshold_for_assoc(capacity: usize, levels: usize) {
//     if levels < 2 {
//         capacity / 5
//     } else {

//     }
// }



#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct BPTreeIdentifier {
    pub pubkey : Pubkey,
    pub segment : u16,
    // pub uid : u16,
}

// #[derive(Debug, Clone, Copy)]
// #[repr(u8)]
// pub enum Flags {
//     Leaf = 1,
// }

// const FLAGS_HAS_ASSOC: u16 = 0x0001;


#[derive(Debug, Copy, Clone)]
#[repr(packed)]
pub struct BPTreeMetaIndex {
    pub version : u16,
    pub capacity : u16,
    // pub root : Pubkey,
    pub last : Pubkey,
    pub assoc : Pubkey,
    pub flags : u16,

}

impl BPTreeMetaIndex {

    // pub fn new(
    //     capacity : usize,
    //     last : Pubkey,
    //     root:Pubkey,
    // ) -> BPTreeMetaIndex {
    //     assert!(capacity < 0xffff);
    //     BPTreeMetaIndex {
    //         version : 1u16,
    //         capacity : capacity as u16,
    //         root,
    //         last
    //     }
    // }


    pub fn try_init(&mut self, capacity: usize, _owner : &Pubkey) -> Result<()> {
        if self.version != 0u16 {
            return Err(ErrorCode::ContainerMetaVersionMismatch.into())
        }

        self.version = 1u16;
        self.capacity = capacity as u16;
        // self.owner = *owner;

        Ok(())
    }

}


// ^ TODO: USE LEVELS AS FACTOR IN WHEN ASSOCIATED ACCOUNT IS CREATED
// ^ TODO: USE LEVELS AS FACTOR IN WHEN ASSOCIATED ACCOUNT IS CREATED
// ^ TODO: USE LEVELS AS FACTOR IN WHEN ASSOCIATED ACCOUNT IS CREATED
// ^ TODO: USE LEVELS AS FACTOR IN WHEN ASSOCIATED ACCOUNT IS CREATED


#[derive(Debug, Copy, Clone)]
#[repr(packed)]
pub struct BPTreeMetaValues {
    pub version : u16,
    pub capacity : u16,
    pub next : Pubkey,

    pub tree : BPTreeIdentifier, // TreeReference,
    
    pub assoc : Pubkey,
    pub flags : u16,
    // pub root : Pubkey,
    // pub parent : Pubkey,
    // pub prev : Pubkey,
    // pub next : Pubkey,
}

impl BPTreeMetaValues {

    // #[inline(always)]
    // pub fn is_leaf(&self) -> bool { if self.flags & (Flags::Leaf as u8) != 0 { true } else { false } }

    pub fn try_init<'info,'refs,K,V>(&mut self, capacity: usize, tree : &BPTree<'info,'refs,K,V>)
    -> Result<()>
    where
     K: Eq + Ord + PartialOrd + Copy + std::fmt::Debug, V: Copy + std::fmt::Debug + Default

    {
        if self.version != 0u16 {
            //log_trace!("BPTreeMetaValues:self.version: {:?}", self.version);
            log_trace!("META SELF: {:#?}", self);
            return Err(ErrorCode::ContainerMetaVersionMismatch.into())
        }

        self.version = 1u16;
        self.capacity = capacity as u16;
        // self.owner = *owner;
        self.tree = BPTreeIdentifier {
            pubkey: *tree.pubkey(),
            segment : tree.segment.idx as u16,
        };

        Ok(())
    }

    // pub fn new(
    //     capacity : usize,
    //     next : Pubkey,
    //     root: Pubkey,
    //     // parent: Pubkey,
    //     // prev: Pubkey,
    //     // next: Pubkey,
    // ) -> BPTreeMetaValues<V> {
    //     assert!(capacity < 0xffff);
    //     BPTreeMetaValues {
    //         version : 1u16,
    //         capacity : capacity as u16,
    //         next
    //     }
    // }
}



const BPTREE_CONTAINER_INDEX: u32 = Containers::BPTreeIndex as u32;
const BPTREE_CONTAINER_VALUES: u32 = Containers::BPTreeValues as u32;

pub fn try_get_container_type(account: &AccountInfo) -> Result<Containers> {
    let ctype = crate::container::try_get_container_type(account)?;
    // log_trace!("[b+tree] detecting container type: {:#x}", ctype);
    match ctype {
        BPTREE_CONTAINER_INDEX => Ok(Containers::BPTreeIndex),
        BPTREE_CONTAINER_VALUES => Ok(Containers::BPTreeValues),
        _ => Err(ErrorCode::BPTreeUnknownContainerType.into())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BPTreeIndexCell<K> 
where 
    K: Eq + PartialOrd + Copy,
{
    pub key : K,
    pub target : Pubkey,
}

impl<K> BPTreeIndexCell<K> where K: Eq + PartialOrd + Copy {
    pub fn new(key: &K, target : &Pubkey) -> BPTreeIndexCell<K> {
        BPTreeIndexCell { key : *key, target : target.clone() }
    }
}

impl<K> std::cmp::PartialOrd for BPTreeIndexCell<K> where K: Eq + PartialOrd + Copy {
    fn partial_cmp(&self, other: &BPTreeIndexCell<K>) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K> std::cmp::Ord for BPTreeIndexCell<K> where K: Eq + Ord + Copy {
    fn cmp(&self, other: &BPTreeIndexCell<K>) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K> std::cmp::PartialEq for BPTreeIndexCell<K> where K: Eq + PartialOrd + Copy {
    fn eq(&self, other: &BPTreeIndexCell<K>) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K> std::cmp::Eq for BPTreeIndexCell<K> where K: Eq + PartialOrd + Copy { }

// ~~~

// ~~~

#[derive(Debug, Clone, Copy)]
pub struct BPTreeValueCell<K,V> 
where 
    K: Eq + PartialOrd + Copy,
    V: Copy
{
    pub key : K,
    pub value : V,
}

impl<K,V> BPTreeValueCell<K,V> where K: Eq + PartialOrd + Copy, V: Copy {
    pub fn new(key: &K, value: &V) -> BPTreeValueCell<K,V> {
        BPTreeValueCell { key : *key, value : *value }
    }
}


impl<K,V> std::cmp::PartialOrd for BPTreeValueCell<K,V> where K: Eq + PartialOrd + Copy, V: Copy {
    fn partial_cmp(&self, other: &BPTreeValueCell<K,V>) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K,V> std::cmp::Ord for BPTreeValueCell<K,V> where K: Eq + Ord + Copy, V: Copy  {
    fn cmp(&self, other: &BPTreeValueCell<K,V>) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K,V> std::cmp::PartialEq for BPTreeValueCell<K,V> where K: Eq + PartialOrd + Copy, V: Copy  {
    fn eq(&self, other: &BPTreeValueCell<K,V>) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K,V> std::cmp::Eq for BPTreeValueCell<K,V> where K: Eq + PartialOrd + Copy, V: Copy  { }

// ~~~

// #[derive(Debug)]
#[container(Containers::BPTreeIndex, u32)]
pub struct BPTreeIndex<'info,'refs,K> 
where
    K : Eq + Ord + PartialOrd + Copy
{
    pub meta : RefCell<&'info mut BPTreeMetaIndex>,
    pub store : SegmentStore<'info,'refs>,

    // v : u32,
    // #[segment(reserve = 1024)]
    #[segment(flex, reserve(Array::<BPTreeIndexCell<K>>::calculate_data_len(1)))]
    pub data : Array<'info,'refs, BPTreeIndexCell::<K>>,
}

impl<'info,'refs, K> BPTreeIndex<'info,'refs,K> 
where
    K: Eq + Ord + PartialOrd + Copy
{
    pub fn try_create_with_args<'pid,'instr>(
        capacity : usize,
        records : usize,
        ctx: &Context<'info,'refs,'pid,'instr>,
        allocation_args: &AccountAllocationArgs<'info,'refs>
    ) -> Result<BPTreeIndex<'info,'refs,K>> {

        let initial_data_len = BPTreeIndex::<K>::initial_data_len_with_records(records);
        let new_account = ctx.create_pda(initial_data_len,allocation_args)?;
        let index = BPTreeIndex::try_create(&new_account)?;

        {
            let mut index_meta = index.meta.borrow_mut();
            index_meta.try_init(capacity, ctx.program_id)?;
        }

        Ok(index)
    }

    pub fn try_create_with_records<'pid,'instr>(ctx: &Context<'info,'refs,'pid,'instr>, records : usize, allocation_args : &AccountAllocationArgs<'info,'refs>) -> Result<BPTreeIndex<'info,'refs,K>> {
        let initial_data_len = BPTreeIndex::<K>::initial_data_len_with_records(records);
        // let allocation_args = AccountAllocationArgs::
        let new_account = ctx.create_pda(initial_data_len,allocation_args)?;
        Ok(BPTreeIndex::try_create(&new_account)?)
    }

    pub fn initial_data_len_with_records(records : usize) -> usize {
        BPTreeIndex::<K>::initial_data_len() + records * std::mem::size_of::<BPTreeIndex<K>>()
    }

    // pub fn binary_search(&self, n : &BPTreeIndexCell<K>) -> std::result::Result<usize,usize> {
    //     self.data.as_slice().binary_search(n)
    // }

    pub fn lookup(&self, cell : &BPTreeIndexCell<K>) 
    -> Result<Pubkey> 
    where K : 'info
    {
    // pub fn lookup(&self, key : &K) -> Result<Pubkey> {
        // let cell = BPTreeIndexCell::<K>::new(key,&Pubkey::default());

        let data_len = self.data.len();
        if data_len == 0 {
            return Err(error_code!(ErrorCode::BPTreeIndexIsEmpty))
        }
        let idx = match self.data.as_slice().binary_search(&cell) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        if idx == data_len {
            let meta = self.meta.borrow();
            Ok(meta.last)
        } else {
            Ok(self.data.get_at(idx).target)
        }
    }



    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.meta.borrow().capacity as usize
    }


    pub unsafe fn try_insert(&self, key: K, target: &Pubkey) -> Result<()> {
        let cell = self.data.try_allocate(false)?;
        cell.key = key;
        cell.target = *target;
        Ok(())
    }

    pub unsafe fn try_insert_at(&self, idx: usize, src: &BPTreeIndexCell<K>) -> Result<()> {
        let cell = self.data.try_allocate_at(idx, false)?;
        *cell = *src;//.clone();
        // cell.key = key;
        // cell.target = *target;
        Ok(())
    }
    // pub fn volatile_try_insert_at(&self, idx: usize, key: K, target : &Pubkey) -> Result<()> {
    //     let cell = self.data.volatile_try_insert_at(idx, false)?;
    //     cell.key = key;
    //     cell.target = *target;
    //     Ok(())
    // }

}



impl<'info,'refs,K> TryInto<BPTreeIndex<'info,'refs,K>> for &'refs AccountInfo<'info> 
where
    K : Eq + Ord + PartialOrd + Copy
{
    type Error = crate::error::Error;

    fn try_into(self) -> std::result::Result<BPTreeIndex<'info,'refs,K>,Self::Error> {
        Ok(BPTreeIndex::try_load(self)?)
    }
}


// ~~~

// #[derive(Debug)]?
#[container(Containers::BPTreeValues, u32)]
pub struct BPTreeValues<'info,'refs,K,V> 
where
    K : Eq + Ord + PartialOrd + Copy,
    V: Copy 
{
    pub meta : RefCell<&'info mut BPTreeMetaValues>,
    pub store : SegmentStore<'info,'refs>,

    // v : u32,
    // #[segment(reserve = 1024)]
    // TODO: SWITCH DEFAULT RECORD COUNT TO 0
    #[segment(flex, reserve(Array::<BPTreeValueCell<K,V>>::calculate_data_len(1)))]
    pub data : Array<'info,'refs, BPTreeValueCell::<K,V>>,
}


impl<'info,'refs, K,V> BPTreeValues<'info,'refs,K,V> 
where
    K: Eq + Ord + PartialOrd + Copy + std::fmt::Debug, V: Copy + std::fmt::Debug + Default 
{

    // #[inline]
    pub fn try_create_with_args<'pid,'instr>( 
        capacity : usize,
        records : usize,
        ctx: &Context<'info,'refs,'pid,'instr>,
        tree: &BPTree<'info,'refs,K,V>,
        allocation_args: &AccountAllocationArgs<'info,'refs>
    ) -> Result<BPTreeValues<'info,'refs,K,V>> {

        let initial_data_len = BPTreeValues::<K,V>::initial_data_len_with_records(records);
        let new_account = ctx.create_pda(initial_data_len,allocation_args)?;
        let values = BPTreeValues::try_create(&new_account)?;

        {
            let mut values_meta = values.meta.borrow_mut();
            values_meta.try_init(capacity, tree)?;
        }

        Ok(values)
    }

    pub fn try_create_with_records<'pid,'instr>(ctx: &Context<'info,'refs,'pid,'instr>, records : usize, allocation_args: &AccountAllocationArgs<'info,'refs>) -> Result<BPTreeValues<'info,'refs,K,V>> {
        let initial_data_len = BPTreeValues::<K,V>::initial_data_len_with_records(records);
        let new_account = ctx.create_pda(initial_data_len,allocation_args)?;
        Ok(BPTreeValues::try_create(&new_account)?)
    }

    pub fn initial_data_len_with_records(records : usize) -> usize {
        BPTreeValues::<K,V>::initial_data_len() + records * std::mem::size_of::<BPTreeValueCell<K,V>>()
    }

    // pub fn initial_data_segment_len_with_records(records : usize) -> usize {
    //     BPTreeValues::<K,V>::initial_data_len() + records * std::mem::size_of::<BPTreeValueCell<K,V>>()
    // }

    pub fn binary_search(&self, n : &BPTreeValueCell<K,V>) -> std::result::Result<usize,usize> where K: 'info, V : 'info {
        self.data.as_slice().binary_search(n)
    }

    

    pub fn binary_search_idx_by_key(&self, key : &K) -> usize where K : 'info, V : 'info {
        let cell = &BPTreeValueCell::<K,V>::new(key,&V::default());
        let idx = match self.data.as_slice().binary_search(cell) {
        // let idx = match values.binary_search(&cell) {
            Ok(idx) => {
                idx
            },
            Err(idx) => {
                idx
            }
        };
        idx
    }



    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.meta.borrow().capacity as usize
    }

    pub unsafe fn try_insert(&self, key: &K, value: &V) -> Result<()> {
        let cell = self.data.try_allocate(false)?;
        cell.key = *key;
        cell.value = *value;
        Ok(())
    }

    pub unsafe fn try_insert_at(&self, idx: usize, key: &K, value: &V) -> Result<()> {
        let cell = self.data.try_allocate_at(idx, false)?;
        cell.key = *key;
        cell.value = *value;
        Ok(())
    }

}


impl<'info,'refs,K,V> TryInto<BPTreeValues<'info,'refs,K,V>> for &'refs AccountInfo<'info> 
where K : Eq + Ord + PartialOrd + Copy, V: Copy 
{
    type Error = crate::error::Error;

    fn try_into(self) -> std::result::Result<BPTreeValues<'info,'refs,K,V>,Self::Error> {
        Ok(BPTreeValues::try_load(self)?)
    }
}


// ~~~

// u32_try_from!(
//     pub enum TreeFlags {
        
//     }
// )

#[derive(Debug)]
pub struct BPTreeMeta {

    // pub flags : u32,
    pub root : Pubkey,
    pub count : u64,
    // pub index_capacity : u32,
    // pub value_capacity : u32, 

}

#[derive(Debug, Clone)]
pub struct BPTree<'info,'refs,K,V>
where
    K : std::fmt::Debug + Ord + Copy + 'refs,
    V: std::fmt::Debug + Default + Copy + 'refs 
{
    pub meta : Rc<RefCell<&'info mut BPTreeMeta>>,
    pub segment : Rc<Segment<'info,'refs>>,
    _k : PhantomData<K>,
    _v : PhantomData<V>,
    // pub index_capacity : usize,
    // pub value_capacity : usize,
}


impl<'info,'refs,K,V> BPTree<'info,'refs,K,V> 
where
    K : std::fmt::Debug + Ord + Copy,// + 'refs,
    V : std::fmt::Debug + Default + Copy,// + 'refs 
{

    pub fn data_len_min() -> usize { std::mem::size_of::<BPTreeMeta>() }

    pub fn pubkey(&self) -> &Pubkey {
        self.segment.store.account.key
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<BPTree<'info,'refs,K,V>> {
        let meta = Rc::new(RefCell::new(segment.try_as_struct_mut::<BPTreeMeta>()?)); 
        Ok(BPTree {
            meta,
            segment,
            _k : PhantomData::<K>::default(),
            _v : PhantomData::<V>::default(),
            // index_capacity : 1024,
            // value_capacity: 1024,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<BPTree<'info,'refs,K,V>> {
        let meta = Rc::new(RefCell::new(segment.try_as_struct_mut::<BPTreeMeta>()?)); 
        Ok(BPTree {
            meta,
            segment,
            _k : PhantomData::<K>::default(),
            _v : PhantomData::<V>::default(),
        })
    }

    // pub fn get_tree_identifier(&self) -> BPTreeIdentifier {
    //     BPTreeIdentifier {
    //         pubkey : *self.pubkey(),
    //     }
    // }

    pub fn root(&self) -> Result<Pubkey> {
        Ok(self.meta.try_borrow_mut()?.root.clone())
    }

    fn try_split_index<'pid,'instr,'idx>(
        &self, 
        ctx : &Rc<Box<Context<'info,'refs,'pid,'instr>>>,
        index : &'idx BPTreeIndex<'info,'refs,K>,
        // ???????
        _parent : Option<BPTreeIndex<'info,'refs,K>>,
        allocation_args: &AccountAllocationArgs<'info,'refs>
    ) -> Result<(&'idx BPTreeIndex<'info,'refs,K>,BPTreeIndex<'info,'refs,K>)>
    where K : 'info, V : 'info
    // where K : Ord + Copy 
    {

        log_trace!("{}",style("   INDEX SPLIT!   ").white().on_magenta());

        let index_len = index.len();
        let left_len = index_len / 2;
        let right_len = index_len - left_len;
        let initial_data_len = BPTreeIndex::<K>::initial_data_len_with_records(right_len);
        let new_account = ctx.create_pda(initial_data_len,allocation_args)?;
        let left = index;
        // let meta_init = BPTreeMetaIndex::new(left_len)?;
        let right = BPTreeIndex::try_create(&new_account)?;
        
        // TODO: check - this should not invoke realloc!
        log_trace!("{}",style("AAA").white().on_cyan());
        unsafe { right.data.try_resize_for_items(right_len,false)?; }
        let src_slice = &left.data.as_slice()[left_len..];
        right.data.as_slice_mut().copy_from_slice(src_slice);
        // right.
        // TODO: reduce account size
        log_trace!("{}",style("BBB").white().on_cyan());
        unsafe { left.data.try_resize_for_items(left_len, false)?; }
        
        {
            let mut left_meta = left.meta.borrow_mut();
            let right_cell = right.data.try_get_at(0)?;

            let mut right_meta = right.meta.borrow_mut();
            let capacity = BPTREE_MAX_INDEX_ITEMS_CAPACITY; //(BPTREE_MAX_VALUES_ACCOUNT_SIZE_BYTES - initial_data_len) / std::mem::size_of::<BPTreeIndexCell<K>>();
            right_meta.try_init(capacity,self.pubkey())?;
            right_meta.last = left_meta.last;
            left_meta.last = right_cell.target;

            // right_meta.next = left_meta.next;
            // left_meta.next = *right.pubkey();
        }

        // let split_cell = &src_slice[0];

        Ok((left,right))
    }

    fn try_split_values_with_insert<'pid,'instr>(
        &self,
        ctx : &Rc<Box<Context<'info,'refs,'pid,'instr>>>,
        values : BPTreeValues<'info,'refs,K,V>,
        insert_cell : BPTreeValueCell<K,V>,
        allocation_args : &AccountAllocationArgs<'info,'refs>,
    ) -> Result<(BPTreeValues<'info,'refs,K,V>,BPTreeValues<'info,'refs,K,V>)> 
    where K : 'info, V : 'info
    // where
    //     K : Ord + Copy,
    //     V : Copy 
    {
        log_trace!("{}",style("   VALUES SPLIT!   ").white().on_magenta());


        let values_len = values.len();
        let left_len = values_len / 2;
        let right_len = values_len - left_len;
        // let mut right_alloc = right_len;

        let split_cell = &values.data[left_len];
        let is_left = insert_cell < *split_cell;
        let right_alloc_records = if is_left { right_len } else { right_len+1 };
        let initial_right_data_len = BPTreeValues::<K,V>::initial_data_len_with_records(right_alloc_records);
        log_trace!("{}:{}",style("left.len():").red(),values.len());
        log_trace!("{}:{}",style("initial_right_data_len:").red(),initial_right_data_len);
        let left = values;

        let new_account = ctx.create_pda(initial_right_data_len,allocation_args)?;

        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // ? FIXME
        let mut _right_layout = BPTreeValues::<K,V>::layout();
        let _right_data_segment = Array::<BPTreeValueCell::<K,V>>::calculate_data_len(right_alloc_records);
        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // * #################################################################
        // ^ #################################################################
        // right_layout.set(1, )

        let right = BPTreeValues::try_create(&new_account)?;
        log_trace!("{}",style("   CCCCCCCCCCCCCC   ").white().on_blue());
        log_trace!("{} {}",style("right_alloc_records:").white().on_blue(),right_alloc_records);
        
        unsafe { right.data.try_resize_for_items(right_alloc_records,false)?; }
        let src_slice = &left.data.as_slice()[left_len..];
        log_trace!("{}:{}",style("src slice len:").red(),src_slice.len());

        if is_left {

                right.data.as_slice_mut().copy_from_slice(src_slice);

                left.data.get_meta().records = left_len as u32;
                let idx = match left.data.as_slice().binary_search(&insert_cell) {
                    Ok(_idx) => { return Err(ErrorCode::BPTreeIndexCellCollision.into()) },
                    Err(idx) => { idx }
                };

                // TODO: reduce account size
                unsafe { 
                    let cell = left.data.try_allocate_at(idx,false)?;
                    *cell = insert_cell;
                }
                // TODO: this should not be needed as volatile_try_insert_at() ^ should handle
                log_trace!("{}",style("DDD").white().on_cyan());
                
                unsafe {
                    left.data.try_resize_for_items(left_len+1, false)?;
                }

        } else {

                let idx = match src_slice.binary_search(&insert_cell) {
                    Ok(_idx) => { return Err(ErrorCode::BPTreeIndexCellCollision.into()) },
                    Err(idx) => { idx }
                };

                let dest_slice = right.data.as_slice_mut();
                log_trace!("{}:{}",style("SEARCH INDEX:").blue(),idx);
                log_trace!("{}:{}",style("dest_slice len:").green(),dest_slice.len());
                dest_slice[0..idx].copy_from_slice(&src_slice[0..idx]);
                dest_slice[idx] = insert_cell;
                dest_slice[idx+1..].copy_from_slice(&src_slice[idx..]);

                // TODO: transfer out lamports
                log_trace!("{}",style("EEE").white().on_cyan());
                unsafe {
                    left.data.try_resize_for_items(left_len, false)?; //init_len(len_left);
                }
        }
        
        // TODO: check - this should not invoke realloc!
        // right.data.try_resize_for_items(right_len,false)?;
        
        {
            let mut left_meta = left.meta.borrow_mut();
            let mut right_meta = right.meta.borrow_mut();
            let capacity = BPTREE_MAX_VALUE_ITEMS_CAPACITY; //(BPTREE_MAX_VALUES_ACCOUNT_SIZE_BYTES - initial_right_data_len) / std::mem::size_of::<BPTreeValueCell<K,V>>();
            right_meta.try_init(capacity,self)?;//.pubkey(),&self.segment)?;
            // ^ TODO: ???
            right_meta.next = left_meta.next;
            left_meta.next = *right.pubkey();
        }

        // ^ TODO: sync rent should transfer funds to 
        let rent_collector = RentCollector::Program;
        left.sync_rent(ctx,&rent_collector)?;
        log_trace!("{}",style("   VALUES SPLIT DONE   ").white().on_magenta());

        // let split_cell = &src_slice[0];
        Ok((left,right))
    }

    // pub fn 

    pub fn insert<'pid,'instr>(
        &self,
        path: &'refs [AccountInfo<'info>],
        ctx : &Rc<Box<Context<'info,'refs,'pid,'instr>>>,
        key: &K,
        value : &V,
        allocation_args: &AccountAllocationArgs<'info,'refs>,
    ) -> Result<()>
    where K : 'info, V : 'info
    // where
    //     K : Ord + Copy + 'refs,
    //     V: Copy + 'refs 
    {

        let mut meta = self.meta.borrow_mut();//.clone();

        if meta.root == Pubkey::default() {
            // * empty tree - create root
            if !path.is_empty() {
                // TODO: review asynchronous submission logic
                return Err(error!("path must be empty!"));
            }
            assert_eq!(path.len(), 0);

            let values = BPTreeValues::try_create_with_args(
                BPTREE_MAX_VALUE_ITEMS_CAPACITY,
                1,
                ctx,
                self,
                allocation_args
            )?;
            // log_trace!("[b+tree] creating values container: {}",values.pubkey().to_string());
            // log_trace!("~~ values segment store meta before: {:#?}", values.store.get_meta());
            // log_trace!("executing: {}", style("values.volatile_try_insert(key,value)?;").red());
            unsafe { values.try_insert(key,value)?; }
            // log_trace!("~~ values segment store meta after init: {:#?}", values.store.get_meta());
            meta.root = *values.pubkey(); //*new_account.key;
            // log_trace!("values segment store meta after root: {:#?}", values.store.get_meta());
            // meta.count = 1;
        } else {
            // * iterate path

            // TODO: allow path
            if path.len() == 0 {
                return Err(ErrorCode::BPTreePathEmpty.into());
            }

            // let mut something_needs_split : bool = false; 
            let mut index_needs_split : Option<usize> = None;
            let mut levels = Vec::new();
            let mut next : Option<Pubkey> = None;
            // let levels = path.len()-1;
            for level in 0..(path.len()-1) {
                // TODO - COLLECT POSITIONS AT EACH LEVEL...
                let account = &path[level];

                if let Some(next) = next {
                    if next != *account.key {
                        return Err(error_code!(ErrorCode::BPTreePathError));
                    }
                }    

                let index = BPTreeIndex::try_load(account)?;
    
                let index_len = index.len();
                let index_capacity = index.capacity();
                // if index_len > index_capacity / BPTREE_ASSOC_THRESHOLD[level] {
// * TODO - defer to assoc
// * TODO - defer to assoc
// * TODO - defer to assoc
// * TODO - defer to assoc
                // }

                if index_needs_split.is_none() {
                    let needs_split = index_len >= index_capacity;
                    if needs_split { index_needs_split = Some(level); }
                }
                
                let search_kv = BPTreeIndexCell::<K>::new(key,&Pubkey::default());

                let target = index.lookup(&search_kv)?;
                next = Some(target);

                // TODO: check if Rc+RefCell are cheaper than direct in-vector storage?
                // levels.push(Rc::new(RefCell::new(index)));
                levels.push(index);
                // levels.push((tree_index, needs_split));
            }

            // TODO - CHECK LEN, SPLIT IF NEEDED, MOVE SPLIT TO TOP...
            let last_account = &path[path.len()-1];

            if let Some(next) = next {
                if next != *last_account.key {
                    return Err(error_code!(ErrorCode::BPTreePathError));
                }
            }

            let values = BPTreeValues::try_load(&last_account)?;
            let values_len = values.len();
            let search_kv = BPTreeValueCell::<K,V>::new(key,value);
            if values_len <= values.capacity() {
                // * simple insert, then split if needed
                match values.data.as_slice().binary_search(&search_kv) {
                    Ok(_idx) => {
                        // collision - key already exists
                        return Err(program_error_code!(ErrorCode::BPTreeCollision)
                            .with_message(&format!("BPTreeCollision {:?}", search_kv)));
                    },
                    Err(idx) => {
                        unsafe { values.try_insert_at(idx,key,value)?; }
                        // meta.count += 1;
                        let rent_collector = RentCollector::Program;
                        values.sync_rent(ctx,&rent_collector)?;
                

                        // * perform a deferred split if needed
                        if let Some(level) = index_needs_split {

                            let index = &levels[level];
                            let (left,right) = self.try_split_index(ctx,index,None,allocation_args)?;
                            let right_cell = right.data.try_get_mut_at(0)?.clone();

                            if level == 0 {
                                // * split and create new root
                                log_trace!("{}",style("[btree] creating new root index").red());

                                let index = BPTreeIndex::try_create_with_args(
                                    BPTREE_MAX_INDEX_ITEMS_CAPACITY,
                                    1,
                                    ctx,
                                    allocation_args
                                )?;
                    
                                let mut index_meta = index.meta.borrow_mut();
                                index_meta.last = *right.pubkey();
                                unsafe { index.try_insert(right_cell.key,left.pubkey())?; }
                                meta.root = *index.pubkey();
            
                            } else {
                                // * insert in parent
                                let parent = &levels[level-1];
                                unsafe { parent.try_insert(right_cell.key,left.pubkey())?; }
                            }

                        }

                    }
                }
            } else {
                // ^ TODO: SPLIT UPWARDS...
                log_trace!("{}",style("   YYYYYYYY   ").white().on_magenta());


                let insert_cell = BPTreeValueCell::<K,V>::new(key,value);
                let (left,right) = self.try_split_values_with_insert(ctx, values, insert_cell, allocation_args)?;
                // ? FIXME - not used
                let _left_cell = left.data.try_get_at(0)?;
                let right_cell = right.data.try_get_at(0)?;

                if path.len() == 1 {

                    // * create first root index 

                    // log_trace!("{}",style("[btree] creating first root index").green());
                    log_trace!("{}",style("[btree] creating first root index").red());

                    // create_index_root

                    let index = BPTreeIndex::try_create_with_args(
                        BPTREE_MAX_INDEX_ITEMS_CAPACITY,
                        1,
                        ctx,
                        allocation_args
                    )?;
        
                    let mut index_meta = index.meta.borrow_mut();
                    index_meta.last = *right.pubkey();
                    unsafe { index.try_insert(right_cell.key,left.pubkey())?; }
                    meta.root = *index.pubkey();

                } else {
        log_trace!("{}",style("   XXXXXXX   ").white().on_magenta());

                    // ^ TODO: insert into parent
                    log_trace!("{}",style("[btree] insert into parent").green());
                    // let (_,_,tree_index) = levels.last().unwrap();
                    let index = levels.last().unwrap();

                    // ^ TODO: TREE INDEX CELL
                    let cell = BPTreeIndexCell::new(&right_cell.key, left.pubkey());

                    match index.data.as_slice().binary_search(&cell) {
                        Ok(_idx) => { /* wtf!() */ 
                            log_trace!("rejecting insert - matching key found in the index");
                            return Err(program_error_code!(ErrorCode::BPTreeCollision)
                                .with_message(&format!("BPTreeIndex -> BPTreeCollision {:?}", right_cell)));
                            // assert!(false); 
                        },
                        Err(idx) => {
                            // tree_index.data.volatile_try_insert_at(idx)

                            if idx == index.len() {

                                let mut meta = index.meta.borrow_mut();
                                // if meta.last == 
                                meta.last = *right.pubkey();
                            } 

                            unsafe { index.try_insert_at(idx, &cell)?; }
                            let rent_collector = RentCollector::Program;
                            index.sync_rent(ctx,&rent_collector)?;
    
                            // else {
                            // }
                        }
                    }
                    // tree_index.volatile_try_insert_at

                }
            }

        }

        meta.count += 1;

        Ok(())
    }

    // fn split_values(values : BPTreeValues) -> Result<()> {

    //     Ok(())
    // }

    // TODO
    pub fn merge_indexes<'pid,'instr>(
        ctx : &Rc<Box<Context<'info,'refs,'pid,'instr>>>,
        left: BPTreeIndex<'info,'refs,K>,
        right: BPTreeIndex<'info,'refs,K>
    )
    -> Result<BPTreeIndex<'info,'refs,K>>
    where K : 'info, V : 'info
    // where
    //     K : Ord + Copy
    {
        let left_len = left.len();
        let right_len = right.len();
        let total_len = right_len + left_len;
        log_trace!("{}",style("FFF").white().on_cyan());
        unsafe { left.data.try_resize_for_items(total_len,false)?; }
        let dest = left.data.as_slice_mut();
        dest[left_len..].copy_from_slice(right.data.as_slice());

        right.purge(ctx,&RentCollector::Program)?;

        Ok(left)
    }

    // TODO
    pub fn merge_values<'pid,'instr>(
        ctx : &Rc<Box<Context<'info,'refs,'pid,'instr>>>,
        left: BPTreeValues<'info,'refs,K,V>,
        right: BPTreeValues<'info,'refs,K,V>
    )
    -> Result<BPTreeValues<'info,'refs,K,V>>
    where K : 'info, V : 'info
    // where
    //     K : Ord + Copy,
    //     V : Default + Copy
    {
        let left_len = left.len();
        let right_len = right.len();
        let total_len = right_len + left_len;
        log_trace!("{}",style("GGG").white().on_cyan());
        unsafe { left.data.try_resize_for_items(total_len,false)?; }
        let dest = left.data.as_slice_mut();
        dest[left_len..].copy_from_slice(right.data.as_slice());

        right.purge(ctx,&RentCollector::Program)?;

        Ok(left)
    }


    pub fn remove<'k>(
        &self,
        path: &[AccountInfo<'info>],
        key: &'k K
    ) -> Result<()> 
    where K : 'info, V : 'info
    // where
    //         K : Ord + Copy,
    //         V : Default + Copy 
    {
        let meta = self.meta.borrow_mut();//.clone();

        if meta.count == 0 {
            return Err(ErrorCode::BPTreeNoSuchRecord.into());
        } else {

            // ^ SCAN PATH
            // ^ TAKE LAST ENTRY
            // ^ REMOVE FROM LAST ENTRY
            // ^ SEE IF BELOW THRESHOLD

            let last_account = &path[path.len()-1];
            let values = BPTreeValues::try_load(&last_account)?;
            // let values_len = values.len();
            let search_kv = BPTreeValueCell::<K,V>::new(key,&V::default());

            let idx = match values.data.as_slice().binary_search(&search_kv) {
                Err(_idx) => {
                    return Err(error_code!(ErrorCode::BPTreeNoSuchRecord));
                },
                Ok(idx) => idx,
            };

            let threshold = 123;

            if values.data.len() < threshold {

                unsafe { values.data.try_remove_at(idx,false)?; }

                // ^ TODO: MERGE

                // ^ GO UPWARDS
            } else {
                unsafe { values.data.try_remove_at(idx,true)?; }

            }


        }

        Ok(())
    }


}


#[cfg(not(target_arch = "bpf"))]
impl<'info,'refs,K,V> BPTree<'info,'refs,K,V> 
where
    K : std::fmt::Debug + std::fmt::Display + Ord + Copy + Default,// + 'refs,
    V: std::fmt::Debug + std::fmt::Display + Default + Copy,// + 'refs 
{

    #[inline]
    pub async fn lookup(&self, k : &K) -> Result<client::BPTreeLookupContext<V>> {
        client::lookup(self,k).await
    }


    pub async fn get_entries_with_iterator(&self, iterator : &client::BPTreeIterator, items : usize)
    -> Result<Vec<(K,V)>> 
    {
        client::get_entries_with_iterator::<K,V>(iterator,items).await
    }
}



// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

/* 
pub struct SliceMeta {

    // pub root : Pubkey,
    pub len : u32, 
}

pub struct Slice<'info,'refs,T> {
    pub meta : Rc<RefCell<&'info mut SliceMeta>>,
    pub segment : Rc<Segment<'info,'refs>>,
    phantom : PhantomData<T>,
}

impl<'info,'refs,T> Slice<'info,'refs,T> {

    pub fn data_len_min() -> usize { std::mem::size_of::<SliceMeta>() }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Slice<'info,'refs,T>> {
        let meta = Rc::new(segment.try_as_struct_mut_ref::<SliceMeta>()?); 
        Ok(Slice {
            meta,
            segment,
            phantom : PhantomData
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Slice<'info,'refs,T>> {
        let meta = Rc::new(segment.try_as_struct_mut_ref::<SliceMeta>()?); 
        Ok(Slice {
            meta,
            segment,
            phantom : PhantomData
        })
    }

    // pub fn get_tree_chain(&self) -> Vec<BPTreeNode {

    // }
}
*/

// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

// ^ TODO:  CLIENT TREE SEARCH USING ASYNC
// 

#[repr(packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct InsertionIndex {
    ts : u64,
    seq : u64,
}

impl InsertionIndex {
    pub fn new(ts: u64, seq: u64) -> InsertionIndex {
        InsertionIndex { ts, seq }
    }
}

impl Default for InsertionIndex {
    fn default() -> InsertionIndex {
        InsertionIndex {
            ts : 0, seq : 0
        }
    }
}




impl std::fmt::Display for InsertionIndex {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ts = self.ts;
        let seq = self.seq;
        write!(f, "InsertionIndex: {}-{}", ts, seq)
    }
}



























#[container(1234,u16)]
pub struct MyStruct<'info,'refs> {
    pub btree : BPTree<'info,'refs,u32,u32>
}

#[cfg(not(target_arch = "bpf"))]
pub mod client {

    // use crate::{accounts::AccountInfoTemplate, client::prelude::{Instruction, InstructionBuilder}, cache::client::Cache};

    use super::*;
    // use crate::transport::*;    
    // use workflow_allocator_macros::{declare_println};
    // use wasm_bindgen::prelude::*;
    // use js_sys::*;
    // use wasm_bindgen_futures::{JsFuture,future_to_promise};
    use solana_program::pubkey::Pubkey;
    // use crate::context::Context;
    use workflow_log::log_trace;
    // #[wasm_bindgen]
    

    pub struct BPTreeContext<'info,'refs,'ctx,K,V>  
    where 
        K : Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display,
        V : Default + Clone + Copy + std::fmt::Debug + std::fmt::Display
    {
        pub root : &'ctx BPTree<'info,'refs,K,V>,
        pub path : &'ctx [&'refs AccountInfo<'info>],
    }



    // #[derive(Debug, Clone)]
    // pub struct InsertionTarget {
    //     pub path : Vec<Pubkey>
    // }

    pub struct BPTreeLookupContext<V> {
        // tree : BPTree<'info,'refs>,
        pub path : Vec<Pubkey>,
        // pub key : K,
        pub value : Option<V>,
        
        pub leaf_index : Option<usize>
    }

    impl<V> BPTreeLookupContext<V> {
        // pub fn new(path : &[Pubkey], value : Option<V>) -> BPTreeLookupContext<V> {
        //     BPTreeLookupContext {
        //         // tree : tree.clone(),
        //         path : path.to_vec(),
        //         value
        //     }
        // }

        pub fn leaf_pubkey(&self) -> Result<Pubkey> {
            Ok(self.path[self.path.len()-1])
        }
        // pub fn values<K>(&self) -> Result<BPTreeValues<'info,'refs,K,V>> {

        // }

    }

    // impl<'info,'refs,V> TryInto<BPTreeLookupContext<'info,'refs,V>> for BPTree<'info,'refs> {
    //     type Error = crate::error::Error;

    //     fn try_into(self) -> std::result::Result<BPTreeLookupContext<'info,'refs,V>,Self::Error> {
    //         Ok(BPTreeLookupContext::new(&self))
    //     }
    // }
  
    // pub async fn lookup<'info,'refs,K,V>(tree : &BPTree<'info,'refs,K,V>, k : &K) -> std::result::Result<BPTreeLookupContext<V>,String> 
    pub async fn lookup<'info,'refs,K,V>(tree : &BPTree<'info,'refs,K,V>, k : &K) -> Result<BPTreeLookupContext<V>> 
    // pub async fn lookup<'info,'refs,K,V>(root : &Pubkey, k : &K) -> std::result::Result<BPTreeLookupContext<V>,String> 
    where 
        K : Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display,
        V : Default + Clone + Copy + std::fmt::Debug + std::fmt::Display
    {

        let transport = crate::transport::Transport::global()?;
        let mut path: Vec<Pubkey> = Vec::new();
        let mut value : Option<V> = None;
        let mut leaf_index : Option<usize> = None;
        let mut _values_pubkey : Option<Pubkey> = None;
        let mut key = tree.root()?;
        // let mut key = root.clone(); //tree.root()?;
        if key == Pubkey::default() {
            return Ok(BPTreeLookupContext {
                // tree : tree.clone(),
                path,
                value,
                leaf_index,
                // key : k.clone()
            });
        }

        let search_index = BPTreeIndexCell::<K>::new(&k,&Pubkey::default());
        let search_values = BPTreeValueCell::<K,V>::new(&k,&V::default());
        
        let mut counter = 0;
        
        loop {

            counter += 1;
            if counter == 10 {
                log_trace!("ABORTING");
                return Err(error_code!(ErrorCode::BPTreeCyclicAbort));
                // break;
            }

            let reference = transport.clone().lookup(&key).await?
                .ok_or(error_code!(ErrorCode::BPTreeIndexDereference))?;
            
            // { //.expect(&format!("missing account for key {}",key.to_string()));
            //     Some(reference) => reference,
            //     None => {
            //         return Err(error_code!(ErrorCode::BPTreeIndexDereference));
            //     }
            // };
            // let mut account_data = account_data_ref_cell.borrow_mut();
            let mut account_data = reference.account_data.lock()?;
            let account_key = key.clone();
            let account_info = (&account_key, &mut *account_data).into_account_info();

            match try_get_container_type(&account_info)? {
                Containers::BPTreeIndex => { 
                    // log_trace!("loading index...");
                    let container = BPTreeIndex::<K>::try_load(&account_info)?;
                    path.push(key.clone());
                    key = container.lookup(&search_index)?;
                },
                Containers::BPTreeValues => {
                    // log_trace!("[b+tree] loading values...");
                    let container = BPTreeValues::<K,V>::try_load(&account_info)?;
                    path.push(key.clone());
                    match container.binary_search(&search_values) {
                        Ok(index) => {
                            // log_trace!("[b+tree] found value at index: {}", index);
                            let cell = &container.data[index];
                            let v = cell.value.clone(); //container.data[index].value.clone();
                            let _k = cell.key.clone(); //container.data[index].key.clone();
                            value = Some(v);
                            leaf_index = Some(index);
                            // let values_pubkey = Some(container.pubkey().clone());
                        },
                        Err(index) => {
                            leaf_index = Some(index);
                            // log_trace!("[b+tree] values not found");
                        }
                    }

                    break;
                },
                _ => {
                    panic!("bptree lookup invoked with unsupported container type")
                }
            }
        }

        let ctx = BPTreeLookupContext {
                // tree : tr,
                path,
                value,
                leaf_index
            };
            
            Ok(ctx)
    }
  
    #[derive(Debug)]
    pub struct BPTreeIteratorInner 
    // where 
    //     K : Key, //Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display + Default,    
    {
        // cursor : Rc<RefCell<(Pubkey, usize)>
        pubkey : Pubkey,
        index : usize,
        end_of_values : bool,
        // last_key : Option<K>,
    }

    pub struct BPTreeIterator
    {
        inner : Rc<RefCell<BPTreeIteratorInner>>
    }

    pub async fn make_iterator<'info,'refs,K,V>(tree : &BPTree<'info,'refs,K,V>, key : Option<&K>) -> Result<BPTreeIterator> 
    where 
        K : Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display + Default,
        V : Default + Clone + Copy + std::fmt::Debug + std::fmt::Display
    {

        let key = match key {
            Some(key) => { key.clone() },
            None => {
                K::default()
            }
        };
        let ctx = lookup(tree,&key).await?;
        let pubkey = ctx.leaf_pubkey()?;

        let inner = BPTreeIteratorInner {
            pubkey,
            index : ctx.leaf_index.unwrap(),
            end_of_values : false,
        };

        let iterator = BPTreeIterator {
            inner : Rc::new(RefCell::new(inner)),
        };

        Ok(iterator)
    }

    pub async fn get_entries_with_iterator<K,V>(iterator : &BPTreeIterator, items : usize)
    -> Result<Vec<(K,V)>> 
    where 
        K : Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display + Default,
        V : Default + Clone + Copy + std::fmt::Debug + std::fmt::Display
    {
        let mut list = Vec::new();
        let transport = crate::transport::Transport::global()?;
        // let cache = crate::store::Store::global()?;

        let mut iterator = iterator.inner.borrow_mut();
        let mut index = iterator.index;
        let mut pubkey = iterator.pubkey;

        log_trace!("iterator: {:?}", iterator);

        loop {
            let values_account_data = match transport.clone().lookup(&pubkey).await? {
                Some(values_account_data) => values_account_data,//.write().await,
                None => {
                    return Err(error_code!(ErrorCode::BPTreeValuesDereference));
                }
            };

            let mut values_account_data = values_account_data.account_data.lock()?;

            // let mut values_account_data = values_account_data_ref_cell.borrow_mut();
            let pubkey_ = pubkey.clone();
            let values_account_info = (&pubkey_, &mut *values_account_data).into_account_info();
            let values = BPTreeValues::<K,V>::try_load(&values_account_info)?;

            log_trace!("values data len: {}", values.data.len());
            if index < values.data.len() {
                let last = std::cmp::min(index+items,values.len());
                log_trace!("last: {}",last);
                for idx in index..last {
                    let cell = values.data.get_at(idx);
                    list.push((cell.key,cell.value))
                }
                index = last;



                // let available = values.len() - index;
                // if available > 0 {
                //     for 
                // }
            }

            if list.len() == items {
                break;
            }

            let meta = values.meta.borrow();
            pubkey = meta.next;

            if pubkey == Pubkey::default() {
                iterator.end_of_values = true;
                break;
            }

        }

        iterator.index = index;
        iterator.pubkey = pubkey;

        Ok(list)
    }

    // pub async fn lookup<'info,'refs,K,V>(tree : &BPTree<'info,'refs,K,V>, k : &K) -> std::result::Result<BPTreeLookupContext<V>,String> 
    pub async fn get_entries<'info,'refs,K,V>(tree : &BPTree<'info,'refs,K,V>, key : Option<&K>, items : usize) -> Result<Vec<(K,V)>> 
    // pub async fn lookup<'info,'refs,K,V>(root : &Pubkey, k : &K) -> std::result::Result<BPTreeLookupContext<V>,String> 
    where 
        K : Eq + Ord + PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display + Default,
        V : Default + Clone + Copy + std::fmt::Debug + std::fmt::Display
    {
        let mut list = Vec::new();  
        let transport = crate::transport::Transport::global()?;

        log_trace!("get_entries for key: {:?}", key);

        // ^ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // ^ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // ^ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        let default = &K::default();
        let key = match key {
            Some(key) => {
                key
            },
            None => {
                default
            }
        };
        // let key = key.unwrap();
        // ^ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // ^ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // ^ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        let ctx = lookup(tree,key).await?;
        let mut pubkey = ctx.leaf_pubkey()?;
        let mut search_by_key = true;

        loop {

            log_trace!("search by key {:?}", search_by_key);
            log_trace!("searching for pubkey {:?}",pubkey);

            let values_account_data_ref_cell = transport.clone().lookup(&pubkey).await?
                .ok_or(error_code!(ErrorCode::BPTreeValuesDereference))?;
            // {
            //     Some(ref_cell) => ref_cell,
            //     None => {
            //         return Err(error_code!(ErrorCode::BPTreeValuesDereference));
            //     }
            // };

            let mut values_account_data = values_account_data_ref_cell.account_data.lock()?;
            let pubkey_ = pubkey.clone();
            let values_account_info = (&pubkey_, &mut *values_account_data).into_account_info();
            let values = BPTreeValues::<K,V>::try_load(&values_account_info)?;

            let idx = if search_by_key {
                search_by_key = false;
                values.binary_search_idx_by_key(&key)
            } else {
                0
            };
            log_trace!("idx: {:?}", idx);
            let available = values.len() - idx;
            let needed = items - list.len();

            let transfer = if available > needed { needed } else { available };

            for i in idx..transfer {
                let cell = values.data.get_at(i);
                list.push((cell.key, cell.value));
            }

            if list.len() < items {
                let meta = values.meta.borrow();
                pubkey = meta.next.clone();
                if pubkey == Pubkey::default() {
                    log_trace!("pubkey == Pubkey::default");
                    break;
                }
            } else {
                break;
            }
            // while list. {
            // };
            // break;
        }
        // BPTreeValues

        // ^ TODO: SEARCH


        Ok(list)

    }
  
    // pub fn create<K,V>(path: &[Pubkey], k: &K, v : &V) -> Result<()> {
    //     Ok(())
    // }

    

    // pub fn lookup<K,V>(k : K) -> Result<V> {

    //     todo!()
    // }

    // pub account_data_to_btree_container

    // account_data_to_btree_container

    // pub async fn fetch_node<K,V>(pubkey: Pubkey) -> Result<BPTreeContainer<'info,'refs,K,V>> 
    // pub async fn fetch_node<K,V>(pubkey: Pubkey) -> Result<BPTreeContainer<'info,'refs,K,V>> 
    
    // {
    //     let account_data = Transport::get_account_data(pubkey,None).await?;

    //     Ok(account_data)
    // }

    // pub type HandlerFn = fn(ctx: &ContextReference) -> ProgramResult;

    // pub fn handler_execute(cache:&Cache, builder: &InstructionBuilder) -> std::result::Result<(),String> {

}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    // use crate::prelude::client::*;
    use crate::emulator::Simulator;
    use solana_program::account_info::IntoAccountInfo;
    use super::InsertionIndex;

    // #[derive(Debug)]
    #[container(0xfefe, u16)]
    pub struct TestContainer<'info,'refs> 
    {
        pub meta : RefCell<&'info mut BPTreeMetaIndex>,
        pub store : SegmentStore<'info,'refs>,

        pub tree : BPTree<'info,'refs,InsertionIndex,Pubkey>,
    }

    // #[cfg(not(target_arch = "wasm32"))]
    #[async_std::test]
    async fn btree_init() -> Result<()> {
        // workflow_allocator::container::registry::init()?;
        workflow_allocator::init()?;

    // smol::block_on(async {

        let simulator = Simulator::try_new_for_testing()?;

        let builder = simulator.new_instruction_builder()
            .with_account_templates(2)
            .with_sequence(0u64)
            .seal()?;
        let mut sequence = builder.sequence(); 
        let accounts = builder.template_accounts().clone();
        let test_container_pubkey = accounts.first().unwrap().pubkey;

        simulator.execute_handler(builder,|ctx:&ContextReference| {
            // log_trace!("ctx.template_accounts[0].key.to_string()1111: {:?}", ctx.template_accounts[0].key.to_string());
            let allocation_args = AccountAllocationArgs::default();
            let account = ctx.create_pda(TestContainer::initial_data_len(), &allocation_args)?;
            let _test_container = TestContainer::try_create(account)?;
            // log_trace!("create test container successful...");
            Ok(())
        }).await?;

        for idx in 0..100u32 {
        // for idx in 0..1u32 {
            log_trace!("\n");
            log_trace!("{} {}",style("~~~ inserting item ~~>").white().on_blue(), idx);

            // load test container
            let mut test_container_account_data = simulator.store.lookup(&test_container_pubkey).await?
                // .ok_or(error!("missing test_container"))?
                .unwrap().account_data.lock()?.clone();
            let test_container_account_info = (&test_container_pubkey, &mut test_container_account_data).into_account_info();
            let test_container = TestContainer::try_load(&test_container_account_info)?;

            // get tree path (branch) to the value
            let key = InsertionIndex::new(0,idx as u64);
            // log_trace!("~ tree lokup...");
            let lookup_ctx = client::lookup(&test_container.tree,&key).await?;
            let path = lookup_ctx.path.clone();

            // log_trace!("~ building index account descriptors...");
            let index_accounts : Vec<AccountMeta> = 
                path
                .iter()
                .map(|pubkey|
                    AccountMeta::new(*pubkey, false)
                ).collect();
// AccountMeta

            log_trace!("~ setting up builder...");

            let hib: [u8; 4] = unsafe { std::mem::transmute(idx.to_le()) };

            // load test container

            let builder = simulator.new_instruction_builder()
                .with_index_accounts(
                    &index_accounts
                )
                .with_handler_accounts(&[
                    AccountMeta::new(test_container_pubkey.clone(),false)
                ]).with_account_templates(2)
                .with_sequence(sequence)
                .with_instruction_data(&hib)
                .seal()?;
            
            sequence = builder.sequence();
            simulator.execute_handler(builder,|ctx:&ContextReference| {
                let test_container_account = &ctx.handler_accounts[0];
                let test_container = TestContainer::try_load(test_container_account)?;
                let allocation_args = AccountAllocationArgs::default();

                let v = ctx.instruction_data[0] as u32 | ((ctx.instruction_data[1] as u32) << 8);
                // log_trace!("test creating instance: {:?}", v);

                let key = InsertionIndex::new(0,v as u64);
                let value = generate_random_pubkey();
                let path = ctx.index_accounts;
                test_container.tree.insert(
                    // &[],
                    path,
                    ctx,
                    &key,
                    &value,
                    &allocation_args,
                )?;
                
                Ok(())
            }).await?;
    
        }
    
        log_trace!("...");
        simulator.store.list().await?;


        let mut test_container_account_data = simulator.store.lookup(&test_container_pubkey).await?
        // .ok_or(error!("missing test_container"))?.read()?.clone();
        .unwrap().account_data.lock()?.clone();
        let test_container_account_info = (&test_container_pubkey, &mut test_container_account_data).into_account_info();
        let test_container = TestContainer::try_load(&test_container_account_info)?;
        

        let iterator = client::make_iterator(&test_container.tree, None).await?;
        log_trace!("... first pass ...");
        let list = test_container.tree.get_entries_with_iterator(&iterator, 10).await?;
        // let list = client::get_entries(
        //     &test_container.tree,
        //     None,
        //     10
        // )?;
        list.iter().for_each(|ii| { log_trace!("{} -> {}",ii.0,ii.1) });
        // log_trace!("list: {:?}", list);

        log_trace!("... second pass ...");
        // let last_key = list.last().unwrap().0;
        // let list = client::get_entries(
        //     &test_container.tree,
        //     Some(&last_key),
        //     10
        // )?;
        // let list = client::get_entries_with_iterator(&iterator, 10)?;

        let list = test_container.tree.get_entries_with_iterator(&iterator, 10).await?;
        list.iter().for_each(|ii| { log_trace!("{} -> {}",ii.0,ii.1) });
        // log_trace!("list: {:?}", list);

        Ok(())
        // })
    }
}
