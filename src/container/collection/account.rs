// use cfg_if::cfg_if;
// use solana_program::pubkey::Pubkey;
// use workflow_allocator_macros::{Meta, container};
use workflow_allocator_macros::Meta;
// use crate::context::ContextReference;
// use crate::error;
// use crate::error_code;
// use std::rc::Rc;
// use crate::error::ErrorCode;
// use borsh::{BorshDeserialize, BorshSerialize};
use crate::result::Result;
// use crate::container::segment::Segment;
// use crate::identity::*;
use workflow_allocator::prelude::*;
use workflow_allocator::error::ErrorCode;
// use workflow_allocator::container::Containers;
// use workflow_allocator::container::keys::Ts;

// use super::TsPubkey;
// use super::Container;


#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct AccountCollectionMeta {
    count : u64,
    data_type : u32,
}

// impl CollectionMeta {
//     pub fn init(&mut self, pubkey : &Pubkey, data_type : u32) {
//         self.set_pubkey(*pubkey);
//         self.set_data_type(data_type);
//         self.set_count(0);
//     }
// }

pub struct AccountCollection<'info,'refs> 
{
    pub external_meta : Option<&'info mut AccountCollectionMeta>,
    pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
}


impl<'info,'refs> AccountCollection<'info,'refs> 
{
    pub fn meta<'meta>(&'meta self) -> Result<&'meta AccountCollectionMeta> {
        if let Some(external_meta) = &self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_ref::<AccountCollectionMeta>())
        } else {
            Err(ErrorCode::AccountCollectionMissingMeta.into())
        }
    }

    pub fn meta_mut<'meta>(&'meta mut self) -> Result<&'meta mut AccountCollectionMeta> {
        if let Some(external_meta) = &mut self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_mut::<AccountCollectionMeta>())
        } else {
            Err(ErrorCode::AccountCollectionMissingMeta.into())
        }
    }

    pub fn data_len_min() -> usize { std::mem::size_of::<AccountCollectionMeta>() }

    pub fn try_from_meta(meta : &'info mut AccountCollectionMeta) -> Result<Self> {
        Ok(AccountCollection {
            segment_meta : None,
            external_meta : Some(meta),
        })
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<AccountCollection<'info,'refs>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(AccountCollection {
            segment_meta : Some(segment),
            external_meta : None,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<AccountCollection<'info,'refs>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(AccountCollection {
            segment_meta : Some(segment),
            external_meta : None,
        })
    }

    // pub fn try_create(&mut self, _ctx: &ContextReference, data_type : u32) -> Result<()> {
    pub fn try_create(&mut self, data_type : u32) -> Result<()> {
        // let data_type = self.meta().get_data_type();
        let meta = self.meta_mut()?;
        meta.set_count(0);
        meta.set_data_type(data_type);

        Ok(())
        // Ok(collection_store)
    }
/* 
    // pub fn try_load<'ctx>(&mut self, ctx:&'ctx ContextReference<'info,'refs,'_,'_>) -> Result<()> {
    pub fn try_load(&mut self, ctx: &ContextReference<'info,'refs,'_,'_>, suffix : &str, bump : u8) -> Result<()> {

        let meta = self.meta()?;

        // ^ TODO: GET PDA ADDRESS

        // use account pubkey as seed start...

        let pda = Pubkey::create_program_address(
            &[suffix.as_bytes()], // , tpl_adderss_data.seed],
            ctx.program_id
        )?;

        if let Some(account_info) = ctx.locate_index_account(&meta.pubkey) {
            // let container = CollectionStore::<'info,'refs,T>::try_load(account_info)?;
            let container = CollectionStore::<T>::try_load(account_info)?;
            self.container = Some(container);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::CollectionNotFound))
        }
    }

    // // pub fn try_insert<'t>(&mut self, record: &'t T) -> Result<()> {
    pub fn try_insert(&mut self, record: &T) -> Result<()> {
        if let Some(container) = &self.container {
            container.try_insert(record)?;
            let meta = self.meta_mut()?;
            let count = meta.get_count();
            meta.set_count(count + 1);
            Ok(())
        } else {
            Err(error_code!(ErrorCode::CollectionNotLoaded))
        }
    }

    */
}

// ~~~

// cfg_if! {
//     if #[cfg(not(target_arch = "bpf"))] {
//         use async_trait::async_trait;
//         use workflow_allocator::container::AccountAggregator;
//         use solana_program::instruction::AccountMeta;

//         #[async_trait(?Send)]
//         impl<'info,'refs,T> AccountAggregator for Collection<'info,'refs,T> 
//         where T : Copy + Eq + PartialEq + Ord + 'info
//         {
//             type Key = T;
//             async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
//                 if key.is_some() {
//                     return Err(error_code!(ErrorCode::NotImplemented));
//                 }
//                 let meta = self.meta()?;
//                 Ok(vec![AccountMeta::new(meta.get_pubkey(), false)])
//             }

//             async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
//                 if key.is_some() {
//                     return Err(error_code!(ErrorCode::NotImplemented));
//                 }
//                 let meta = self.meta()?;
//                 Ok(vec![AccountMeta::new_readonly(meta.get_pubkey(), false)])
//             }
        
//         }
//     }
// }
