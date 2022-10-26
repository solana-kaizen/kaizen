use std::{cell::RefCell, rc::Rc, mem};
use std::marker::PhantomData;
use std::ops::{Index,IndexMut};
use solana_program::account_info::AccountInfo;
use workflow_allocator_macros::Meta;
use crate::container::segment::Segment;
use crate::result::Result;
use crate::error::*;
use crate::utils;
use workflow_log::*;

pub const MAPPED_ARRAY_VERSION: u32 = 27;//0xfe;

#[repr(packed)]
#[derive(Meta)]
pub struct ArrayMeta {
    pub version: u32,
    pub records : u32
}

impl ArrayMeta {
    pub fn from_buffer<'refs>(data: &'refs [u8], offset : usize) -> &'refs ArrayMeta {
        unsafe { & *((data[offset..]).as_ptr() as *const ArrayMeta) }
    }
    pub fn from_buffer_mut(data: &mut [u8], offset : usize) -> &mut ArrayMeta {
        unsafe { &mut *((data[offset..]).as_ptr() as *mut ArrayMeta) }
    }
    pub fn from_account_buffer_mut<'refs,'info>(account: &'refs AccountInfo<'info>, offset : usize) -> &'info mut ArrayMeta {
        let data = account.data.borrow_mut();
        unsafe { &mut *((data[offset..]).as_ptr() as *mut ArrayMeta) }
    }
}

#[derive(Debug)]
pub struct Array<'info, 'refs, T> 
where T : Copy + 'info
{
    pub account : &'refs AccountInfo<'info>,
    pub segment : Rc<Segment<'info, 'refs>>,
    phantom: PhantomData<&'refs T>,
    // TODO: realloc_on_remove : bool,
}

impl<'info, 'refs, T> Array<'info, 'refs, T> 
where T: Copy + 'info
{

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Array<'info, 'refs, T>> {
        let store = Self::try_load_from_segment(segment)?;

        store.try_init_meta()?;

        Ok(store)
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Array<'info, 'refs, T>> {

        if segment.get_data_len() < mem::size_of::<ArrayMeta>() {
            return Err(ErrorCode::MappedArraySegmentSizeTooSmall.into());
        }

        let store = Array {
            account : segment.store.account,
            segment : segment.clone(),
            phantom : PhantomData,
            // TODO: realloc_on_remove: false,
        };

        Ok(store)
    }

    pub fn data_len_min() -> usize {
        std::mem::size_of::<ArrayMeta>()
    }

    #[inline(always)]
    pub fn get_meta(&self) -> &'info mut ArrayMeta {
        ArrayMeta::from_account_buffer_mut(
            self.account, 
            self.get_offset()
        )
    }

    // pub fn try_as_linear_store_slice<'slice>(
    //     account : &'slice AccountInfo,
    pub fn try_as_linear_store_slice(
        account : &'refs AccountInfo<'info>,
        byte_offset : usize,
    ) -> Result<&'info [T]> {
        let elements = {
            let data = account.data.borrow_mut();
            let meta = ArrayMeta::from_buffer(&data, byte_offset);
            meta.records as usize
        };
        let data_offset = byte_offset + mem::size_of::<ArrayMeta>();
        let slice = utils::account_buffer_as_slice_mut(account,data_offset,elements);
        Ok(slice)
    }

    // pub fn try_as_linear_store_slice_mut<'slice>(
    pub fn try_as_linear_store_slice_mut(
        account : &'refs AccountInfo<'info>, 
        byte_offset : usize
    ) -> Result<&'info mut [T]> {
        let elements = {
            let data = account.data.borrow_mut();
            let meta = ArrayMeta::from_buffer(&data, byte_offset);
            meta.records as usize
        };
        let data_offset = byte_offset + mem::size_of::<ArrayMeta>();
        let slice = utils::account_buffer_as_slice_mut(account,data_offset,elements);
        Ok(slice)
    }

    pub fn try_init_meta(&self) -> Result<&'info mut ArrayMeta> {
        // let offset = self.get_offset();
        let meta = self.get_meta();
        if meta.version != 0u32 {
            return Err(ErrorCode::MappedArrayMetaNotBlank.into());
        }
        meta.version = MAPPED_ARRAY_VERSION;
        Ok(meta)
    }

    pub fn try_init_meta_with_templates(&self, records: usize)
        -> Result<&mut ArrayMeta>
    {
        #[cfg(feature = "check-buffer-sizes")]
        if self.segment.get_data_len() < Array::<T>::calculate_data_len(records) {
            return Err(ErrorCode::MappedArraySegmentSizeTooSmall.into());
        }

        let meta = self.try_init_meta()?;
        meta.records = records as  u32;
        Ok(meta)
    }

    pub fn try_init_meta_with_records(&self, records : &[T])
        -> Result<&'info mut ArrayMeta> where T : 'info + Copy
    {
        #[cfg(feature = "check-buffer-sizes")]
        if self.segment.get_data_len() < Array::<T>::calculate_data_len(records.len()) {
            return Err(ErrorCode::MappedArraySegmentSizeTooSmall.into());
        }

        let meta = self.try_init_meta()?;
        meta.records = records.len() as  u32;

        if records.len() != 0 {
            let elements = self.as_slice_mut();
            #[cfg(test)]
            assert_eq!(records.len(),elements.len());
            for idx in 0..records.len() {
                elements[idx] = records[idx];
            }
        }

        Ok(meta)
    }

    pub fn try_init_meta_with_refs(&self, records : &[&T]) 
        -> Result<&mut ArrayMeta> where T : 'info + Copy
    {
        #[cfg(feature = "check-buffer-sizes")]
        if self.segment.get_data_len() < Array::<T>::calculate_data_len(records.len()) {
            return Err(ErrorCode::MappedArraySegmentSizeTooSmall.into());
        }

        let meta = self.try_init_meta()?;
        meta.records = records.len() as  u32;

        if records.len() != 0 {
            let elements = self.as_slice_mut();
            #[cfg(test)]
            assert_eq!(records.len(),elements.len());
            for idx in 0..records.len() {
                elements[idx] = *records[idx];
            }
        }

        Ok(meta)
    }

    pub fn get_capacity(&self) -> usize {
        self.segment.get_data_len()
    }

    #[inline(always)]
    pub fn init_len(&mut self, records : usize) {
        self.get_meta().records = records as u32;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.get_meta().records as usize
    }

    pub fn get_offset(&self) -> usize {
        self.segment.get_offset()
    }

    pub fn get_data_offset(&self) -> usize {
        self.get_offset() + mem::size_of::<ArrayMeta>()
    }

    pub fn calculate_data_len(records:usize) -> usize {
        mem::size_of::<ArrayMeta>() + records * mem::size_of::<T>()
    }

    #[inline(always)]
    pub fn try_get_at(&self, idx: usize) -> Result<&'refs T> {
        if idx >= self.len() {
            return Err(ErrorCode::MappedArrayBounds.into());
        }

        let data_offset = self.get_data_offset();
        let data = self.account.data.borrow();
        Ok(unsafe { & *((data[(data_offset + idx*mem::size_of::<T>())..]).as_ptr() as *const T) })
    }

    #[inline(always)]
    pub fn try_get_mut_at(&self, idx: usize) -> Result<&'refs mut T> {
        if idx >= self.len() {
            return Err(ErrorCode::MappedArrayBounds.into());
        }

        let data_offset = self.get_data_offset();
        let data = self.account.data.borrow();
        Ok(unsafe { &mut *((data[(data_offset + idx*mem::size_of::<T>())..]).as_ptr() as *mut T) })
    }

    pub fn as_slice(&self) -> &'info [T] {
        utils::account_buffer_as_slice(self.account,self.get_data_offset(),self.len())
    }

    pub fn as_slice_mut(&self) -> &'info mut [T] {
            utils::account_buffer_as_slice_mut(self.account,self.get_data_offset(),self.len())
    }

    pub fn as_struct_slice<S>(&self) -> &'info [S] {
        utils::account_buffer_as_slice(self.account,self.get_data_offset(),self.len())
    }

    pub fn as_struct_slice_mut<S>(&self) -> &'info mut [S] {
            utils::account_buffer_as_slice_mut(self.account,self.get_data_offset(),self.len())
    }

    #[inline(always)]
    pub fn get_at(&self, idx: usize) -> &'refs T {
        let data_offset = self.get_data_offset();
        let data = self.account.data.borrow();
        unsafe { &mut *((data[(data_offset + idx*mem::size_of::<T>())..]).as_ptr() as *mut T) }
    }

    #[inline(always)]
    pub fn get_at_mut(&self, idx: usize) -> &'refs mut T {
        let data_offset = self.get_data_offset();
        let data = self.account.data.borrow();
        unsafe { &mut *((data[(data_offset + idx*mem::size_of::<T>())..]).as_ptr() as *mut T) }
    }

    pub unsafe fn try_resize_for_items(&self, records: usize, zero_init: bool) -> Result<()> {
        log_trace!("try_resize_for_items records:{}", records);
        let capacity = self.get_capacity();

        // TODO review potential capacity problem - memory is available but segment is not sized correctly

        let new_byte_len = Array::<T>::calculate_data_len(records);
        log_trace!("***########### resize for items -  capacity: {}  new_byte_len: {}", capacity, new_byte_len);
        // panic!("***");
        if new_byte_len > capacity {
            self.segment.try_resize(new_byte_len, zero_init)?;

            #[cfg(test)]
            assert_eq!(new_byte_len, self.get_capacity());
        }

        let meta = self.get_meta();
        meta.records = records as u32;

        Ok(())
    }

    pub unsafe fn try_insert(&self, record : &T) -> Result<()> 
    // where T: 'info
    {
        let dest = self.try_allocate(false)?;
        *dest = *record;
        Ok(())
    }

    pub unsafe fn try_insert_at(&self, idx : usize, record : &T) -> Result<()> {
        let dest = self.try_allocate_at(idx, false)?;
        *dest = *record;
        Ok(())
    }

    // pub unsafe fn try_allocate(&self, zero_init:bool) -> Result<&'refs mut T> {
    // pub unsafe fn try_allocate(&self, zero_init:bool) -> Result<&'refs mut T> 
    pub unsafe fn try_allocate(&self, zero_init:bool) -> Result<&'refs mut T> 
    // where T : 'info
    {
        Ok(self.try_allocate_at(self.len(),zero_init)?)
    }

    pub fn get_byte_offset_at_idx(&self, idx: usize) -> usize {
        mem::size_of::<ArrayMeta>() + idx * mem::size_of::<T>()
    }

    pub unsafe fn try_allocate_at(&self, idx : usize, zero_init:bool) -> Result<&'refs mut T> {
        let records_before = self.len();
        let capacity = self.get_capacity();
        let records_after = records_before+1;
        let new_byte_len = self.get_byte_offset_at_idx(records_after);

        if new_byte_len > capacity {
            log_trace!("[linear store] resizing...  current: {} bytes,  new: {} bytes, delta: {} size_of<T>: {}", 
                capacity,
                new_byte_len,
                new_byte_len-capacity,
                mem::size_of::<T>()
            );
            
            // log_trace!("account before: {:#?}",self.account);
            
            // println!("~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-");
            // let meta = self.get_meta();
            // log_trace!("meta A: {:?}", meta);
            // let test_index = self.segment.store.get_index::<u16>();
            // log_trace!("test index: {:?}", test_index);
            // println!("~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-");

            // log_trace!("volatile_try_insert_at");
            self.segment.try_resize(new_byte_len,zero_init)?;
            // log_trace!("account after: {:#?}",self.account);
            // let meta = self.get_meta();
            // log_trace!("meta B: {:?}", meta);
            // log_trace!("resizing ... A");
            if idx < records_before {
                let segment_offset = self.segment.get_offset();
                // log_trace!("resizing ... B");
                // let data_offset = self.get_data_offset();
                let mut data = self.account.data.borrow_mut();
                let from = segment_offset + self.get_byte_offset_at_idx(idx);
                let to = segment_offset + self.get_byte_offset_at_idx(records_before);
                let dest = from + mem::size_of::<T>();
                // log_trace!("Array<T> resizing where size_of<T> is: {}", mem::size_of::<T>());
                data[..].copy_within(from..to, dest);
            } else {
                // log_trace!("+ + + + + + + + + + + + %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%% T SIZE IS: {}", mem::size_of::<T>());

            }
        } else {
            // log_trace!("resizing ... C");
            log_warning!("[linear store] segment shrinking is not implemented");
            // we do nothing for now
            // todo!("reduction of account during try_insert is not implemented");
        }
        // log_trace!("resizing ... D");

        let meta = self.get_meta();
        // log_trace!("meta: {:#?}", meta);
        // log_trace!("resizing ... E");
        #[cfg(test)]
        {
            // log_trace!("{} resize: new byte len {} capacity {}", style("[linear store]").magenta(), new_byte_len, capacity);
            // assert_eq!(new_byte_len, self.get_capacity());
            assert_eq!(meta.get_version(), MAPPED_ARRAY_VERSION);
        }
        meta.records = records_after as u32;
        self.try_get_mut_at(idx)
    }

    pub unsafe fn try_remove_at(&self, idx: usize, realloc: bool) -> Result<()> {
        if idx >= self.len() {
            return Err(ErrorCode::MappedArrayBounds.into());
        }

        let new_len = {
            let meta = self.get_meta();
            let mut records = meta.records as usize;
            let data_offset = self.get_data_offset();

            if records > 1 && idx+1 < records {

                let dest = data_offset + idx*mem::size_of::<T>();
                let src = dest + mem::size_of::<T>();
                let last = data_offset + records*mem::size_of::<T>();
                let mut data = self.account.data.borrow_mut();
                data[..].copy_within(src..last,dest);
            }

            records -= 1;
            meta.records = records as u32;
            mem::size_of::<ArrayMeta>() + records * mem::size_of::<T>()
        };

        if realloc {
            log_trace!("try_remove_at");
            self.segment.try_resize(new_len,false)
        } else {
            Ok(())
        }
    }


    pub fn try_remove_at_swap_last(&self, idx: usize, realloc: bool, zero_init:bool) -> Result<()> {
        // FIXME: finish try_remove_at_swap_last implementation!
        if idx >= self.len() {
            return Err(ErrorCode::MappedArrayBounds.into());
        }

        let meta = self.get_meta();
        let records = meta.records as usize;
        let _data_offset = self.get_data_offset();
        // todo!("try_remove_at_swap_last");
        if records > 1 && idx+1 < records {
            // FIXME finish implementation
        }

        // let new_len = {
        let meta = self.get_meta();
        let mut records = meta.records as usize;
        let data_offset = self.get_data_offset();

        if records > 1 && idx+1 < records {

            let dest = data_offset + idx*mem::size_of::<T>();
            let src = data_offset + (records-1)*mem::size_of::<T>();
            // let last = data_offset + records*mem::size_of::<T>();
            let mut data = self.account.data.borrow_mut();
            data[..].copy_within(src..(src+mem::size_of::<T>()),dest);
        }

        records -= 1;
        meta.records = records as u32;
        let new_len = mem::size_of::<ArrayMeta>() + records * mem::size_of::<T>();
        // };

        if realloc {
            log_trace!("try remove at swap last");
            self.segment.try_resize(new_len,zero_init)
        } else {

            if zero_init {
                utils::fill_account_buffer_u8(self.account, new_len..new_len+mem::size_of::<T>(),0);
            }

            Ok(())
        }
    }


    pub fn iter(&self) -> MappedArrayIterator<'info, T> {
        MappedArrayIterator {
            offset : self.get_offset(),
            data : self.account.data.clone(),
            idx: 0,
            len : self.len(),
            phantom : PhantomData,
        }
    }
}

impl<'info,'refs,T> Array<'info,'refs,T> where T : Copy + Ord + 'info {
    pub fn binary_search(&self, value: &T) -> std::result::Result<usize,usize> {
        self.as_slice().binary_search(value)
    }


}

impl<'info, 'refs, T> Index<usize> for Array<'info, 'refs, T> 
where T: Copy
{
    type Output = T;
    fn index(&self, idx : usize) -> &Self::Output {
        self.get_at(idx)
    }
}

impl<'info, 'refs, T> IndexMut<usize> for Array<'info, 'refs, T> 
where T: Copy
{
    fn index_mut(&mut self, idx : usize) -> &mut Self::Output {
        self.get_at_mut(idx)
    }
}

pub struct MappedArrayIterator<'info, T> where T : 'info {
    idx: usize,
    len : usize,
    offset : usize,
    data : Rc<RefCell<&'info mut [u8]>>,
    phantom : PhantomData<T>,
}

impl<'info, T> Iterator for MappedArrayIterator<'info, T> {
    type Item = &'info mut T;

    fn next(&mut self) -> Option<Self::Item> {

        if self.idx >= self.len {
            None
        } else {
            let v = Some(self.get_at(self.idx));
            self.idx += 1;
            v
        }
    }
}

impl<'info, 'refs, T> MappedArrayIterator<'info, T> {
    #[inline(always)]
    fn get_at(&self, idx: usize) -> &'refs mut T {
        let data = self.data.borrow();
        let data_offset = self.offset + mem::size_of::<ArrayMeta>();
        unsafe { &mut *((data[(data_offset + idx*mem::size_of::<T>())..]).as_ptr() as *mut T) }
    }
}
