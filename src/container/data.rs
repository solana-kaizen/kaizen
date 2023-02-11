//!
//! Segment-base Memory-mapped variable-type data
//!

use crate::container::segment::Segment;
use crate::result::Result;
use std::rc::Rc;

#[derive(Debug)]
pub struct Data<'info, 'refs> {
    pub segment: Rc<Segment<'info, 'refs>>,
}

impl<'info, 'refs> Data<'info, 'refs> {
    pub fn data_len_min() -> usize {
        0
    }

    pub fn try_create_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Data<'info, 'refs>> {
        Ok(Data { segment })
    }

    pub fn try_load_from_segment(segment: Rc<Segment<'info, 'refs>>) -> Result<Data<'info, 'refs>> {
        Ok(Data { segment })
    }

    pub fn as_slice<T>(&self) -> &[T]
    where
        T: 'info,
    {
        self.segment.as_slice()
    }

    pub fn as_slice_mut<T>(&self) -> &mut [T]
    where
        T: 'info,
    {
        self.segment.as_slice_mut()
    }
}

impl<'info, 'refs> AsRef<[u8]> for Data<'info, 'refs> {
    fn as_ref(&self) -> &[u8] {
        self.segment.as_ref_u8()
    }
}

impl<'info, 'refs> AsMut<[u8]> for Data<'info, 'refs> {
    fn as_mut(&mut self) -> &mut [u8] {
        (*self.segment).as_ref_mut_u8()
    }
}
