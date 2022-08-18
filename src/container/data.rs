use std::rc::Rc;
// use std::marker::PhantomRaw;
use crate::result::Result;
// use crate::error::*;
// use crate::client::prelude::Segment;
use crate::container::segment::Segment;

#[derive(Debug)]
pub struct Data<'info,'refs> {
    // pub meta : Rc<RefCell<&'info mut SliceMeta>>,
    pub segment : Rc<Segment<'info,'refs>>,
    // phantom : PhantomRaw<T>,
}

impl<'info,'refs> Data<'info,'refs> {

    pub fn data_len_min() -> usize { 0 }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Data<'info,'refs>> {
        // let meta = Rc::new(segment.try_as_struct_mut_ref::<SliceMeta>()?); 
        Ok(Data {
            // meta,
            segment,
            // phantom : PhantomRaw
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<Data<'info,'refs>> {
        // let meta = Rc::new(segment.try_as_struct_mut_ref::<SliceMeta>()?); 
        Ok(Data {
            // meta,
            segment,
            // phantom : PhantomRaw
        })
    }

    pub fn as_slice<T>(&self) -> &[T] where T : 'info {
        self.segment.as_slice()
    }

    pub fn as_slice_mut<T>(&self) -> &mut [T] where T : 'info {
        self.segment.as_slice_mut()
    }

    // pub fn try_as_ref(&self) -> Result<&'info T> {
    //     self.segment.try_as_struct_ref()
    // }

    // pub fn try_as_mut_ref(&self) -> Result<&'info mut T> {
    //     self.segment.try_as_struct_mut_ref()
    // }
    // pub fn get_tree_chain(&self) -> Vec<BPTreeNode {

    // }
}

impl<'info,'refs> AsRef<[u8]> for Data<'info,'refs> {
    fn as_ref(&self) -> &[u8] {  
        self.segment.as_ref_u8()
    }
}

impl<'info,'refs> AsMut<[u8]> for Data<'info,'refs> {
    fn as_mut(&mut self) -> &mut [u8] {
        (*self.segment).as_ref_mut_u8()
    }
}

