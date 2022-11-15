use kaizen::prelude::*;
use kaizen::error::ErrorCode;
use kaizen::result::Result;
use kaizen::container::*;
use kaizen::context::*;

#[derive(Meta)]
pub struct ProxyMeta {
    container_type : u32,
    reference : Pubkey,
}

pub struct Proxy<'info,'refs> {
    account : &'refs AccountInfo<'info>,
    meta : &'info mut ProxyMeta,
}

impl<'info,'refs> Proxy<'info,'refs> {

    pub fn account(&self) -> &'refs AccountInfo<'info> {
        self.account
    }

    pub fn reference(&self) -> &Pubkey {
        &self.meta.reference
    }

    pub fn data_len() -> usize {
        std::mem::size_of::<ProxyMeta>()
    }

    pub fn try_create(
        account: &'refs AccountInfo<'info>,
        reference: &Pubkey,
    ) -> Result<Self> {
        let data = account.data.borrow_mut();
        let meta = unsafe { std::mem::transmute::<_,&mut ProxyMeta>(data.as_ptr()) };
        meta.set_container_type(Containers::Proxy as u32);
        meta.set_reference(*reference);

        let proxy = Proxy {
            account,
            meta
        };

        Ok(proxy)
    }

    pub fn try_load(
        account: &'refs AccountInfo<'info>,
    ) -> Result<Self> {
        let data = account.data.borrow_mut();
        let meta = unsafe { std::mem::transmute::<_,&mut ProxyMeta>(data.as_ptr()) };

        if meta.get_container_type()!= Containers::Proxy as u32 {
            return Err(error_code!(ErrorCode::InvalidProxyContainerType));
        }
        
        let proxy = Proxy {
            account,
            meta
        };

        Ok(proxy)
    }

    // pub fn try_load_reference<T>(&self,ctx: &ContextReference<'info,'refs,'_,'_>) 
    // -> Result<<T as Container<'info,'refs>>::T>
    // where T : Container<'info,'refs>
    // {



    // }
    
}




impl<'info,'refs> Container<'info,'refs> for Proxy<'info,'refs> {
    type T = Self;
    // type T = #struct_name #struct_params;

   // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//    pub 

    fn container_type() -> u32 {
        Containers::Proxy as u32
    }

    fn initial_data_len() -> usize {
        Proxy::data_len()
    }

    fn account(&self) -> &'refs solana_program::account_info::AccountInfo<'info> {
        self.account()
    }

    fn pubkey(&self) -> &solana_program::pubkey::Pubkey {
        self.account().key
    }

    fn try_allocate(
        _ctx: &ContextReference<'info,'refs,'_,'_>,
        _allocation_args : &AccountAllocationArgs<'info,'refs,'_>,
        _reserve_data_len : usize
    ) -> kaizen::result::Result<Proxy<'info,'refs>> {
        // #struct_name :: #struct_params :: try_allocate(ctx, allocation_args, reserve_data_len)

        // let account_info = ctx.try_create_pda(Proxy::data_len(),allocation_args)?;



        unimplemented!()
    }

    fn try_create(_account : &'refs AccountInfo<'info>) -> Result<Proxy<'info,'refs>> {
        unimplemented!()
        //#struct_name :: #struct_params :: try_create(account)
    }
    
    fn try_load(account : &'refs AccountInfo<'info>) -> Result<Proxy<'info,'refs>> {
        Self::try_load(account)
    }
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

}