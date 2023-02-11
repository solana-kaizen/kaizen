//!
//! Segment-based raw UTF-8 String storage
//!

use crate::container::segment::Segment;
use crate::result::Result;
use std::rc::Rc;
use std::string::*;
// use kaizen::prelude::*;

pub struct Utf8String<'info, 'refs> {
    pub segment: Rc<Segment<'info, 'refs>>,
}

impl<'info, 'refs> Utf8String<'info, 'refs> {
    pub fn data_len_min() -> usize {
        0
    }

    pub fn try_create_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Utf8String<'info, 'refs>> {
        Ok(Utf8String { segment })
    }

    pub fn try_load_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Utf8String<'info, 'refs>> {
        Ok(Utf8String { segment })
    }

    /// # Safety
    /// This function can shift the underlying account data layout.
    /// As such, any references to account data following this function
    /// call should be considered to be invalid and reaquired.
    #[inline]
    pub unsafe fn store(&self, text: &str) -> Result<()> {
        let bytes = text.as_bytes();
        self.store_bytes(bytes)?;
        Ok(())
    }

    /// # Safety
    /// This function can resize the underlying account data.
    /// As such, after its use, any references to segments
    /// within the account should be considered invalid.
    #[inline]
    pub unsafe fn store_bytes(&self, bytes: &[u8]) -> Result<()> {
        self.segment.try_resize(bytes.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(bytes);
        Ok(())
    }
}

impl<'info, 'refs> ToString for Utf8String<'info, 'refs> {
    fn to_string(&self) -> String {
        let bytes = self.segment.as_slice::<u8>();
        unsafe { String::from_utf8_unchecked(bytes.to_vec()) }
    }
}
