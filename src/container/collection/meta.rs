use workflow_allocator_macros::Meta;
use crate::result::Result;
use workflow_allocator::prelude::*;

pub trait CollectionMeta {
    fn min_data_len() -> usize;
    fn try_create(&mut self, seed : &[u8], container_type : Option<u32>) -> Result<()>;
    fn try_load(&mut self) -> Result<()>;
    fn get_seed<'data>(&'data self) -> Vec<u8>;
    fn get_len(&self) -> u64;
    fn set_len(&mut self, _len: u64);
    fn get_container_type(&self) -> Option<u32>;
}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct AccountCollectionMeta {
    collection_seed : u64,
    collection_len : u64,
    collection_container_type : u32,
}

impl AccountCollectionMeta {

    fn try_create(&mut self, seed_src : &[u8], container_type : Option<u32>) -> Result<()> {
        // TODO check that len, seed and container_type are blank
        self.set_len(0);
        self.set_collection_container_type(container_type.unwrap_or(0u32));
        // let seed = u64::from_le_bytes(seed_src[0..8].try_into().unwrap());
        let mut seed_dst = [0u8; 8];
        seed_dst.clone_from_slice(&seed_src[0..]);
        let seed = u64::from_be_bytes(seed_dst);
        self.set_collection_seed(seed);
        Ok(())
    }

    fn get_seed<'data>(&'data self) -> Vec<u8> { // &'data [u8] {
        let bytes: [u8; 8] = unsafe { std::mem::transmute(self.get_collection_seed().to_le()) };
        bytes.to_vec()
    }

    fn get_len(&self) -> u64 {
        self.get_collection_len()
    }

    fn set_len(&mut self, len : u64) {
        self.set_collection_len(len);
    }

    fn get_container_type(&self) -> Option<u32> {
        let container_type = self.get_collection_container_type();
        if container_type == 0 {
            None
        } else {
            Some(container_type)
        }
    }

}

pub struct AccountCollectionMetaReference<'info> {
    data : &'info mut AccountCollectionMeta,
    // seed : Vec<u8>,
}

impl<'info> AccountCollectionMetaReference<'info> {

    pub fn new(data : &'info mut AccountCollectionMeta) -> Self {
        Self { data }
    }

    pub fn data_ref<'data>(&'data self) -> &'data AccountCollectionMeta {
        self.data
    }

    pub fn data_mut<'data>(&'data mut self) -> &'data mut AccountCollectionMeta {
        self.data
    }
}


impl<'info> CollectionMeta for AccountCollectionMetaReference<'info> {
    fn try_create(&mut self, seed : &[u8], container_type : Option<u32>) -> Result<()> {
        self.data_mut().try_create(seed,container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        // self.seed = self.data_ref().get_seed();
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<AccountCollectionMeta>()
    }

    fn get_seed<'data>(&'data self) -> Vec<u8> {
        self.data_ref().get_seed()
    }
    
    fn get_len(&self) -> u64 {
        self.data_ref().get_len()
    }
    
    fn set_len(&mut self, len : u64) {
        self.data_mut().set_len(len);
    }
    
    fn get_container_type(&self) -> Option<u32> {
        self.data_ref().get_container_type()
    }

}

pub struct AccountCollectionMetaSegment<'info,'refs> {
    segment : Rc<Segment<'info,'refs>>
}

impl<'info,'refs> AccountCollectionMetaSegment<'info,'refs> {
    pub fn new(segment : Rc<Segment<'info,'refs>>) -> Self {
        Self { segment }
    }

    pub fn data_ref<'data>(&'data self) -> &'data AccountCollectionMeta {
        self.segment.as_struct_ref::<AccountCollectionMeta>()
    }

    pub fn data_mut<'data>(&'data self) -> &'data mut AccountCollectionMeta {
        self.segment.as_struct_mut::<AccountCollectionMeta>()
    }
}

impl<'info,'refs> CollectionMeta for AccountCollectionMetaSegment<'info,'refs> {

    fn try_create(&mut self, seed : &[u8], container_type : Option<u32>) -> Result<()> {
        self.data_mut().try_create(seed,container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<AccountCollectionMeta>()
    }

    fn get_seed<'data>(&'data self) -> Vec<u8> {
        self.data_ref().get_seed()
    }
    
    fn get_len(&self) -> u64 {
        self.data_ref().get_len()
    }
    
    fn set_len(&mut self, len : u64) {
        self.data_mut().set_len(len)
    }
    
    fn get_container_type(&self) -> Option<u32> {
        self.data_ref().get_container_type()
    }

}
