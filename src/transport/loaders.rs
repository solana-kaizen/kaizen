use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;
use workflow_allocator::container::Container;
use futures::future::join_all;

// pub async fn load_container_clone<'this,T> (pubkey : &Pubkey) 
// -> Result<Option<AccountDataContainer<'this,T>>> 
// where T: workflow_allocator::container::Container<'this,'this>
// {
//     let transport = Transport::global()?;
//     load_container_clone_with_transport::<T>(&transport, pubkey).await
// }

// pub async fn load_container_clone_with_transport<'this,T> (transport: &Arc<Transport>, pubkey : &Pubkey) 
// -> Result<Option<AccountDataContainer<'this,T>>> 
// where T: workflow_allocator::container::Container<'this,'this>
// {
//     match transport.lookup(pubkey).await? {
//         Some(reference) => {
//             let container = reference.try_load_container_clone::<T>()?;
//             Ok(Some(container))
//         },
//         None => return Ok(None)
//     }
// }

pub async fn with_loaded_container<'this, C>(
    pubkey:Pubkey,
    callback:impl Fn(Option<ContainerReference<'this, C>>)->Result<()>
)->Result<()> where C: Container<'this,'this>
{
    if let Some(res) = load_container::<C>(&pubkey).await?{
        callback(Some(res))?;
    }else{
        callback(None)?;
    }

    Ok(())
}

pub async fn with_reloaded_container<'this, C>(
    pubkey:Pubkey,
    callback:impl Fn(Option<ContainerReference<'this, C>>)->Result<()>
)->Result<()> where C: Container<'this,'this>
{
    if let Some(res) = reload_container::<C>(&pubkey).await?{
        callback(Some(res))?;
    }else{
        callback(None)?;
    }

    Ok(())
}

pub async fn load_container<'this,T> (pubkey : &Pubkey) 
-> Result<Option<ContainerReference<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    let transport = Transport::global()?;
    load_container_with_transport::<T>(&transport,pubkey).await
}

pub async fn load_containers<'this,T> (pubkeys : &[Pubkey]) 
-> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
where T: workflow_allocator::container::Container<'this,'this>
{
    let transport = Transport::global()?;

    let mut lookups = Vec::new();
    for pubkey in pubkeys.iter() {
        lookups.push(load_container_with_transport::<T>(&transport,pubkey));
    }

    Ok(join_all(lookups).await)
}

pub async fn load_container_with_transport<'this,T> (transport : &Arc<Transport>, pubkey : &Pubkey) 
-> Result<Option<ContainerReference<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    let account_data_reference = match transport.lookup(pubkey).await? {
        Some(account_data_reference) => account_data_reference,
        None => return Ok(None)
    };

    let container = account_data_reference.try_into_container::<T>()?;
    Ok(Some(container))
}

pub async fn reload_container<'this,T> (pubkey : &Pubkey) 
-> Result<Option<ContainerReference<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    let transport = Transport::global()?;
    reload_container_with_transport::<T>(&transport,pubkey).await
}

pub async fn reload_containers<'this,T> (pubkeys : &[Pubkey]) 
-> Result<Vec<Result<Option<ContainerReference<'this,T>>>>>
where T: workflow_allocator::container::Container<'this,'this>
{
    let transport = Transport::global()?;

    let mut lookups = Vec::new();
    for pubkey in pubkeys.iter() {
        lookups.push(reload_container_with_transport::<T>(&transport,pubkey));
    }

    Ok(join_all(lookups).await)
}

pub async fn reload_container_with_transport<'this,T> (transport : &Arc<Transport>, pubkey : &Pubkey) 
-> Result<Option<ContainerReference<'this,T>>> 
where T: workflow_allocator::container::Container<'this,'this>
{
    log_trace!("... reloading container {}",pubkey);
    transport.purge(Some(pubkey))?;
    let account_data_reference = match transport.lookup(pubkey).await? {
        Some(account_data_reference) => account_data_reference,
        None => return Ok(None)
    };

    let container = account_data_reference.try_into_container::<T>()?;
    Ok(Some(container))
}

// ~

pub async fn load_reference(pubkey : &Pubkey) 
-> Result<Option<Arc<AccountDataReference>>> 
{
    let transport = Transport::global()?;
    load_reference_with_transport(&transport,pubkey).await
}

pub async fn load_references(pubkeys : &[Pubkey]) 
-> Result<Vec<Result<Option<Arc<AccountDataReference>>>>> 
{
    let transport = Transport::global()?;

    let mut lookups = Vec::new();
    for pubkey in pubkeys.iter() {
        lookups.push(load_reference_with_transport(&transport,pubkey));
    }

    Ok(join_all(lookups).await)
}

pub async fn load_reference_with_transport(transport : &Arc<Transport>, pubkey : &Pubkey) 
-> Result<Option<Arc<AccountDataReference>>> 
{
    Ok(transport.lookup(pubkey).await?)
}

pub async fn reload_reference(pubkey : &Pubkey) 
-> Result<Option<Arc<AccountDataReference>>> 
{
    let transport = Transport::global()?;
    transport.purge(Some(pubkey))?;
    load_reference_with_transport(&transport,pubkey).await
}

pub async fn reload_references(pubkeys : &[Pubkey]) 
-> Result<Vec<Result<Option<Arc<AccountDataReference>>>>> 
{
    let transport = Transport::global()?;

    let mut lookups = Vec::new();
    for pubkey in pubkeys.iter() {
        transport.purge(Some(pubkey))?;
        lookups.push(load_reference_with_transport(&transport,pubkey));
    }

    Ok(join_all(lookups).await)
}

pub fn purge_reference(pubkey : &Pubkey) 
-> Result<()> 
{
    let transport = Transport::global()?;
    transport.purge(Some(pubkey))?;
    Ok(())
}

// pub async fn reload_container_clone<'this,T> (pubkey : &Pubkey) 
// -> Result<Option<AccountDataContainer<'this,T>>> 
// where T: workflow_allocator::container::Container<'this,'this>
// {
//     let transport = Transport::global()?;
//     transport.purge(pubkey)?;
//     load_container_clone_with_transport::<T>(&transport,pubkey).await
// }

// pub async fn execute_and_load<'this,T> ((pubkey, instruction) : (Pubkey, Instruction))
// -> Result<Option<ContainerReference<'this,T>>> 
// where T: workflow_allocator::container::Container<'this,'this>
// {
//     let transport = Transport::global()?;
//     transport.execute(&instruction).await?;
//     load_container_with_transport::<T>(&transport,&pubkey).await
// }