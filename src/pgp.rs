// use std::{cell::{Ref, RefCell}, rc::Rc, mem};
//use anchor_lang::prelude::*;
//  use crate::error::*;
// use ident::identity::*;
// use crate::containers::Containers;
use workflow_allocator::container::segment::{Segment,SegmentStore};
use workflow_allocator_macros::container;
use workflow_allocator::container::Containers;
// use workflow_allocator::result::Result;

// const IdentityContainer: u32 = 0x49444e54;
// #[repr(u32)]
// enum IdentityContainer {
//     Id = 0xf0000001 //49444e54
// }
// #[container(0x49444e54, u16)]

pub struct PgpPubkey {
    
}

// #[derive(Debug)]
#[container(Containers::PGPPubkey)]
pub struct PGPData<'info,'refs> {
    // pub meta : RefCell<&'info mut IdentityMeta>,
    pub store : SegmentStore<'info,'refs>,
    // ---
    // #[segment(reserve(std::mem::size_of::<u32>()*4))]
    // pub _v1 : MyType<'info,'refs>,
    // #[segment(std::mem::size_of::<IdentityEntry>()*4)]
    #[segment(reserve = 2048)]

    pub pgp_pubkey : Segment<'info,'refs>, 
//    pub pubkey : Data<PgpPubkey>,
    // pub list : LinearStore<'info,'refs, IdentityEntry>,
    // pub _v2: MyType<'info,'refs>,
    // pub _v3: MyType<'info,'refs>,
}


/* 


pub const PGP_MAGIC : u32 = 0x50475000;
pub const PGP_VERSION : u32 = 1;


pub struct PgpPubkeyMeta {
    pub magic : u32,
    pub version : u32,
    pub keysize : u32
}

impl From<Ref<'_,&mut [u8]>> for &mut PgpPubkeyMeta {
    fn from(data: Ref<'_,&mut [u8]>) -> Self {
        unsafe { &mut *(data.as_ptr() as *mut PgpPubkeyMeta) }
    }
}

impl From<&mut [u8]> for &mut PgpPubkeyMeta {
    fn from(data: &mut [u8]) -> Self {
        unsafe { &mut *(data.as_ptr() as *mut PgpPubkeyMeta) }
    }
}

pub struct PgpPubkey<'a> {
    pub meta: &'a mut PgpPubkeyMeta,
    pub data: Rc<RefCell<&'a mut [u8]>>,
}

impl<'a> PgpPubkey<'a> {

    pub fn create(
        data: &Rc<RefCell<&'a mut [u8]>>
    ) -> Result<PgpPubkey<'a>> {

        let meta : &mut PgpPubkeyMeta = data.borrow().into(); //RootMeta::try_from(data)?;

        let mut identity = PgpPubkey {
            meta,
            data : data.clone(),
        };

        identity.init_meta()?;
        Ok(identity)
    }

    pub fn load(data: &Rc<RefCell<&'a mut [u8]>>) -> Result<PgpPubkey<'a>> {
        //let meta = unsafe { &mut *(data.as_ptr() as *mut PgpPubkeyMeta) };
        Ok(PgpPubkey {
            meta : data.borrow().into(),
            data : data.clone(),
        })
    }

    pub fn init_meta(&mut self) -> Result<()> {
        // self.meta ...
        // let _meta : &mut PgpPubkeyMeta = self.data.borrow().into();
        // if meta.magic != 0 {
        //     return Err(ErrorCode::IdentityAccountDataNotBlank.into());
        // }
        // meta.magic = PGPPUBKEY_MAGIC;
        // meta.version = PGPPUBKEY_VERSION;
        // meta.payload_len = mem::size_of::<IdentityMeta>() as u32;
        Ok(())
    }

    pub fn get_pgp_public_key_data(&self) -> Result<&'a mut [u8]> {
        let data = self.data.borrow();
        let v = unsafe { &mut *((data[(mem::size_of::<PgpPubkeyMeta>())..]).as_ptr() as *mut &mut[u8]) };
        Ok(v)

    }
}

*/

#[cfg(not(target_arch = "bpf"))]
pub mod client {
    use pgp::composed::{KeyType, 
        // KeyDetails, SecretKey, SecretSubkey, 
        key::SecretKeyParamsBuilder};
    use pgp::errors::Result;
    // use pgp::packet::{KeyFlags, UserAttribute, UserId};
    use pgp::types::{
        // PublicKeyTrait, 
        SecretKeyTrait, CompressionAlgorithm};
    use pgp::crypto::{sym::SymmetricKeyAlgorithm, hash::HashAlgorithm};
    use smallvec::*;

    pub fn generate_key() -> Result<()> {
        let mut key_params = SecretKeyParamsBuilder::default();

        key_params
            .key_type(KeyType::Rsa(2048))
            .can_create_certificates(false)
            .can_sign(true)
            .primary_user_id("Me <me@example.com>".into())
            .preferred_symmetric_algorithms(smallvec![
                SymmetricKeyAlgorithm::AES256,
            ])
            .preferred_hash_algorithms(smallvec![
                HashAlgorithm::SHA2_256,
            ])
            .preferred_compression_algorithms(smallvec![
                CompressionAlgorithm::ZLIB,
            ]);
        let secret_key_params = key_params.build().expect("Must be able to create secret key params");
        let secret_key = secret_key_params.generate().expect("Failed to generate a plain key.");
        let passwd_fn = || String::new();
        let signed_secret_key = secret_key.sign(passwd_fn).expect("Must be able to sign its own metadata");
        let _public_key = signed_secret_key.public_key();


        Ok(())

    }
}