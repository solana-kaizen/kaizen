use std::rc::Rc;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::result::Result;
use crate::container::segment::Segment;

pub struct SerializedT<'info,'refs, T>
where T: BorshSerialize + BorshDeserialize
{
    pub segment : Rc<Segment<'info,'refs>>,
    _t_ : std::marker::PhantomData<T>,
}

impl<'info,'refs, T> SerializedT<'info,'refs, T>
where T: BorshSerialize + BorshDeserialize
{

    pub fn data_len_min() -> usize { 0 }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<SerializedT<'info,'refs,T>> {
        Ok(SerializedT {
            segment,
            _t_ : std::marker::PhantomData,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<SerializedT<'info,'refs,T>> {
        Ok(SerializedT {
            segment,
            _t_ : std::marker::PhantomData,
        })
    }

    pub fn load(&self) -> Result<T> {
        let mut src = self.segment.as_slice::<u8>();
        let v = BorshDeserialize::deserialize(&mut src)?;
        Ok(v)
    }

    pub fn store_volatile(&self, v : &T) -> Result<()> {
        let vec = v.try_to_vec()?;
        self.segment.try_resize(vec.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(&vec);
        Ok(())
    }

}


pub struct Serialized<'info,'refs> {
    pub segment : Rc<Segment<'info,'refs>>,
}

impl<'info,'refs> Serialized<'info,'refs> {

    pub fn data_len_min() -> usize { 0 }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Serialized<'info,'refs>> {
        Ok(Serialized {
            segment,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Serialized<'info,'refs>> {
        Ok(Serialized {
            segment,
        })
    }

    pub fn load<T>(&self) -> Result<T> where T : BorshDeserialize + BorshSerialize {
        let mut src = self.segment.as_slice::<u8>();
        let v = BorshDeserialize::deserialize(&mut src)?;
        Ok(v)
    }

    pub fn store_volatile<T>(&self, v : &T) -> Result<()> where T : BorshDeserialize + BorshSerialize {
        let vec = v.try_to_vec()?;
        self.segment.try_resize(vec.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(&vec);
        Ok(())
    }

}