//!
//!  Segment-based Memory-mapped strongly-typed data (a single struct)
//!
use crate::result::Result;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::container::segment::Segment;

pub struct Struct<'info, 'refs, T> {
    pub segment: Rc<Segment<'info, 'refs>>,
    phantom: PhantomData<T>,
}

impl<'info, 'refs, T> Struct<'info, 'refs, T> {
    pub fn data_len_min() -> usize {
        std::mem::size_of::<T>()
    }

    pub fn try_create_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Struct<'info, 'refs, T>> {
        Ok(Struct {
            segment,
            phantom: PhantomData,
        })
    }

    pub fn try_load_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Struct<'info, 'refs, T>> {
        Ok(Struct {
            segment,
            phantom: PhantomData,
        })
    }

    pub fn try_as_ref(&self) -> Result<&T>
    where
        T: 'info,
    {
        self.segment.try_as_struct_ref()
    }

    pub fn try_as_mut_ref(&self) -> Result<&mut T>
    where
        T: 'info,
    {
        self.segment.try_as_struct_mut()
    }
}
