use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;
use workflow_allocator::container::*;

pub async fn load_container_clone<'this,T> (pubkey : &Pubkey) 
-> Result<Option<AccountDataContainer<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    let transport = Transport::global()?;
    load_container_clone_with_transport::<T>(&transport, pubkey).await
}

pub async fn load_container_clone_with_transport<'this,T> (transport: &Arc<Transport>, pubkey : &Pubkey) 
-> Result<Option<AccountDataContainer<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    match transport.lookup(pubkey).await? {
        Some(reference) => {
            let container = reference.try_load_container_clone::<T>()?;
            Ok(Some(container))
        },
        None => return Ok(None)
    }
}

pub async fn load_container<'this,T> (pubkey : &Pubkey) 
-> Result<Option<ContainerReference<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    let transport = Transport::global()?;
    load_container_with_transport::<T>(&transport,pubkey).await
}

pub async fn load_container_with_transport<'this,T> (transport : &Arc<Transport>, pubkey : &Pubkey) 
-> Result<Option<ContainerReference<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    let account_data_reference = match transport.lookup(pubkey).await? {
        Some(account_data_reference) => account_data_reference,
        None => return Ok(None)
    };

    let container = account_data_reference.try_load_container::<T>()?;
    Ok(Some(container))
}

