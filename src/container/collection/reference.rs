use cfg_if::cfg_if;
use workflow_allocator_macros::Meta;
use crate::address::ProgramAddressData;
use crate::container::Container;
use crate::result::Result;
use workflow_allocator::prelude::*;
use workflow_allocator::error::ErrorCode;
use super::proxy::Proxy;

#[derive(Meta, Copy, Clone)]
#[repr(packed)]
pub struct AccountReferenceCollectionMeta {
    seed : u64,
    len : u64,
    container_type : u32,
}

impl AccountReferenceCollectionMeta {
    pub fn get_seed_as_bytes(&self) -> [u8;8] {
        unsafe { std::mem::transmute(self.get_seed().to_le()) }
    }
}

pub struct AccountReferenceCollection<'info,'refs> 
{
    pub account : &'refs AccountInfo<'info>,
    pub external_meta : Option<&'info mut AccountReferenceCollectionMeta>,
    pub segment_meta : Option<Rc<Segment<'info,'refs>>>,
}


impl<'info,'refs> AccountReferenceCollection<'info,'refs> 
{
    pub fn len(&self) -> usize {
        self.meta().unwrap().get_len() as usize
    }

    pub fn account(&self) -> &'refs AccountInfo<'info> {
        self.account
    }

    pub fn meta<'meta>(&'meta self) -> Result<&'meta AccountReferenceCollectionMeta> {
        if let Some(external_meta) = &self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_ref::<AccountReferenceCollectionMeta>())
        } else {
            Err(ErrorCode::AccountReferenceCollectionMissingMeta.into())
        }
    }

    pub fn meta_mut<'meta>(&'meta mut self) -> Result<&'meta mut AccountReferenceCollectionMeta> {
        if let Some(external_meta) = &mut self.external_meta {
            return Ok(external_meta);
        } else if let Some(segment) = &self.segment_meta {
            Ok(segment.as_struct_mut::<AccountReferenceCollectionMeta>())
        } else {
            Err(ErrorCode::AccountReferenceCollectionMissingMeta.into())
        }
    }

    pub fn data_len_min() -> usize { std::mem::size_of::<AccountReferenceCollectionMeta>() }

    pub fn try_from_meta(
        meta : &'info mut AccountReferenceCollectionMeta,
        account_info : &'refs AccountInfo<'info>,
    ) -> Result<Self> {
        Ok(AccountReferenceCollection {
            account: account_info,
            segment_meta : None,
            external_meta : Some(meta),
        })
    }

    pub fn try_create_from_segment(
        segment : Rc<Segment<'info, 'refs>>
    ) -> Result<AccountReferenceCollection<'info,'refs>> {
        // let meta = segment.as_struct_mut_ref::<CollectionMeta>();
        Ok(AccountReferenceCollection {
            account : segment.account(),
            segment_meta : Some(segment),
            external_meta : None,
        })
    }

    pub fn try_load_from_segment(
            segment : Rc<Segment<'info, 'refs>>
    ) -> Result<AccountReferenceCollection<'info,'refs>> {
        Ok(AccountReferenceCollection {
            account : segment.account(),
            segment_meta : Some(segment),
            external_meta : None,
        })
    }

    pub fn try_init(&mut self, container_type : u32, seed : u64) -> Result<()> {
        let meta = self.meta_mut()?;
        meta.set_len(0);
        meta.set_container_type(container_type);
        meta.set_seed(seed);

        Ok(())
    }

    pub fn get_proxy_seed_at(&self, meta: &AccountReferenceCollectionMeta, idx : u64) -> Vec<u8> {
        let domain = self.account().key.as_ref();
        let index_bytes: [u8; 8] = unsafe { std::mem::transmute(idx.to_le()) };
        [domain, &meta.get_seed_as_bytes(),&index_bytes].concat()
    }

    // pub fn try_load<T>(
    //     &self,
    //     ctx: &ContextReference<'info,'refs,'_,'_>,
    //     // suffix : &str,
    //     index: u64,
    //     seed_bump : u8
    // ) 
    // -> Result<<T as Container<'info,'refs>>::T>
    // where T : Container<'info,'refs>
    // {
    //     let meta = self.meta()?;
    //     assert!(index < meta.get_len());
    //     // let index_bytes: [u8; 8] = unsafe { std::mem::transmute(index.to_le()) };

    //     let mut program_address_data_bytes = self.get_proxy_seed_at(meta,index);
    //     program_address_data_bytes.push(seed_bump);


    //     let pda = Pubkey::create_program_address(
    //         &[&program_address_data_bytes], //suffix.as_bytes(),&index_bytes,&[seed_bump]],
    //         ctx.program_id
    //     )?;

    //     if let Some(account_info) = ctx.locate_index_account(&pda) {
    //         let proxy = Proxy::try_load(account_info)?;
    //         Ok(container)
    //     } else {
    //         Err(error_code!(ErrorCode::AccountCollectionNotFound))
    //     }
    // }

    // pub fn get_pubkey_seed_at(&self, idx : usize) {

    //     let user_seed = self.account().key.as_ref();
    //     let index_bytes: [u8; 8] = unsafe { std::mem::transmute((idx as u64).to_le()) };
    //     let program_address_data_bytes : Vec<u8> = [user_seed,suffix.as_bytes(),&index_bytes,&[bump_seed]].concat();

    // }

    pub fn try_insert_reference<T>(
        &mut self,
        ctx: &ContextReference<'info,'refs,'_,'_>,
        seed_bump : u8,
        // allocation_args : &AccountAllocationArgs<'info,'refs,'_>,
        container: &T
    )
    -> Result<()>
    where T : Container<'info,'refs>
    {
        // let user_seed = self.account().key.as_ref();

        let meta = self.meta()?;
        if T::container_type() != meta.get_container_type() {
            return Err(error_code!(ErrorCode::ContainerTypeMismatch));
        }
        let next_index = meta.get_len() + 1;

        let mut program_address_data_bytes = self.get_proxy_seed_at(meta,next_index);
        program_address_data_bytes.push(seed_bump);


        // let index_bytes: [u8; 8] = unsafe { std::mem::transmute(next_index.to_le()) };
        // let program_address_data_bytes : Vec<u8> = [
        //     &meta.seed_as_bytes().as_slice(),
        //     index_bytes.as_slice(),
        //     &[seed_bump]
        // ].concat();
        let tpl_program_address_data = ProgramAddressData::from_bytes(program_address_data_bytes.as_slice());

        let pda = Pubkey::create_program_address(
            &[tpl_program_address_data.seed],
            // &[user_seed,suffix.as_bytes(),&index_bytes,&[bump_seed]],
            ctx.program_id
        )?;

        let tpl_account_info = match ctx.locate_index_account(&pda) {
            Some(account_info) => account_info,
            None => {
                return Err(error_code!(ErrorCode::AccountCollectionNotFound))
            }
        };

        let allocation_args = AccountAllocationArgs::new(AddressDomain::None);
        let account_info = ctx.try_create_pda_with_args(
            Proxy::data_len(),
            &allocation_args,
            // user_seed,
            tpl_program_address_data,
            tpl_account_info,
            false
        )?;

        let _proxy = Proxy::try_create(account_info, container.pubkey())?;

        self.meta_mut()?.set_len(next_index);

        Ok(())
    }
    

}


cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {
        
        // use futures::join;
        // use futures::{stream::FuturesUnordered, StreamExt};
        use futures::{stream::FuturesOrdered, StreamExt};

        impl<'info,'refs> AccountReferenceCollection<'info,'refs> {

            pub fn get_proxy_pda_at(&self, program_id : &Pubkey, idx : u64) -> Result<(Pubkey, u8)> {
                let (address, bump) = Pubkey::find_program_address(
                    &[&self.get_proxy_seed_at(self.meta()?, idx)], //domain,&meta.get_seed_as_bytes(),&index_bytes],
                    program_id
                );

                Ok((address, bump))
            }


            pub fn get_proxy_pubkey_at(&self, program_id : &Pubkey, idx : usize) -> Result<Pubkey> {
                Ok(self.get_proxy_pda_at(program_id,idx as u64)?.0)
                // let meta = self.meta()?;
                // let user_seed = self.account().key.as_ref();
                // let index_bytes: [u8; 8] = unsafe { std::mem::transmute((idx as u64).to_le()) };
                // let (address, _bump_seed) = Pubkey::find_program_address(
                //     &[user_seed,&meta.seed_as_bytes(),&index_bytes],
                //     program_id
                // );

                // Ok(address)
            }
            
            // pub fn get_proxy_pubkey_seed_at(&self, program_id : &Pubkey, idx : usize) -> Result<Vec<u8>> {
    
            //     let meta = self.meta()?;

            //     let user_seed = self.account().key.as_ref();
            //     let index_bytes: [u8; 8] = unsafe { std::mem::transmute((idx as u64).to_le()) };
            //     let mut pda_seed : Vec<u8> = [meta.seed_as_bytes().as_slice(),&index_bytes].concat();

            //     let (_address, bump_seed) = Pubkey::find_program_address(
            //         &[user_seed,pda_seed.as_slice()],
            //         program_id
            //     );

            //     pda_seed.push(bump_seed);
            //     Ok(pda_seed)
            // }
            
            pub async fn load_container_at<'this,T>(&self, program_id: &Pubkey, idx: usize) 
            -> Result<Option<ContainerReference<'this,T>>> 
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                Ok(self.load_container_at_with_transport::<T>(program_id, idx, &transport).await?)
            }

            pub async fn load_container_at_with_transport<'this,T>(&self, program_id: &Pubkey, idx: usize, transport: &Arc<Transport>) 
            -> Result<Option<ContainerReference<'this,T>>> 
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let proxy_pubkey = self.get_proxy_pubkey_at(program_id, idx)?;
                let proxy = match load_container_with_transport::<Proxy>(&transport, &proxy_pubkey).await? {
                    Some(proxy) => proxy,
                    None => return Err(error_code!(ErrorCode::AccountReferenceCollectionProxyNotFound))
                };

                let container_pubkey = proxy.reference();
                Ok(load_container_with_transport::<T>(&transport,container_pubkey).await?)
            }
            
    
            pub async fn load_container_range<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>) 
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                Ok(self.load_container_range_with_transport::<T>(program_id, range, &transport).await?)
            }
    
            pub async fn load_container_range_with_transport<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>, transport: &Arc<Transport>) 
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: workflow_allocator::container::Container<'this,'this>
            {
                let mut futures = FuturesOrdered::new();
                for idx in range {
                    let f = self.load_container_at_with_transport::<T>(program_id, idx, &transport);
                    futures.push_back(f);
                }

                Ok(futures.collect::<Vec<_>>().await)
            }

        }            

        // #[async_trait(?Send)]
        // impl<'info,'refs,T> AccountAggregator for Collection<'info,'refs,T> 
        // where T : Copy + Eq + PartialEq + Ord + 'info
        // {
        //     type Key = T;
        //     async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
        //         if key.is_some() {
        //             return Err(error_code!(ErrorCode::NotImplemented));
        //         }
        //         let meta = self.meta()?;
        //         Ok(vec![AccountMeta::new(meta.get_pubkey(), false)])
        //     }

        //     async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>> {
        //         if key.is_some() {
        //             return Err(error_code!(ErrorCode::NotImplemented));
        //         }
        //         let meta = self.meta()?;
        //         Ok(vec![AccountMeta::new_readonly(meta.get_pubkey(), false)])
        //     }
        
        // }
    }
}
