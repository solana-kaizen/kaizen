//!
//! Segment-based Borsh-serialized store (for pre-defined or arbitrary data types)
//!

use crate::container::segment::Segment;
use crate::result::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use std::rc::Rc;

pub struct Serialized<'info, 'refs, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub segment: Rc<Segment<'info, 'refs>>,
    _t_: std::marker::PhantomData<T>,
}

impl<'info, 'refs, T> Serialized<'info, 'refs, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub fn data_len_min() -> usize {
        0
    }

    pub fn try_create_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Serialized<'info, 'refs, T>> {
        Ok(Serialized {
            segment,
            _t_: std::marker::PhantomData,
        })
    }

    pub fn try_load_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<Serialized<'info, 'refs, T>> {
        Ok(Serialized {
            segment,
            _t_: std::marker::PhantomData,
        })
    }

    #[inline]
    pub fn load(&self) -> Result<Option<Box<T>>> {
        let mut src = self.segment.as_slice::<u8>();
        if src.is_empty() {
            return Ok(None);
        }
        let v = BorshDeserialize::deserialize(&mut src)?;
        Ok(Some(Box::new(v)))
    }

    #[inline]
    pub fn load_or_default<D>(&self) -> Result<Box<D>>
    where
        D: Default + BorshDeserialize,
    {
        let mut src = self.segment.as_slice::<u8>();
        if src.is_empty() {
            Ok(Box::default())
        } else {
            Ok(Box::new(BorshDeserialize::deserialize(&mut src)?))
        }
    }

    #[inline]
    pub fn store(&self, v: &T) -> Result<()> {
        self.store_bytes(&v.try_to_vec()?)?;
        Ok(())
    }

    #[inline]
    pub fn store_bytes(&self, vec: &[u8]) -> Result<()> {
        self.segment.try_resize(vec.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(vec);
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.segment.get_data_len() == 0
    }
}

pub struct SerializedVariant<'info, 'refs> {
    pub segment: Rc<Segment<'info, 'refs>>,
}

impl<'info, 'refs> SerializedVariant<'info, 'refs> {
    pub fn data_len_min() -> usize {
        0
    }

    pub fn try_create_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<SerializedVariant<'info, 'refs>> {
        Ok(SerializedVariant { segment })
    }

    pub fn try_load_from_segment(
        segment: Rc<Segment<'info, 'refs>>,
    ) -> Result<SerializedVariant<'info, 'refs>> {
        Ok(SerializedVariant { segment })
    }

    pub fn load<T>(&self) -> Result<T>
    where
        T: BorshDeserialize + BorshSerialize,
    {
        let mut src = self.segment.as_slice::<u8>();
        let v = BorshDeserialize::deserialize(&mut src)?;
        Ok(v)
    }

    pub fn store<T>(&self, v: &T) -> Result<()>
    where
        T: BorshDeserialize + BorshSerialize,
    {
        let vec = v.try_to_vec()?;
        self.segment.try_resize(vec.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(&vec);
        Ok(())
    }

    #[inline]
    pub fn store_bytes(&self, vec: &[u8]) -> Result<()> {
        self.segment.try_resize(vec.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(vec);
        Ok(())
    }
}
