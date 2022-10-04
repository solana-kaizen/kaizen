use cfg_if::cfg_if;
// use solana_program::rent::Rent;
// use solana_program::pubkey::Pubkey;
// use workflow_allocator_macros::{Meta, container};
use workflow_allocator_macros::Meta;
use crate::address::ProgramAddressData;
use crate::container::Container;
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
use workflow_allocator::container;
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
    pub account : &'refs AccountInfo<'info>,
    pub external_meta : Option<&'info mut AccountCollectionMeta>,
    pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
}


impl<'info,'refs> AccountCollection<'info,'refs> 
{
    pub fn account(&self) -> &'refs AccountInfo<'info> {
        self.account
    }

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

    pub fn try_from_meta(
        meta : &'info mut AccountCollectionMeta,
        account_info : &'refs AccountInfo<'info>,
    ) -> Result<Self> {
        Ok(AccountCollection {
            account: account_info,
            segment_meta : None,
            external_meta : Some(meta),
        })
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<AccountCollection<'info,'refs>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(AccountCollection {
            account : segment.account(),
            segment_meta : Some(segment),
            external_meta : None,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<AccountCollection<'info,'refs>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(AccountCollection {
            account : segment.account(),
            segment_meta : Some(segment),
            external_meta : None,
        })
    }

    // pub fn try_create(&mut self, _ctx: &ContextReference, data_type : u32) -> Result<()> {
    pub fn try_init(&mut self, data_type : u32) -> Result<()> {
        // let data_type = self.meta().get_data_type();
        let meta = self.meta_mut()?;
        meta.set_count(0);
        meta.set_data_type(data_type);

        Ok(())
        // Ok(collection_store)
    }

    pub fn try_load<T>(&self, ctx: &ContextReference<'info,'refs,'_,'_>, suffix : &str, index: u64, bump_seed : u8) 
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {
        let meta = self.meta()?;
        assert!(index < meta.get_count());
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(index.to_le()) };

        let pda = Pubkey::create_program_address(
            &[suffix.as_bytes(),&index_bytes,&[bump_seed]],
            ctx.program_id
        )?;

        if let Some(account_info) = ctx.locate_index_account(&pda) {
            let container = T::try_load(account_info)?;
            Ok(container)
        } else {
            Err(error_code!(ErrorCode::AccountCollectionNotFound))
        }
    }

    // pub fn try_create_and_insert<T>(
    pub fn try_create_pda<T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        suffix : &str,
        bump_seed : u8,
        // tpl_program_address_data : ProgramAddressData,

        allocation_args : &AccountAllocationArgs<'info,'refs>,
        data_len : Option<usize>,
    )
    -> Result<<T as Container<'info,'refs>>::T>
    where T : Container<'info,'refs>
    {

        let user_seed = self.account().key.as_ref();

        let meta = self.meta_mut()?;
        let next_index = meta.get_count() + 1;
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(next_index.to_le()) };

        let program_address_data_bytes : Vec<u8> = [suffix.as_bytes(),&index_bytes,&[bump_seed]].concat();
        let tpl_program_address_data = ProgramAddressData::from_bytes(program_address_data_bytes.as_slice());

        let pda = Pubkey::create_program_address(
            &[tpl_program_address_data.seed],
            //&[suffix.as_bytes(),&index_bytes,&[bump_seed]],
            ctx.program_id
        )?;

        let tpl_account_info = match ctx.locate_index_account(&pda) {
            Some(account_info) => account_info,
            None => {
                return Err(error_code!(ErrorCode::AccountCollectionNotFound))
            }
        };

        let data_len = match data_len {
            Some(data_len) => data_len,
            None => T::initial_data_len()
        };

        let account_info = ctx.try_create_pda_with_args(
            data_len,
            allocation_args,
            user_seed,
            tpl_program_address_data,
            tpl_account_info,
            false
        )?;

        meta.set_count(next_index);

        let container = T::try_create(account_info)?;
        Ok(container)



    }

    pub fn try_insert_pda<T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        suffix : &str,
        bump_seed : u8,
        container: &T
    )
    -> Result<()>
    where T : Container<'info,'refs>
    {
        let user_seed = self.account().key.as_ref();

        let meta = self.meta_mut()?;
        let next_index = meta.get_count() + 1;
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(next_index.to_le()) };

        let pda = Pubkey::create_program_address(
            &[user_seed,suffix.as_bytes(),&index_bytes,&[bump_seed]],
            ctx.program_id
        )?;

        if container.pubkey() != &pda {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidAddress));
        }

        let account = match ctx.locate_index_account(&pda) {
            Some(account_info) => account_info,
            None => {
                return Err(error_code!(ErrorCode::AccountCollectionNotFound))
            }
        };

        if account.data_len() < std::mem::size_of::<u32>() {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidAccount))
        }

        if T::container_type() != container::try_get_container_type(account)? {
            return Err(error_code!(ErrorCode::AccountCollectionInvalidContainerType))
        }

        meta.set_count(next_index);

        Ok(())
    }

    
}

cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {
        impl<'info,'refs> AccountCollection<'info,'refs> 
        {

            pub fn try_create(&self, program_id : &Pubkey, suffix : &str) -> Result<(Pubkey, u8)> {
                self.try_create_with_offset(program_id, suffix, 0)
            }

            pub fn try_create_with_offset(&self, program_id : &Pubkey, suffix : &str, index_offset : u64) -> Result<(Pubkey, u8)> {

                let meta = self.meta()?;
                let next_index = meta.get_count()+1+index_offset;
                let index_bytes: [u8; 8] = unsafe { std::mem::transmute(next_index.to_le()) };
        
                let (address, bump_seed) = Pubkey::find_program_address(
                    &[suffix.as_bytes(),&index_bytes],
                    program_id
                );
            
                Ok((address, bump_seed))
            }

        }
    }
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
