use std::rc::Rc;
use crate::result::Result;
use crate::container::segment::Segment;
use std::string::*;

pub struct Utf8String<'info,'refs> {
    pub segment : Rc<Segment<'info,'refs>>,
}

impl<'info,'refs> Utf8String<'info,'refs> {

    pub fn data_len_min() -> usize { 0 }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Utf8String<'info,'refs>> {
        Ok(Utf8String {
            segment,
        })
    }

    pub fn try_load_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Utf8String<'info,'refs>> {
        Ok(Utf8String {
            segment,
        })
    }

    #[inline]
    pub fn store(&self, text: &str) -> Result<()> {
        let bytes = text.as_bytes();
        self.store_bytes(&bytes)?;
        Ok(())
    }

    #[inline]
    pub fn store_bytes(&self, bytes: &[u8]) -> Result<()> {
        self.segment.try_resize(bytes.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(&bytes);
        Ok(())
    }

}

impl<'info,'refs> ToString for Utf8String<'info,'refs> {
    fn to_string(&self) -> String {
        let bytes = self.segment.as_slice::<u8>();
        unsafe {
            String::from_utf8_unchecked(bytes.to_vec()).to_string()
        }
    }
}
