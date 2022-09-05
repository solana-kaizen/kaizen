// use std::cell::Ref;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc, mem};
// use std::convert::AsRef;
use crate::realloc::account_info_realloc;
use solana_program::account_info::AccountInfo;
use crate::result::Result;
use crate::error::*;
use crate::container::array::Array;
use crate::utils;
use workflow_core::enums::u16_try_from;
use num::Integer;

// #[cfg(not(target_arch = "bpf"))]
use workflow_log::*;

pub const SEGMENT_STORE_MAGIC : u32 = 0x47455347;
pub const SEGMENT_STORE_VERSION : u32 = 1;

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct SegmentStoreMeta {
    magic : u32,
    version : u32,
    payload_len: u16,
    index_unit_size : u16,
    segments : u32,
}

impl SegmentStoreMeta {
    pub fn from<'a>(data: &Rc<RefCell<&'a mut [u8]>>, offset : usize) -> &'a mut SegmentStoreMeta {
        let data = data.borrow();
        unsafe { &mut *((data[offset..]).as_ptr() as *mut SegmentStoreMeta) }
    }
}

#[derive(Debug, Clone)]
pub struct Segment<'info, 'refs> {
    pub store: SegmentStore<'info, 'refs>,
    // pub store: &SegmentStore<'info, 'refs>,
    pub idx : usize,
    pub resizable : bool,
}

impl<'info, 'refs> Segment<'info, 'refs> {

    pub fn data_len_min() -> usize { 0 }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Segment<'info, 'refs>> {
        Ok((*segment).clone())
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Segment<'info,'refs>> {
        Ok((*segment).clone())
    }

    //

    pub fn from(store : &SegmentStore<'info,'refs>, idx: usize) -> Result<Segment<'info, 'refs>> {
        Ok(Segment { store : store.clone(), idx, resizable : true })
    }

    pub fn try_create(store : &SegmentStore<'info,'refs>, data_len: usize) -> Result<Segment<'info, 'refs>> {
        store.try_allocate_segment(data_len)
    }

    pub fn fix(&mut self) {
        self.resizable = false;
    }

    pub fn try_resize(&self, new_data_len : usize, zero_init: bool) -> Result<()> {
        if self.resizable {
            self.store.try_resize_segment(self.idx,new_data_len,zero_init)
        } else {
            Err(ErrorCode::SegmentNotResizable.into())
        }
    }

    pub fn try_get_segment_data_len(&self) -> Result<usize> {
        self.store.try_get_segment_data_len(self.idx)
    }

    pub fn get_data_len(&self) -> usize {
        self.store.get_segment_data_len(self.idx)
    }

    pub fn try_get_offset(&self) -> Result<usize> {
        self.store.try_get_segment_offset(self.idx)
    }

    pub fn get_offset(&self) -> usize {
        self.store.get_segment_offset(self.idx)
    }

    // pub fn as_ref_unsafe(&self) -> &[u8] {
    //     self.store.get_segment_ref(self.idx)
    // }

    pub fn try_as_ref_mut_u8(&self) -> Result<&mut [u8]> {
        self.store.try_get_segment_ref_mut_u8(self.idx)
    }

    pub fn try_as_ref_u8(&self) -> Result<&[u8]> {
        self.store.try_get_segment_ref_u8(self.idx)
    }

    pub fn as_ref_u8(&self) -> &[u8] {
        self.store.get_segment_ref_u8(self.idx)
    }

    pub fn as_ref_mut_u8(&self) -> &mut [u8] {
        self.store.get_segment_ref_mut_u8(self.idx)
    }

    pub fn as_struct_ref<T>(&self) -> &'info T {
        utils::account_buffer_as_struct_ref::<T>(self.store.account,self.get_offset())
    }

    pub fn try_as_struct_ref<T>(&self) -> Result<&T> where T : 'info {
        if self.get_data_len() != std::mem::size_of::<T>() {
            return Err(ErrorCode::SegmentStorageSize.into());
        }
        let struct_ref = utils::account_buffer_as_struct_ref(self.store.account,self.get_offset());
        Ok(struct_ref)
    }

    pub fn as_struct_mut<T>(&self) -> &'info mut T {
        utils::account_buffer_as_struct_mut(self.store.account,self.get_offset())
    }

    pub fn try_as_struct_mut<T>(&self) -> Result<&'info mut T> {
        if self.get_data_len() != std::mem::size_of::<T>() {
            return Err(ErrorCode::SegmentStorageSize.into());
        }
        let struct_mut_ref = utils::account_buffer_as_struct_mut(self.store.account,self.get_offset());
        Ok(struct_mut_ref)
    }

    pub fn as_slice<T>(&self) -> &[T] where T : 'info {
        let elements = self.get_data_len() / mem::size_of::<T>();
        utils::account_buffer_as_slice(self.store.account,self.get_offset(),elements)
    }

    pub fn as_slice_mut<T>(&self) -> &mut [T] where T : 'info {
        let elements = self.get_data_len() / mem::size_of::<T>();
        utils::account_buffer_as_slice_mut(self.store.account,self.get_offset(),elements)
    }

}


// impl<'info,'refs> AsRef<[u8]> for Segment<'info,'refs> {
//     fn as_ref(&self) -> &[u8] {
//     // fn as_ref(&self) -> &'info [u8] {
//         self.store.get_segment_ref(self.idx)
//     }
// }

// impl<'info,'refs> AsMut<[u8]> for Segment<'info,'refs> {
//     fn as_mut(&mut self) -> &mut [u8] {
//     // fn as_mut(&mut self) -> &'info mut [u8] {
//         self.store.get_segment_ref_mut(self.idx)
//     }
// }


u16_try_from!(
    #[derive(Debug, Copy, Clone)]
    pub enum IndexUnitSize {
        Bits16 = 2,
        Bits32 = 4,
    }
);

// pub struct IndexUnit<T>(T);

pub trait IndexUnit {
    fn from_usize(v: usize) -> Self;
    fn as_usize(v: Self) -> usize;
    // fn as_usize(v: Self) -> usize;
    fn value(v: Self) -> usize;
}

#[macro_export]
macro_rules! impl_index_unit {
    ($($ty:ty)*) => {
        $(
            impl IndexUnit for $ty {
                #[inline]
                fn from_usize(v: usize) -> $ty {
                    v as $ty
                }
                #[inline]
                fn as_usize(v:$ty) -> usize {
                    v as usize
                }
                #[inline]
                fn value(v:$ty) -> usize {
                    v as usize
                }
            }
        )*
    }
}

pub use impl_index_unit;

impl_index_unit!(u16 u32);
// impl_index_unit!(u8 u16 u32 u64 usize);


fn value_of<T>(v:T) -> usize where T : IndexUnit {
    T::value(v)
}

#[derive(Debug)]
pub struct IndexInfo {
    pub offset: usize,
    pub size: usize
}

impl IndexInfo {
    pub fn new(offset: usize, size: usize) -> IndexInfo {
        IndexInfo { offset, size }
    }
}

#[derive(Debug, Copy)]
pub struct IndexEntry<T : Integer+IndexUnit> {
    pub offset: T,
    pub size : T,
}

impl<T> Clone for IndexEntry<T> where T : Integer+IndexUnit+Copy {
    fn clone(&self) -> IndexEntry<T> {
        IndexEntry {
            offset : self.offset,
            size : self.size,
        }
    }
}

impl<T> IndexEntry<T> where T : Integer+IndexUnit+Copy {
    pub fn new(offset: usize, size: usize) -> IndexEntry<T> {
        IndexEntry {
            offset : IndexUnit::from_usize(offset.into()),
            size : IndexUnit::from_usize(size.into()),
        }
    }

    pub fn zero(&mut self) {
        self.offset = IndexUnit::from_usize(0);
        self.size = IndexUnit::from_usize(0);
    }

    pub fn next_offset(&self) -> usize {
        value_of(self.offset) + value_of(self.size)
    }
}

#[derive(Debug)]
pub struct Layout<T : Debug+Integer+IndexUnit+Copy> {
    user_segment_sizes : Vec<usize>,
    phantom : PhantomData<T>
}

impl<T> Layout<T> where T : Debug+Integer+IndexUnit+Copy {

    pub fn from(segments: &[usize]) -> Layout<T> {
        Layout {
            user_segment_sizes : segments.to_vec(),
            phantom : PhantomData
        }
    }

    pub fn set_segment_size(&mut self, idx:usize, size:usize) {
        self.user_segment_sizes[idx] = size;
    }

    pub fn get_segment_size(&mut self, idx:usize) -> usize {
        self.user_segment_sizes[idx]
    }

    pub fn data_len(&self) -> usize {
        let segments = self.user_segment_sizes.len()+1;

        mem::size_of::<SegmentStoreMeta>() +
        Layout::<T>::calculate_index_size(segments) +
        self.user_segment_sizes.iter().sum::<usize>()
    }

    pub fn generate_index(&self, offset : usize) -> Vec<IndexEntry<T>> {
        let segments = self.user_segment_sizes.len()+1;
        let mut index = Vec::with_capacity(segments);
        let mut offset = offset + mem::size_of::<SegmentStoreMeta>();
        let mut size = Layout::<T>::calculate_index_size(segments);
        index.push(IndexEntry::new(offset, size));
        for idx in 0..self.user_segment_sizes.len() {
            offset += size;
            size = self.user_segment_sizes[idx];
            index.push(IndexEntry::new(offset, size));
        }
        index
    }

    pub fn calculate_index_size(segments:usize) -> usize {
        segments * mem::size_of::<IndexEntry<T>>()
    }

}

#[derive(Debug, Clone, Copy)]
pub struct SegmentStore<'info, 'refs> {
    pub account : &'refs AccountInfo<'info>,
    offset : usize,
    index_unit_size : IndexUnitSize
}

impl<'info, 'refs> SegmentStore<'info, 'refs> {

    pub fn try_create<T>(
        account: &'refs AccountInfo<'info>,
        offset: usize,
        layout : &Layout<T>
        // layout : &SegmentLayout
    ) -> Result<SegmentStore<'info, 'refs>> where T : 'info+Integer+Debug+IndexUnit+Copy {
        // let segments = layout.user_segment_sizes.len()+1;

        // log_trace!("%%%%%%% -> SegmetStore::try_create() offset: {:?}", offset);


        let index = layout.generate_index(offset);
        log_trace!("| {} {:?}", style("segment store index entries:").green(), index);
        #[cfg(feature = "check-buffer-sizes")] {
            let data_len_needed = index[index.len()-1].next_offset();// - index[0].offset as usize;
            if account.data_len() < data_len_needed {
                log_trace!("segment store create...");
                log_trace!("data_len_needed: {}  account.data_len(): {}", data_len_needed, account.data_len());
                log_trace!("\n-------------------");
                log_trace!("{:#?}", layout);
                log_trace!("-------------------\n");
                return Err(ErrorCode::AccountSizeTooSmall.into());
            }
        }

        let index_unit_size = match std::mem::size_of::<T>() {
            2 => IndexUnitSize::Bits16,
            4 => IndexUnitSize::Bits32,
            _ => panic!("only 16 and 32 bit indexes are currently supported"),
        };

        let store = SegmentStore {
            account,
            offset,
            index_unit_size
        };
        // store.try_init_meta(&index)?;
        store.try_init_meta(&index)?;
        // store.try_init_meta(index_unit_size, layout)?;
        Ok(store)
    }

    pub fn try_load(account: &'refs AccountInfo<'info>, offset: usize) -> Result<SegmentStore<'info, 'refs>> {
        // pub fn try_from_account(account: &'refs AccountInfo<'info>, offset: usize) -> Result<SegmentStore<'info, 'refs>> {

        // log_trace!("%%%%%%% -> SegmetStore::try_load() offset: {:?}", offset);

        let data = account.data.try_borrow()?;

        if data.len() == 0 {
            log_trace!("\n{}\n",style("* * * SegmentStore::try_load() error - trying to load from a blank account * * *").red());
            return Err(error_code!(ErrorCode::AccountIsBlank).with_account(account.key));
        }
        
        if data.len() < mem::size_of::<SegmentStoreMeta>()+1 {
            log_trace!("\n{}\n",style("* * * SegmentStore::try_load() error - account size is too small (below SegmentStoreMeta) * * *").red());
            return Err(error_code!(ErrorCode::AccountSizeTooSmall).with_account(account.key));
        }

        let meta = SegmentStoreMeta::from(&account.data, offset);
        let magic = meta.magic;
        if magic != SEGMENT_STORE_MAGIC {

            log_trace!("segment store magic {:#x} should be: {:#x}",magic,SEGMENT_STORE_MAGIC);
            log_trace!("meta: {:#?}", meta);
            log_trace!("loading segment store from offset {}", offset);
            #[cfg(not(target_arch = "bpf"))]
            log_trace!("{}",format_hex(&data));
            return Err(error_code!(ErrorCode::SegmentStoreMagic)
                .with_account(account.key));
        }

        // FIXME unit size result handling
        let index_unit_size = meta.index_unit_size.try_into().unwrap(); 

        let store = SegmentStore {
            account,
            offset,
            index_unit_size,
            // phantom : PhantomData
        };

        Ok(store)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.get_meta().segments as usize
    }

    #[inline(always)]
    pub fn get_meta(&self) -> &mut SegmentStoreMeta {
        SegmentStoreMeta::from(&self.account.data, self.offset)
    }


    //? ----------------------------------------------------------------
    // TODO -
    pub fn try_allocate_segment<'store>(&self, _data_len : usize) -> Result<Segment<'info,'refs>> {
        // let meta = self.get_meta();
        // let index = self.try_get_segment_at
        let idx = self.len(); //meta.segments as usize;
        let segments = idx+1;
        self.get_meta().segments = segments as u32;
        let index_size = segments * self.index_unit_size as usize; //mem::size_of::<u32>();
        if self.get_segment_data_len(0) < index_size {
            self.try_resize_segment(0, index_size, false)?;

            #[cfg(test)]
            assert_eq!(index_size,self.get_segment_data_len(0));
        }

        // let bytes_used = meta.bytes_used;
        // let buffer_len = self.account.data_len();
        // let available_len = buffer_len - bytes_used;
        // if data_len < available_len {
        //     if meta.indices 
        // } else {
        // }

        let segment = Segment::<'info,'refs> {
            store : self.clone(),
            idx,
            resizable : true,
        };

        // FIXME implement try_allocate_segment
        // todo!("finish implementation");
        Ok(segment)
    }
    pub fn try_purge_segment(&mut self, idx: usize) -> Result<()> {
        match self.index_unit_size {
            IndexUnitSize::Bits16 => {
                Ok(self.try_purge_segment_impl::<u16>(idx)?)
            },
            IndexUnitSize::Bits32 => {
                Ok(self.try_purge_segment_impl::<u32>(idx)?)

            }
        }
    }

    pub fn try_purge_segment_impl<T>(&mut self, idx: usize) -> Result<()> where T : 'info+Integer+IndexUnit+Copy {
        let segments = self.len();
        if idx >= segments || idx == 0 {
            return Err(ErrorCode::SegmentStorageBounds.into());
        }

        let dest = self.get_segment_offset(idx);
        // let sizes = self.get_segment_sizes().borrow();
        let index = self.get_index::<T>();
        let segment_data_len = IndexUnit::as_usize(index[idx].size);
        let src = dest+segment_data_len;
        let account_data_len = self.account.data_len();
        let migration_data_len = account_data_len - src;

        {
            let mut data = self.account.data.borrow_mut();
            data[..].copy_within(
                src..(src+migration_data_len),
                dest
            );
        }
log_trace!("ALLOC A");
        account_info_realloc(self.account, account_data_len - segment_data_len, false,false)?;

        //? TODO
        //? TODO
        //? TODO
        //? TODO
        //? TODO

        // we do not reize the index segment (there is potential for too much memory movement)
        //let mut segment_offsets = self.segment_offsets.borrow_mut();
        for k in idx..(segments-1) {

            index[k].size = index[k+1].size;
            index[k].offset = index[k+1].offset - IndexUnit::from_usize(segment_data_len);
            
            // let dest = &mut index[k];
            // let src = &mut index[k+1];
            // dest.size = src.size;
            // // index[k] = sizes[k+1];
            // dest.offset = src.offset - segment_data_len as u32;
            // segment_offsets[k] = segment_offsets[k+1] - segment_data_len;
        }

        // TODO - zero memory
        // TODO - zero memory
        // TODO - zero memory
        // TODO - zero memory

        index[segments].zero();
        // segment_offsets.pop();

        Ok(())
    }

    pub fn try_resize_segment(&self, idx: usize, new_len : usize, zero_init : bool) -> Result<()> {
        match self.index_unit_size {
            IndexUnitSize::Bits16 => {

                if new_len > 0xffff {
                    #[cfg(not(target_arch = "bpf"))]
                    panic!("segment size is too large for 16 bit indexes");
                    #[cfg(target_arch = "bpf")]
                    return Err(error_code!(ErrorCode::SegmentSizeTooLargeForIndexUnitSize));
                }

                log_trace!("try_resize_segment");

                self.try_resize_segment_impl::<u16>(idx,new_len,zero_init)
            },
            IndexUnitSize::Bits32 => {
                self.try_resize_segment_impl::<u32>(idx,new_len,zero_init)
            },
        }
    }

    // ^ TODO
    // ^ TODO
    // ^ TODO
    // ^ TODO
    // ^ TODO
    // ^ TODO
    pub fn try_resize_segment_impl<T>(&self, idx: usize, new_len : usize, zero_init : bool) 
    -> Result<()> 
    where T : 'info + Integer + IndexUnit + std::fmt::Debug + std::ops::SubAssign + Copy + std::ops::AddAssign
    {
        let segments = self.len();
        if idx >= segments {
            return Err(ErrorCode::SegmentStorageBounds.into());
        }
        let next_idx = idx+1;

        let index = self.get_index::<T>();
        // log_trace!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // log_trace!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // log_trace!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // log_trace!("index before: {:?}", index);
        // log_trace!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // log_trace!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // log_trace!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // let total_size
        let segment_data_len = IndexUnit::as_usize(index[idx].size);//.as_usize(); // as usize;
        // let segment_data_len = index[idx].size.as_usize(); // as usize;
        // log_trace!("@@@\n@@@\n@@@\n idx: {} new_len: {}  segment_data_len: {} \n@@@\n@@@", idx, new_len, segment_data_len);
        if new_len > segment_data_len {
            let delta = new_len - segment_data_len;
            let account_data_len = self.account.data_len();

            // !  - - - - - - - - - - - -
            // !  - - - - - - - - - - - -
            // ^ TODO:  ACCOUNT DATA LEN MUST BE THE SUM OF ALL SEGMENTS
            // ^ TODO:  ACCOUNT DATA LEN MUST BE THE SUM OF ALL SEGMENTS
            // let headers = accounts::account_info_headers(self.account)?;
            // log_trace!("{} serialized: {} slice: {}",style("HEADERS ===============>").white().on_red(),headers.0,headers.1);


            // let new_account_data_len = account_data_len + delta;
            // let migration_data_len = if next_idx == segments {
            //     0
            // } else {
            //     account_data_len - IndexUnit::as_usize(index[idx+1].offset) // as usize
            // };
            // log_trace!("ALLOC B - account_data_len: {}, new_account_data_len: {}", account_data_len, new_account_data_len);

            // account_info_realloc(self.account, new_account_data_len, false,false)?;
// ^ TODO:  ACCOUNT DATA LEN MUST BE THE SUM OF ALL SEGMENTS
            // ^ TODO:  ACCOUNT DATA LEN MUST BE THE SUM OF ALL SEGMENTS
            // ^ TODO:  ACCOUNT DATA LEN MUST BE THE SUM OF ALL SEGMENTS

            let total_segment_data_len = 
                IndexUnit::as_usize(index[segments-1].offset)
                + IndexUnit::as_usize(index[segments-1].size);

            if account_data_len < total_segment_data_len {
                panic!("account data len is less than total segment data len");
            }

            // let new_account_data_len = account_data_len + delta;
            let new_account_data_len = total_segment_data_len + delta;
            let migration_data_len = if next_idx == segments {
                0
            } else {
                total_segment_data_len - IndexUnit::as_usize(index[next_idx].offset) // as usize
            };
            // log_trace!("ALLOC B - account_data_len: {}, new_account_data_len: {}", account_data_len, new_account_data_len);
            // log_trace!("{:#?}", self.account);
            // let headers = accounts::account_info_headers(self.account)?;
            // log_trace!("{} serialized: {} slice: {}",style("===============>").white().on_red(),headers.0,headers.1);
            // log_trace!("{} migration data len: {}",style("===============>").white().on_red(),migration_data_len);
            if new_account_data_len > account_data_len {
                account_info_realloc(self.account, new_account_data_len, false,false)?;
            } else {
                // log_trace!("{}",style("~ ~ ~ ~ ~ ~ ~ ~ SKIPPING ALLOCATION ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~").white().on_red());
            }
            // ^ TODO:  ACCOUNT DATA LEN MUST BE THE SUM OF ALL SEGMENTS

            // log_trace!("index after: {:?}", index);
            let index = self.get_index::<T>();
            // log_trace!("index reaquire: {:?}", index);
            // log_trace!("index[{}] = {}",idx,new_len);
            index[idx].size = IndexUnit::from_usize(new_len);//  as u32;

            let mut data = self.account.data.borrow_mut();
            if migration_data_len != 0 {
                let src = IndexUnit::as_usize(index[next_idx].offset);// as usize;
                let dest = src+delta;
                data[..].copy_within(
                    src..(src+migration_data_len),
                    dest
                );

                for k in next_idx..segments {
                    index[k].offset += IndexUnit::from_usize(delta);// as u32;
                }

                if zero_init {
                    // TODO: cleanup, set to 0
                    data[src..src+delta].fill(88);
                }
            }

            // log_trace!("data[{}]: {:?}",data.len(), data);


        } else if new_len < segment_data_len {

            log_trace!("!!!!!!!!!!!!!!!!!!");
            log_trace!("!!!!!!!!!!!!!!!!!!");
            log_trace!("!!!!!!!!!!!!!!!!!!");
            log_trace!("[segment store] reduce segment size... idx: {}", idx);
            log_trace!("[segment store] segment_data_len: {}", segment_data_len);
            log_trace!("[segment store] new_len: {}", new_len);
            let delta = segment_data_len - new_len;
            let account_data_len = self.account.data_len();
            log_trace!("[segment store] delta: {}", delta);
            log_trace!("[segment store] account_data_len: {}", account_data_len);
            let new_account_data_len = account_data_len - delta;

            if idx < segments-1 {
                let src = index[idx].next_offset();
                log_trace!("[segment store] src:{}", src);
                //self.get_segment_offset(idx+1);

                let migration_data_len = account_data_len - src;

                log_trace!("[segment store] resize [reduce segment size] segment[{}] account_data_len: {} delta: {}  new_account_data_len: {}",
                    idx, self.account.data_len(), delta, new_account_data_len);
                // let mut segment_offsets = self.segment_offsets.borrow_mut();
                for k in idx..segments {
                    index[k].offset -= IndexUnit::from_usize(delta);// as u32;
                }

                let dest = src-delta;

                {
                    let mut data = self.account.data.borrow_mut();
                    data[..].copy_within(
                        src..(src+migration_data_len),
                        dest
                    );
                }
            }

            index[idx].size = IndexUnit::from_usize(new_len);// as u32;
            log_trace!("ALLOC C");

            account_info_realloc(self.account, new_account_data_len, false,false)?;
            log_trace!("[segment store] reduce segment size is done");
        }

        Ok(())
    }



    #[inline(always)]
    pub fn try_get_segment_data_len(&self, idx: usize) -> Result<usize> {
        let segments = self.len();
        if idx >= segments {
            return Err(ErrorCode::SegmentStorageBounds.into());
        }
        Ok(self.get_segment_data_len(idx))
    }

    #[inline(always)]
    pub fn get_segment_data_len(&self, idx: usize) -> usize {
        match self.index_unit_size {
            IndexUnitSize::Bits16 => {
                let index = self.get_index::<u16>();
                index[idx].size as usize
            },
            IndexUnitSize::Bits32 => {
                let index = self.get_index::<u32>();
                index[idx].size as usize
            }
        }
    }

    pub fn get_index_info_at(&self, idx: usize) -> IndexInfo {
        match self.index_unit_size {
            IndexUnitSize::Bits16 => {
                let index = self.get_index::<u16>();
                IndexInfo::new(index[idx].offset as usize, index[idx].size as usize)
            },
            IndexUnitSize::Bits32 => {
                let index = self.get_index::<u32>();
                IndexInfo::new(index[idx].offset as usize, index[idx].size as usize)
            }
        }
    }

    pub fn get_index<'f, T>(&'f self) -> &'f mut [IndexEntry<T>] where T : 'info+Integer+IndexUnit {

        if std::mem::size_of::<T>() != self.index_unit_size as usize {
            log_trace!("error: index access unit size mismatch");
            assert_eq!(std::mem::size_of::<T>(), self.index_unit_size as usize);
        }

        let data_offset = self.offset + mem::size_of::<SegmentStoreMeta>();
        let index = utils::account_buffer_as_slice_mut(self.account,data_offset, self.len());
        index
    }

    pub fn get_index_at<'f, T>(&'f self, idx:usize) -> &'f IndexEntry<T> where T : 'info+Integer+IndexUnit {
        let index = self.get_index::<T>();
        &index[idx]
    }

    pub fn try_get_segment_offset(&self, idx: usize) -> Result<usize> {
        if idx >= self.len() {
            return Err(ErrorCode::SegmentStorageBounds.into());
        } else {
            Ok(self.get_segment_offset(idx))
        }
    }

    #[inline(always)]
    pub fn get_segment_offset(&self, idx: usize) -> usize {
        match self.index_unit_size {
            IndexUnitSize::Bits16 => {
                let index = self.get_index::<u16>();
                // println!("$$$$$$ :: get_segment_offset[{}]  --> {:?}", idx, index);
                index[idx].offset as usize
            },
            IndexUnitSize::Bits32 => {
                let index = self.get_index::<u32>();
                index[idx].offset as usize
            }
        }
    }

    #[inline(always)]
    pub fn get_segment_ref_u8(&self, idx: usize) -> &[u8] {
        let segment = self.get_index_info_at(idx);
        utils::account_buffer_as_slice::<u8>(self.account, segment.offset, segment.size)
    }
    
    #[inline(always)]
    pub fn get_segment_ref_mut_u8(&self, idx: usize) -> &'info mut [u8] {
        let segment = self.get_index_info_at(idx);
        utils::account_buffer_as_slice_mut::<u8>(self.account, segment.offset, segment.size)
    }
    
    #[inline(always)]
    pub fn try_get_segment_ref_u8(&self, idx: usize) -> Result<&[u8]> {
        let segment = self.get_index_info_at(idx);
        Ok(utils::account_buffer_as_slice::<u8>(self.account, segment.offset, segment.size))
    }

    #[inline(always)]
    pub fn try_get_segment_ref_mut_u8(&self, idx: usize) -> Result<&'info mut [u8]> {
        let segment = self.get_index_info_at(idx);
        Ok(utils::account_buffer_as_slice_mut::<u8>(self.account, segment.offset, segment.size))
    }

    pub fn try_get_segment_at<'store>(&self, idx: usize) -> Result<Rc<Segment<'info,'refs>>> {

        if idx >= self.len() {
            #[cfg(test)]
            log_trace!("try_get_segment() out of bounds idx: {}", idx);
            return Err(ErrorCode::SegmentStorageBounds.into());
        } else {
            Ok(Rc::new(Segment::<'info,'refs>::from(self,idx)?))
        }
    }

    pub fn try_create_linear_store<T>(&self, idx: usize) -> Result<Array<'info,'refs,T>> 
    where T: Copy
    {

        let segment = self.try_get_segment_at(idx)?;
        let linear_store = Array::try_load_from_segment(segment)?;
        linear_store.try_init_meta()?;
        Ok(linear_store)

    }

    pub fn try_get_linear_store<T>(&self, idx: usize) -> Result<Array<'info,'refs,T>> 
    where T: Copy
    {

        let segment = self.try_get_segment_at(idx)?;
        // log_trace!("{:#?}", segment);
        let linear_store = Array::try_load_from_segment(segment)?;
        Ok(linear_store)

    }

    pub fn try_init_meta<T>(&self, new_index : &[IndexEntry<T>]) -> Result<()> where T : 'info+Integer+IndexUnit+Copy+std::fmt::Debug {
        // let meta = SegmentStoreMeta::from(&self.account.data, self.offset);
        let meta = self.get_meta();
        if meta.magic != 0 {
            return Err(ErrorCode::SegmentStoreMetaNotBlank.into())
        }
        // log_trace!("~ ~ ~ init segment store meta magic to {} ({:#x})",SEGMENT_STORE_MAGIC,SEGMENT_STORE_MAGIC);
        meta.magic = SEGMENT_STORE_MAGIC;
        meta.version = SEGMENT_STORE_VERSION;
        meta.payload_len = mem::size_of::<SegmentStoreMeta>() as u16;
        meta.index_unit_size = self.index_unit_size as u16;

        let segments = new_index.len();
        meta.segments = segments as u32;
        // let meta_magic = meta.magic;
        // log_trace!("~ ~ ~ resulting magic to {} ({:#x})",meta_magic,meta_magic);

        let index = self.get_index::<T>();
        // log_trace!("* * * * * * * * INDEX in try_init_meta: {:?}", index);
        // log_trace!("* * * * * * * * incoming INDEX: {:?}", new_index);
        index[0..segments].copy_from_slice(new_index);
        // log_trace!("* * * * * * * * INDEX in try_init_meta: {:?}", index);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::*;
    use crate::container::array::MAPPED_ARRAY_VERSION;
    use crate::container::array::ArrayMeta;

    fn check_lsv<T>(ls:&Array<T>) where T: Copy {
        // {
        //     let data = ls.segment.store.account.data.borrow();
        //     log_trace!("data: {:?}\n---------------------------------\n",data);
        // }

        // log_trace!("list a meta: {:#?}",ls.get_meta());


        assert_eq!(ls.get_meta().get_version(), MAPPED_ARRAY_VERSION);
        assert!(ls.get_meta().records < 16);
    }

    #[test]
    fn allocator_segment_resize() -> Result<()> {

        // log_trace!("hello world");
        let mut container = MockAccountDataInstance::new(128);
        let _account = container.into_account_info();

        // FIXME implement allocator_segment_resize

        Ok(())
    }

    #[test]
    fn allocator_segment_init() -> Result<()> {

        let layout = Layout::<u16>::from(&[
            Array::<u8>::calculate_data_len(1),
            Array::<u16>::calculate_data_len(2),
            Array::<u32>::calculate_data_len(3),
        ]);
        let data_len = layout.data_len();
        assert_eq!(data_len,89);
        assert_eq!(layout.user_segment_sizes,[
            mem::size_of::<ArrayMeta>()+mem::size_of::<u8>()*1,
            mem::size_of::<ArrayMeta>()+mem::size_of::<u16>()*2,
            mem::size_of::<ArrayMeta>()+mem::size_of::<u32>()*3
        ]);

        Ok(())
    }

    #[test]
    fn allocator_segment_test() -> Result<()> {

        let layout = Layout::<u16>::from(&[
            Array::<u8>::calculate_data_len(0),
            Array::<u8>::calculate_data_len(0),
            Array::<u8>::calculate_data_len(0),
        ]);

        // let data_len = layout.data_len();
        // log_trace!("data len: {}", data_len);

        // return Ok(());

        // let aa = AccountContainer::new(128).into_account_info();
        let mut container = MockAccountDataInstance::new(128);
        let account = container.into_account_info();
        // let account = AccountContainer::new(128).into_account_info();
        // let account = AccountContainer::new(128).into_account_info();

//        fill_account_buffer_u8(&account,0, 16, 0xfe);


        let store = SegmentStore::try_create(&account, 0, &layout)?;

        let list_a = store.try_create_linear_store::<u8>(1)?;
        check_lsv(&list_a);

        // {
        //     let data = store.account.data.borrow();
        //     log_trace!("---------------------\naa data: {:?}",data);
        // }

        let list_b = store.try_create_linear_store::<u8>(2)?;
        let list_c = store.try_create_linear_store::<u8>(3)?;

        unsafe { *(list_c.try_allocate(false)?) = 10u8; }

        // {
        //     let data = store.account.data.borrow();
        //     log_trace!("cc data: {:?}\n---------------------------------\n",data);
        // }


        check_lsv(&list_a);




        check_lsv(&list_b);
        check_lsv(&list_c);

        unsafe { 
            *(list_c.try_allocate(false)?) = 20u8;
            *(list_c.try_allocate(false)?) = 30u8;
            *(list_c.try_allocate(false)?) = 40u8;
            *(list_c.try_allocate(false)?) = 50u8;
        }
// log_trace!("A");
// let slice = list_c.as_slice();
// log_trace!(":::::::::::::::::::::::: - {:#?}", slice);
// return Ok(());

        // {
        //     let data = store.account.data.borrow();
        //     log_trace!("data: {:?}",data);
        // }

        check_lsv(&list_a);
        check_lsv(&list_b);
        check_lsv(&list_c);

        
        
        // log_trace!("list a meta: {:#?}",list_a.get_meta());
        // log_trace!("list b meta: {:#?}",list_b.get_meta());
        // log_trace!("list c meta: {:#?}",list_c.get_meta());
        //? ---
        // {
        //     let data = store.account.data.borrow();
        //     log_trace!("---\nBEFORE: data: {:?}",data);
        // }
        unsafe { *(list_a.try_allocate(false)?) = 177u8; }
        // {
        //     let data = store.account.data.borrow();
        //     log_trace!(" AFTER: data: {:?}\n---",data);
        // }


        // let slice = list_a.as_slice();
        // log_trace!(":::::::::::::::::::::::: - {:#?}", slice);

        // log_trace!("list b1 meta: {:#?}",list_b.get_meta());
        // {
        //     let data = store.account.data.borrow();
        //     log_trace!("data: {:?}",data);
        // }
        //? ---


        check_lsv(&list_a);
        check_lsv(&list_b);
        check_lsv(&list_c);

        unsafe {

            *(list_a.try_allocate(false)?) = 2u8;
            // log_trace!("list b2 meta: {:#?}",list_b.get_meta());
            *(list_a.try_allocate(false)?) = 3u8;
    // log_trace!("B");
            *(list_b.try_allocate(false)?) = 4u8;
            // log_trace!("list b3 meta: {:#?}",list_b.get_meta());
            *(list_b.try_allocate(false)?) = 5u8;
            *(list_b.try_allocate(false)?) = 6u8;

        }

//        list_c.try_remove_at(2,true,false)?;
        // let list_x = store.try_get_linear_store::<u8>(2)?;
        // log_trace!("list x1 meta: {:#?}",list_x.get_meta());

// // log_trace!("list: {:#?}", list_a);
        // for idx in 1..4 {
        //     let ptr = list_c.try_insert(true)?;
        //     *ptr = idx as u8;
        // }

        // for idx in 1..6 {
        //     let ptr = list_b.try_insert(true)?;
        //     *ptr = idx as u8;
        // }
        // for idx in 1..8 {
        //     let ptr = list_a.try_insert(true)?;
        //     *ptr = idx as u8;
        // }

        // {
        //     let data = store.account.data.borrow();
        //     log_trace!("data: {:?}",data);
        // }
//        log_trace!("account: {:#?}",data);

        // let account_data_len = 128;
        // let mut account_data = AccountData::new_as_detached_container(account_data_len);

        // let account_info : AccountInfo = (account_data;



        Ok(())
    }
}