use std::rc::Rc;
use std::marker::PhantomData;
use crate::result::Result;
// use crate::error::*;
// use crate::client::prelude::Segment;
use crate::container::segment::Segment;

pub struct Struct<'info,'refs,T> {
    // pub meta : Rc<RefCell<&'info mut SliceMeta>>,
    pub segment : Rc<Segment<'info,'refs>>,
    phantom : PhantomData<T>,
}

impl<'info,'refs,T> Struct<'info,'refs,T> {

    pub fn data_len_min() -> usize { std::mem::size_of::<T>() }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Struct<'info,'refs,T>> {
        // let meta = Rc::new(segment.try_as_struct_mut_ref::<SliceMeta>()?); 
        Ok(Struct {
            // meta,
            segment,
            phantom : PhantomData
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Struct<'info,'refs,T>> {
        // let meta = Rc::new(segment.try_as_struct_mut_ref::<SliceMeta>()?); 
        Ok(Struct {
            // meta,
            segment,
            phantom : PhantomData
        })
    }

    pub fn try_as_ref(&self) -> Result<&T> where T : 'info {
        self.segment.try_as_struct_ref()
    }

    pub fn try_as_mut_ref(&self) -> Result<&mut T> where T : 'info {
        self.segment.try_as_struct_mut()
    }
    // pub fn get_tree_chain(&self) -> Vec<BPTreeNode {

    // }
}