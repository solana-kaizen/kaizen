use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;
use workflow_allocator::container::Container;
use std::cell::UnsafeCell;
use std::sync::MutexGuard;
use owning_ref::OwningHandle;

pub async fn load_container_clone<'info:'refs,'refs:'info,T> (pubkey : &Pubkey) 
-> Result<
    Option<
        OwningHandle<
            OwningHandle<Box<UnsafeCell<AccountData>>,Box<AccountInfo<'info>>>,
            Box<<T as Container<'info,'refs>>::T>
        >
    >
> 
where T: workflow_allocator::container::Container<'info,'refs>
{
    let transport = Transport::global()?;
    let account_data = match transport.lookup(pubkey).await? {
        Some(account_data) => account_data,
        None => return Ok(None)
    }.clone_for_storage()?;

    let cell = UnsafeCell::new(account_data);
    let account_data_account_info = 
        OwningHandle::<Box<UnsafeCell<AccountData>>,Box<AccountInfo>>::new_with_fn(Box::new(cell), |x| {
            Box::new( unsafe { 
                let r = x.as_ref().unwrap();
                let m = r.get().as_mut().unwrap();
                m.into_account_info() 
            })
        });

    let container = 
    OwningHandle::<
        OwningHandle::<Box<UnsafeCell<AccountData>>,Box<AccountInfo<'info>>>,
        Box<<T as Container<'info,'refs>>::T>
    >::new_with_fn(account_data_account_info, |x| {
        Box::new( unsafe { 
            let account_info : &'refs AccountInfo<'info> = x.as_ref().unwrap();
            let t = T::try_load(account_info).unwrap(); // ^ TODO
            t
        })
    });

    Ok(Some(container))
}


pub async fn load_container<'lock:'info+'refs, 'info:'refs,'refs:'info,T> (pubkey : &Pubkey) 
-> 
Result<
    Option<
        OwningHandle<
            OwningHandle<
                OwningHandle::<
                    OwningHandle::<
                        Arc<AccountDataReference>,
                        Box<UnsafeCell<MutexGuard<'lock, AccountData>>>>, 
                    Box<UnsafeCell<&'refs mut AccountData>>>,
                Box<AccountInfo<'info>>>, 
            Box<<T as Container<'info,'refs>>::T>
        >
    >
> 
where T: workflow_allocator::container::Container<'info,'refs>
{
    let transport = Transport::global()?;
    let account_data_reference = match transport.lookup(pubkey).await? {
        Some(account_data_reference) => account_data_reference,
        None => return Ok(None)
    };

    let account_data_ref_account_data_lock = 
        OwningHandle::<Arc<AccountDataReference>,Box<UnsafeCell<MutexGuard<'lock, AccountData>>>>::new_with_fn(account_data_reference, |x| {
            Box::new( unsafe { 
                let r = x.as_ref().unwrap();
                UnsafeCell::new(r.account_data.lock().unwrap())
            })
        });

    let account_data_lock_ref = 
        OwningHandle::<
            OwningHandle::<
                Arc<AccountDataReference>,
                Box<UnsafeCell<MutexGuard<'lock,AccountData>>>>
        
        ,Box<UnsafeCell<&mut AccountData>>>::new_with_fn(account_data_ref_account_data_lock, |x| {
            Box::new( unsafe { 
                let r = x.as_ref().unwrap();
                let m = r.get().as_mut().unwrap();
                UnsafeCell::new(&mut *m)
            })
        });

    let account_data_account_info = 
        OwningHandle::<
            OwningHandle::<
                    OwningHandle::<Arc<AccountDataReference>,Box<UnsafeCell<MutexGuard<'lock, AccountData>>>>
            ,Box<UnsafeCell<&mut AccountData>>>
        ,Box<AccountInfo>>::new_with_fn(account_data_lock_ref, |x| {
            Box::new( unsafe { 
                let r = x.as_ref().unwrap();
                let m = (*r).get().as_mut().unwrap();
                m.into_account_info() 
            })
        });

    let container = 
    OwningHandle::<
        OwningHandle::<
            OwningHandle::<
                OwningHandle::<Arc<AccountDataReference>,Box<UnsafeCell<MutexGuard<'lock, AccountData>>>>,
                Box<UnsafeCell<&'refs mut AccountData>>>,
            Box<AccountInfo<'info>>
        >,
        Box<<T as Container<'info,'refs>>::T>
    >::new_with_fn(account_data_account_info, |x| {
        Box::new( unsafe { 
            let account_info : &'refs AccountInfo<'info> = x.as_ref().unwrap();
            let t = T::try_load(account_info).unwrap(); // ^ TODO
            t
        })
    });

    Ok(Some(container))
}
