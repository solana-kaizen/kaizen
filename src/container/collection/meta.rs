//!
//! CollectionMeta traits used by collection interfaces.
//! 
use crate::result::Result;
use kaizen::prelude::*;
use kaizen_macros::Meta;
use std::cmp::Ordering;

pub trait CollectionMeta {
    fn min_data_len() -> usize;
    fn try_create(&mut self) -> Result<()>; // }, seed : &[u8], container_type : Option<u32>) -> Result<()>;
    fn try_load(&mut self) -> Result<()>;
    fn get_seed(&self) -> &[u8]; //Vec<u8>;
    fn get_len(&self) -> u64;
    fn set_len(&mut self, _len: u64);
    fn get_container_type(&self) -> Option<u32>;
}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PdaCollectionMeta {
    // collection_seed : u64,
    collection_len: u64,
    // collection_container_type : u32,
}

impl PdaCollectionMeta {
    fn try_create(&mut self) -> Result<()> {
        // }, seed_src : &[u8], container_type : Option<u32>) -> Result<()> {
        // TODO check that len, seed and container_type are blank
        self.set_len(0);
        // self.set_collection_container_type(container_type.unwrap_or(0u32));
        // let seed = u64::from_le_bytes(seed_src[0..8].try_into().unwrap());
        // let mut seed_dst = [0u8; 8];
        // seed_dst.clone_from_slice(&seed_src[0..]);
        // let seed = u64::from_be_bytes(seed_dst);
        // self.set_collection_seed(seed);
        Ok(())
    }

    // fn get_seed<'data>(&'data self) -> Vec<u8> { // &'data [u8] {
    //     let bytes: [u8; 8] = unsafe { std::mem::transmute(self.get_collection_seed().to_le()) };
    //     bytes.to_vec()
    // }

    fn get_len(&self) -> u64 {
        self.get_collection_len()
    }

    fn set_len(&mut self, len: u64) {
        self.set_collection_len(len);
    }

    // fn get_container_type(&self) -> Option<u32> {
    //     let container_type = self.get_collection_container_type();
    //     if container_type == 0 {
    //         None
    //     } else {
    //         Some(container_type)
    //     }
    // }
}

pub struct PdaCollectionMetaInterface<'info> {
    data: &'info mut PdaCollectionMeta,
    seed: &'static [u8],
    container_type: Option<u32>,
    // seed : Vec<u8>,
}

impl<'info> PdaCollectionMetaInterface<'info> {
    pub fn new(
        data: &'info mut PdaCollectionMeta,
        seed: &'static [u8],
        container_type: Option<u32>,
    ) -> Self {
        Self {
            data,
            seed,
            container_type,
        }
    }

    pub fn data_ref(&self) -> &PdaCollectionMeta {
        self.data
    }

    pub fn data_mut(&mut self) -> &mut PdaCollectionMeta {
        self.data
    }
}

impl<'info> CollectionMeta for PdaCollectionMetaInterface<'info> {
    fn try_create(&mut self) -> Result<()> {
        // }, seed : &[u8], container_type : Option<u32>) -> Result<()> {
        self.data_mut().try_create() //seed,container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        // self.seed = self.data_ref().get_seed();
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<PdaCollectionMeta>()
    }

    fn get_seed(&self) -> &[u8] {
        //Vec<u8> {
        //self.data_ref().get_seed()
        self.seed
    }

    fn get_len(&self) -> u64 {
        self.data_ref().get_len()
    }

    fn set_len(&mut self, len: u64) {
        self.data_mut().set_len(len);
    }

    fn get_container_type(&self) -> Option<u32> {
        self.container_type
        //        self.data_ref().get_container_type()
    }
}

#[derive(Debug)]
pub struct PdaCollectionSegmentInterface<'info, 'refs> {
    segment: Rc<Segment<'info, 'refs>>,
    seed: &'static [u8],
    container_type: Option<u32>,
}

impl<'info, 'refs> PdaCollectionSegmentInterface<'info, 'refs> {
    pub fn new(
        segment: Rc<Segment<'info, 'refs>>,
        seed: &'static [u8],
        container_type: Option<u32>,
    ) -> Self {
        Self {
            segment,
            seed,
            container_type,
        }
    }

    pub fn data_ref(&self) -> &PdaCollectionMeta {
        self.segment.as_struct_ref::<PdaCollectionMeta>()
    }

    pub fn data_mut(&self) -> &mut PdaCollectionMeta {
        self.segment.as_struct_mut::<PdaCollectionMeta>()
    }
}

impl<'info, 'refs> CollectionMeta for PdaCollectionSegmentInterface<'info, 'refs> {
    fn try_create(&mut self) -> Result<()> {
        // }, seed : &[u8], container_type : Option<u32>) -> Result<()> {
        self.data_mut().try_create() //seed,container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<PdaCollectionMeta>()
    }

    fn get_seed(&self) -> &[u8] {
        //Vec<u8> {
        // self.data_ref().get_seed()
        self.seed
    }

    fn get_len(&self) -> u64 {
        self.data_ref().get_len()
    }

    fn set_len(&mut self, len: u64) {
        self.data_mut().set_len(len)
    }

    fn get_container_type(&self) -> Option<u32> {
        // self.data_ref().get_container_type()
        self.container_type
    }
}

// ~~~

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyCollectionMeta {
    pubkey: Pubkey,
    collection_len: u64,
    sequence: u64,
    data_type: u32,
    container_type: u32,
}

impl PubkeyCollectionMeta {
    pub fn try_create(
        &mut self,
        pubkey: &Pubkey,
        data_type: Option<u32>,
        container_type: Option<u32>,
    ) -> Result<()> {
        self.pubkey = *pubkey;
        self.set_data_type(data_type.unwrap_or(0));
        self.set_container_type(container_type.unwrap_or(0));
        Ok(())
    }

    pub fn advance_sequence(&mut self) -> u32 {
        let seq = self.get_sequence() + 1;
        self.set_sequence(seq);
        seq as u32
    }
}

pub trait PubkeyCollectionMetaTrait {
    fn try_create(
        &mut self,
        pubkey: &Pubkey,
        data_type: Option<u32>,
        container_type: Option<u32>,
    ) -> Result<()>;
    fn try_load(&mut self) -> Result<()>;
    fn min_data_len() -> usize;
    fn pubkey(&self) -> &Pubkey;
    fn get_len(&self) -> u64;
    fn set_len(&mut self, count: u64);
    fn advance_sequence(&mut self) -> u32;
    fn get_data_type(&self) -> Option<u32>;
    fn get_container_type(&self) -> Option<u32>;
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct PubkeyCollectionMetaInterface<'data> {
    data: &'data mut PubkeyCollectionMeta,
}

impl<'data> PubkeyCollectionMetaInterface<'data> {
    pub fn new(data: &'data mut PubkeyCollectionMeta) -> Self {
        Self { data }
    }

    pub fn data_ref(&self) -> &PubkeyCollectionMeta {
        self.data
    }

    pub fn data_mut(&mut self) -> &mut PubkeyCollectionMeta {
        self.data
    }
}

impl<'info> PubkeyCollectionMetaTrait for PubkeyCollectionMetaInterface<'info> {
    fn try_create(
        &mut self,
        pubkey: &Pubkey,
        data_type: Option<u32>,
        container_type: Option<u32>,
    ) -> Result<()> {
        self.data_mut()
            .try_create(pubkey, data_type, container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<PubkeyCollectionMeta>()
    }

    fn pubkey(&self) -> &Pubkey {
        &self.data_ref().pubkey
    }

    fn get_len(&self) -> u64 {
        self.data_ref().get_collection_len()
    }

    fn set_len(&mut self, len: u64) {
        self.data_mut().set_collection_len(len);
    }

    fn advance_sequence(&mut self) -> u32 {
        self.data_mut().advance_sequence()
    }

    fn get_data_type(&self) -> Option<u32> {
        let data_type = self.data_ref().get_data_type();
        if data_type == 0 {
            None
        } else {
            Some(data_type)
        }
    }

    fn get_container_type(&self) -> Option<u32> {
        let container_type = self.data_ref().get_container_type();
        if container_type == 0 {
            None
        } else {
            Some(container_type)
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub struct PubkeyCollectionSegmentInterface<'info, 'refs> {
    segment: Rc<Segment<'info, 'refs>>,
}

impl<'info, 'refs> PubkeyCollectionSegmentInterface<'info, 'refs> {
    pub fn new(segment: Rc<Segment<'info, 'refs>>) -> Self {
        Self { segment }
    }

    pub fn data_ref(&self) -> &PubkeyCollectionMeta {
        self.segment.as_struct_ref::<PubkeyCollectionMeta>()
    }

    pub fn data_mut(&self) -> &mut PubkeyCollectionMeta {
        self.segment.as_struct_mut::<PubkeyCollectionMeta>()
    }
}

impl<'info, 'refs> PubkeyCollectionMetaTrait for PubkeyCollectionSegmentInterface<'info, 'refs> {
    fn try_create(
        &mut self,
        pubkey: &Pubkey,
        data_type: Option<u32>,
        container_type: Option<u32>,
    ) -> Result<()> {
        self.data_mut()
            .try_create(pubkey, data_type, container_type)
    }

    fn try_load(&mut self) -> Result<()> {
        Ok(())
    }

    fn min_data_len() -> usize {
        std::mem::size_of::<PubkeyCollectionMeta>()
    }

    fn pubkey(&self) -> &Pubkey {
        &self.data_ref().pubkey
    }

    fn get_len(&self) -> u64 {
        self.data_ref().get_collection_len()
    }

    fn set_len(&mut self, len: u64) {
        self.data_mut().set_collection_len(len)
    }

    fn advance_sequence(&mut self) -> u32 {
        self.data_mut().advance_sequence()
    }

    fn get_data_type(&self) -> Option<u32> {
        let data_type = self.data_ref().get_data_type();
        if data_type == 0 {
            None
        } else {
            Some(data_type)
        }
    }

    fn get_container_type(&self) -> Option<u32> {
        let container_type = self.data_ref().get_container_type();
        if container_type == 0 {
            None
        } else {
            Some(container_type)
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyMeta {
    seq: u32,
    pub key: Pubkey,
}

impl PubkeyMeta {
    pub fn new(seq: u32, key: Pubkey) -> Self {
        PubkeyMeta { seq, key }
    }
}

impl Ord for PubkeyMeta {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.seq, &self.key).cmp(&(other.seq, &other.key))
    }
}

impl PartialOrd for PubkeyMeta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PubkeyMeta {
    fn eq(&self, other: &Self) -> bool {
        (self.seq, &self.key) == (other.seq, &other.key)
    }
}

impl Eq for PubkeyMeta {}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeySequence {
    seq: u32,
    pub key: Pubkey,
}

impl PubkeySequence {
    pub fn new(seq: u32, key: Pubkey) -> Self {
        PubkeySequence { seq, key }
    }
}

impl Ord for PubkeySequence {
    fn cmp(&self, other: &Self) -> Ordering {
        // (self.seq, &self.key).cmp(&(other.seq, &other.key))
        self.get_seq().cmp(&other.get_seq())
    }
}

impl PartialOrd for PubkeySequence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PubkeySequence {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq
    }
}

impl Eq for PubkeySequence {}

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct PubkeyReference {
    seq: u32,
    pub key: Pubkey,
}

impl PubkeyReference {
    pub fn new(seq: u32, key: Pubkey) -> Self {
        PubkeyReference { seq, key }
    }
}

impl Ord for PubkeyReference {
    fn cmp(&self, other: &Self) -> Ordering {
        // (self.seq, &self.key).cmp(&(other.seq, &other.key))
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for PubkeyReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PubkeyReference {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for PubkeyReference {}
