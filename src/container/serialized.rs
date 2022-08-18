use std::rc::Rc;
use borsh::{BorshSchema, BorshDeserialize, BorshSerialize};
use crate::result::Result;
use crate::container::segment::Segment;

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

    pub fn load<T>(&self) -> Result<T> where T : BorshDeserialize + BorshSchema + BorshSerialize {
        let mut src = self.segment.as_slice::<u8>();
        let v = BorshDeserialize::deserialize(&mut src)?;
        Ok(v)
    }

    pub fn store_volatile<T>(&self, v : &T) -> Result<()> where T : BorshDeserialize + BorshSchema + BorshSerialize {
        let vec = v.try_to_vec()?;
        self.segment.try_resize(vec.len(), false)?;
        self.segment.as_slice_mut().copy_from_slice(&vec);
        Ok(())
    }

}