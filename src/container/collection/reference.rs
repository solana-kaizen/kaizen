//!
//! Proxied collection for an arbitrary set of pubkeys based on a segment-defined seed vector.
//! 
//! `Seed Vector -> Proxy Accounts -> Pubkey`
//! 

use cfg_if::cfg_if;
// use crate::address::ProgramAddressData;
use super::meta::*;
use super::proxy::Proxy;
use crate::container::Container;
use crate::result::Result;
use kaizen::error::ErrorCode;
use kaizen::prelude::*;

pub type PdaProxyCollection<'info, 'refs> =
    PdaProxyCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>;
pub type PdaProxyCollectionReference<'info> =
    PdaProxyCollectionInterface<'info, PdaCollectionMetaInterface<'info>>;

pub struct PdaProxyCollectionInterface<'info, M> {
    pub domain: &'info [u8],
    meta: M,
}

impl<'info, M> PdaProxyCollectionInterface<'info, M>
where
    M: CollectionMeta,
{
    fn try_create_impl(
        domain: &'info [u8],
        mut meta: M,
        // seed : &[u8],
        // container_type : Option<u32>
    ) -> Result<Self> {
        meta.try_create()?; //seed,container_type)?;
        Ok(Self { domain, meta })
    }

    fn try_load_impl(domain: &'info [u8], mut meta: M) -> Result<Self> {
        meta.try_load()?;
        Ok(Self { domain, meta })
    }

    pub fn data_len_min() -> usize {
        M::min_data_len()
    }

    pub fn try_create_from_meta(
        data: &'info mut PdaCollectionMeta,
        account_info: &AccountInfo<'info>,
        seed: &'static [u8],
        container_type: Option<u32>,
    ) -> Result<PdaProxyCollectionInterface<'info, PdaCollectionMetaInterface<'info>>> {
        PdaProxyCollectionInterface::<PdaCollectionMetaInterface>::try_create_impl(
            account_info.key.as_ref(),
            PdaCollectionMetaInterface::new(data, seed, container_type),
            // seed,
            // container_type,
        )
    }

    pub fn try_load_from_meta(
        data: &'info mut PdaCollectionMeta,
        account_info: &AccountInfo<'info>,
        seed: &'static [u8],
        container_type: Option<u32>,
    ) -> Result<PdaProxyCollectionInterface<'info, PdaCollectionMetaInterface<'info>>> {
        PdaProxyCollectionInterface::<PdaCollectionMetaInterface>::try_load_impl(
            account_info.key.as_ref(),
            PdaCollectionMetaInterface::new(data, seed, container_type),
        )
    }

    pub fn try_create_from_segment_with_collection_args<'refs>(
        segment: Rc<Segment<'info, 'refs>>,
        seed: &'static [u8],
        container_type: Option<u32>,
    ) -> Result<PdaProxyCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>>
    {
        PdaProxyCollectionInterface::<PdaCollectionSegmentInterface>::try_load_impl(
            segment.account().key.as_ref(),
            PdaCollectionSegmentInterface::new(segment, seed, container_type),
        )
    }

    pub fn try_load_from_segment_with_collection_args<'refs>(
        segment: Rc<Segment<'info, 'refs>>,
        seed: &'static [u8],
        container_type: Option<u32>,
    ) -> Result<PdaProxyCollectionInterface<'info, PdaCollectionSegmentInterface<'info, 'refs>>>
    {
        PdaProxyCollectionInterface::<PdaCollectionSegmentInterface>::try_load_impl(
            segment.account().key.as_ref(),
            PdaCollectionSegmentInterface::new(segment, seed, container_type),
        )
    }

    pub fn try_create(
        &mut self,
        // seed : &[u8],
        // container_type : Option<u32>,
    ) -> Result<()> {
        self.meta.try_create() //seed, container_type)
    }

    pub fn len(&self) -> usize {
        self.meta.get_len() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.meta.get_len() == 0
    }

    pub fn get_proxy_seed_at<'seed>(
        &'seed self,
        idx: &u64,
        suffix: Option<&'seed [u8]>,
    ) -> Vec<&'seed [u8]> {
        // let index_bytes: &[u8; 8] = unsafe { std::mem::transmute(idx as *const u64) };
        let index_bytes = unsafe { &*(idx as *const u64 as *const [u8; 8]) };
        // let index_bytes: &[u8; 8] = unsafe { std::mem::transmute(idx as *const u64) };
        if let Some(suffix) = suffix {
            vec![self.domain, self.meta.get_seed(), index_bytes, suffix]
        } else {
            vec![self.domain, self.meta.get_seed(), index_bytes]
        }
    }

    // pub fn get_proxy_seed_at(&self, idx : &u64, suffix : Option<u8>) -> Vec<&[u8]> {
    //     let index_bytes: &[u8;8] = unsafe { std::mem::transmute(idx as * const u64) };
    //     if let Some(suffix) = suffix {
    //         vec![self.domain, &self.meta.get_seed(), index_bytes, &[suffix]]
    //     } else {
    //         vec![self.domain, &self.meta.get_seed(), index_bytes]
    //     }
    // }

    // pub fn get_proxy_seed_at(&self, idx : u64) -> Vec<u8> {
    //     let domain = self.domain;
    //     let index_bytes: [u8; 8] = unsafe { std::mem::transmute(idx.to_le()) };
    //     [domain, &self.meta.get_seed(),&index_bytes].concat()
    // }

    pub fn try_insert_reference<'refs, T>(
        &mut self,
        ctx: &ContextReference<'info, 'refs, '_, '_>,
        bump: u8,
        container: &T,
    ) -> Result<()>
    where
        T: Container<'info, 'refs>,
    {
        if let Some(container_type) = self.meta.get_container_type() {
            if T::container_type() != container_type {
                return Err(error_code!(ErrorCode::ContainerTypeMismatch));
            }
        }

        let next_index = self.meta.get_len() + 1;
        // let tpl_seeds = self.get_proxy_seed_at(&next_index,Some(&[bump]));
        let bump = &[bump];
        let tpl_seeds = self.get_proxy_seed_at(&next_index, Some(bump));
        // program_address_data_bytes.push(bump);

        // let tpl_program_address_data = ProgramAddressData::from_bytes(program_address_data_bytes.as_slice());
        let pda = Pubkey::create_program_address(
            // &[tpl_program_address_data.seed],
            &tpl_seeds,
            ctx.program_id,
        )?;

        let tpl_account_info = match ctx.locate_index_account(&pda) {
            Some(account_info) => account_info,
            None => return Err(error_code!(ErrorCode::AccountCollectionNotFound)),
        };

        let allocation_args = AccountAllocationArgs::new(AddressDomain::None);
        // let account_info =
        ctx.try_create_pda_with_args(
            Proxy::data_len(),
            &allocation_args,
            &tpl_seeds,
            // tpl_program_address_data,
            tpl_account_info,
            false,
        )?;

        Proxy::try_create(tpl_account_info, container.pubkey())?;
        self.meta.set_len(next_index);

        Ok(())
    }
}

cfg_if! {
    if #[cfg(not(target_os = "solana"))] {

        use futures::{stream::FuturesOrdered, StreamExt};

        impl<'info,M> PdaProxyCollectionInterface<'info,M>
        where M: CollectionMeta
        {

        // impl<'info,'refs> AccountReferenceCollection<'info,'refs> {

            pub fn get_proxy_pda_at(&self, program_id : &Pubkey, idx : u64) -> Result<(Pubkey, u8)> {
                let (address, bump) = Pubkey::find_program_address(
                    &self.get_proxy_seed_at(&idx,None),
                    // &[&self.get_proxy_seed_at(idx)], //domain,&meta.get_seed_as_bytes(),&index_bytes],
                    program_id
                );

                Ok((address, bump))
            }


            pub fn get_proxy_pubkey_at(&self, program_id : &Pubkey, idx : usize) -> Result<Pubkey> {
                Ok(self.get_proxy_pda_at(program_id,idx as u64)?.0)
            }

            pub async fn load_container_at<'this,T>(&self, program_id: &Pubkey, idx: usize)
            -> Result<Option<ContainerReference<'this,T>>>
            where T: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                self.load_container_at_with_transport::<T>(program_id, idx, &transport).await
            }

            pub async fn load_container_at_with_transport<'this,T>(&self, program_id: &Pubkey, idx: usize, transport: &Arc<Transport>)
            -> Result<Option<ContainerReference<'this,T>>>
            where T: kaizen::container::Container<'this,'this>
            {
                let proxy_pubkey = self.get_proxy_pubkey_at(program_id, idx)?;
                let proxy = match load_container_with_transport::<Proxy>(transport, &proxy_pubkey).await? {
                    Some(proxy) => proxy,
                    None => return Err(error_code!(ErrorCode::AccountReferenceCollectionProxyNotFound))
                };

                let container_pubkey = proxy.reference();
                load_container_with_transport::<T>(transport,container_pubkey).await
            }


            pub async fn load_container_range<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>)
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: kaizen::container::Container<'this,'this>
            {
                let transport = Transport::global()?;
                self.load_container_range_with_transport::<T>(program_id, range, &transport).await
            }

            pub async fn load_container_range_with_transport<'this,T>(&self, program_id: &Pubkey, range: std::ops::Range<usize>, transport: &Arc<Transport>)
            -> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
            where T: kaizen::container::Container<'this,'this>
            {
                let mut futures = FuturesOrdered::new();
                for idx in range {
                    let f = self.load_container_at_with_transport::<T>(program_id, idx, transport);
                    futures.push_back(f);
                }

                Ok(futures.collect::<Vec<_>>().await)
            }

        }

    }
}
