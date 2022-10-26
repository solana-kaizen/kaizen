use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
// use workflow_allocator::*;
use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;
use workflow_allocator::error::ErrorCode;
// use solana_program::account_info::AccountInfo;
// use workflow_allocator_macros::Meta;

pub trait Container<'info,'refs> {
    type T;

    fn container_type() -> u32;
    fn initial_data_len() -> usize;
    fn try_allocate(
        ctx: &workflow_allocator::context::ContextReference<'info,'refs,'_,'_>,
        allocation_args : &workflow_allocator::context::AccountAllocationArgs<'info,'refs,'_>,
        reserve_data_len : usize
    ) -> workflow_allocator::result::Result<Self::T>;

    fn try_create(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self::T>; 
    // fn try_create_with_layout(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self::T>; 
    fn try_load(account : &'refs solana_program::account_info::AccountInfo<'info>) -> workflow_allocator::result::Result<Self::T>; 
    fn account(&self) -> &'refs solana_program::account_info::AccountInfo<'info>;
    fn pubkey(&self) -> &solana_program::pubkey::Pubkey;
}

#[derive(Meta)]
#[repr(packed)]
pub struct ContainerHeader {
    pub container_type : u32,
}

#[inline]
pub fn try_get_container_type(account: &AccountInfo) -> Result<u32> {
    let data = account.data.try_borrow_mut()?;
    if data.len() < std::mem::size_of::<ContainerHeader>() {
        return Err(ErrorCode::UnknownContainerType.into())
    }
    let header = unsafe { std::mem::transmute::<_,&ContainerHeader>(
        data.as_ptr()
    )};

    Ok(header.container_type)
}

#[repr(u32)]
pub enum Ranges {
    UnitTests = 0xe0000000,
    Framework = 0xf0000000,
    Distributors = 0xf00e0000,
    Indexes = 0xf00f0000,
}


#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum Containers {
    UnitTests = Ranges::UnitTests as u32,
    TransportTestInterface,
    TransferTestInterface,
    PDATestInterface,
    CollectionTestInterface,

    FrameworkContainers = Ranges::Framework as u32,
    Proxy,
    IdentityProxy,
    Identity,
    OrderedCollection,
    PGPPubkey,

    IndexContainers = Ranges::Indexes as u32,
    BPTreeIndex,
    BPTreeValues,
}

cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {

        use std::sync::Arc;
        use std::cell::UnsafeCell;
        use std::sync::MutexGuard;
        use owning_ref::OwningHandle;
        use workflow_allocator::accounts::*;
        
        pub type ContainerReferenceInner<'this,T> =
        OwningHandle<
            OwningHandle<
                OwningHandle<
                    OwningHandle::<
                        OwningHandle::<
                            Arc<AccountDataReference>,
                            Box<UnsafeCell<MutexGuard<'this, AccountData>>>>, 
                        Box<UnsafeCell<&'this mut AccountData>>>,
                    Box<AccountInfo<'this>>>, 
                Box<UnsafeCell<Option<Result<<T as Container<'this,'this>>::T>>>>>,
            Box<<T as Container<'this,'this>>::T>
        >;
        
        pub struct ContainerReference<'inner,T>
        where T: Container<'inner,'inner>,
        {
            inner : ContainerReferenceInner<'inner,T>
        }

        // ~~~

        unsafe impl<'inner,T> Send for ContainerReference<'inner,T>
        where T: Container<'inner,'inner> {}
        
        unsafe impl<'inner,T> Sync for ContainerReference<'inner,T>
        where T: Container<'inner,'inner> {}

        // ~~~
        
        impl<'inner,T> ContainerReference<'inner,T>
        where T: Container<'inner,'inner>,
        {
            pub fn new(inner : ContainerReferenceInner<'inner,T>) -> Self {
                ContainerReference { inner }
            }
        }



        impl<'inner,T> std::ops::Deref for ContainerReference<'inner,T>
        where T: Container<'inner,'inner>,
        {
            type Target = ContainerReferenceInner<'inner,T>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        // pub type AccountDataContainer<'this,T> = 
        //     OwningHandle<
        //         OwningHandle<
        //             OwningHandle<
        //                 Box<UnsafeCell<AccountData>>,
        //                 Box<AccountInfo<'this>>>,
        //             Box<UnsafeCell<Option<Result<<T as Container<'this,'this>>::T>>>>>,
        //         Box<<T as Container<'this,'this>>::T>
        //     >;

        pub mod registry {
            use super::*;
            use workflow_log::log_trace;
            use std::{sync::{RwLock, Arc}};
            use ahash::AHashMap;
            use derivative::Derivative;
        
            pub type ContainerDebugFn = fn(account_info: &AccountInfo<'_>) -> Result<()>;
        
            #[derive(Derivative)]
            #[derivative(Clone, Debug)]
            // #[derive]
            pub struct ContainerDeclaration {
                pub container_type_id : u32,
                pub name : &'static str,
                // #[derivative(Debug="ignore")]
                // pub debug_fn : Arc<&'static ContainerDebugFn>,
            }
            
            impl ContainerDeclaration {
                pub const fn new(container_type_id: u32, name: &'static str, 
                    // debug_fn: &'static ContainerDebugFn
                ) -> Self {
                    ContainerDeclaration {
                        container_type_id,
                        name,
                        // debug_fn : Arc::new(debug_fn),
                    }
                }
            }
        
            impl std::fmt::Display for ContainerDeclaration {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "0x{:08x} {}", self.container_type_id,self.name)
                }    
            }
        
        
            #[cfg(not(target_arch = "wasm32"))]
            inventory::collect!(ContainerDeclaration);
        
            // pub type ContainerTypeRegistry = BTreeMap<u32,ContainerDeclaration>;
            pub type ContainerTypeRegistry = Arc<RwLock<AHashMap<u32,ContainerDeclaration>>>;
            static mut CONTAINER_TYPE_REGISTRY : Option<ContainerTypeRegistry> = None;
        
            pub fn global() -> ContainerTypeRegistry {
                let registry = unsafe { (&CONTAINER_TYPE_REGISTRY).as_ref()};
                match registry {
                    Some(registry) => registry.clone(),
                    None => {
                        let registry = Arc::new(RwLock::new(AHashMap::new()));
                        unsafe { CONTAINER_TYPE_REGISTRY = Some(registry.clone()); }
                        registry
                    }
                }
            }
        
            pub fn lookup(container_type_id: u32) -> Result<Option<ContainerDeclaration>> {
                // let registry = global();
                // let registry = registry.read()?;
                Ok(global().read()?.get(&container_type_id).cloned())
            }
        
            #[cfg(not(target_arch = "wasm32"))]
            pub fn init() -> Result<()> {
                // println!("initializing container registry...");
                let registry = global();
                let mut map = registry.write()?;
                // let mut map = global().write()?;
                if map.len() != 0 {
                    // println!("existing container registry: {:?}", map);
                    panic!("container registry is already initialized");
                }
        
                for container_declaration in inventory::iter::<workflow_allocator::container::registry::ContainerDeclaration> {
                    // log_trace!("[container] registering 0x{:08x} {}", 
                    //     container_declaration.container_type, 
                    //     container_declaration.name
                    // );
                    if let Some(previous_declaration) = map.insert(container_declaration.container_type_id, container_declaration.clone()) {
                        panic!("duplicate container type registration for type {}:\n{:#?}\n~vs~\n{:#?}", 
                            container_declaration.container_type_id,
                            container_declaration,
                            previous_declaration
                        );
                    }
                }
        
                Ok(())
            }
        
            pub fn register_container_declaration(container_declaration: ContainerDeclaration) ->Result<()> {
                // log_trace!("[container] registering 0x{:08x} {}", 
                //     container_declaration.container_type, 
                //     container_declaration.name
                // );
        
                let registry = global();
                let mut map = registry.write()?;
                // let mut map = global().write()?;
                if let Some(previous_declaration) = map.insert(container_declaration.container_type_id, container_declaration.clone()) {
                    panic!("duplicate container type registration for type {}:\n{:#?}\n~vs~\n{:#?}", container_declaration.container_type_id, container_declaration,previous_declaration);
                }
                Ok(())
            }
        
            #[wasm_bindgen]
            pub fn list_containers() -> Result<()> {
                let registry = global();
                let map = registry.read()?;
                for (_,container) in map.iter() {
                    log_trace!("[container] {}", container);
                }
                Ok(())
            }
        
        
            #[cfg(target_arch = "wasm32")]
            pub mod wasm {
        
                use super::*;
                use js_sys::Array;
                use wasm_bindgen::prelude::*;
                // use workflow_allocator::trace;
        
                #[wasm_bindgen]
                pub fn load_container_registry(pkg: &JsValue) -> Result<()> {
        
                    // let registry = unsafe { (&CONTAINER_TYPE_REGISTRY).as_ref()};
                    // if registry.is_none() {
                    //     let registry : ContainerTypeRegistry = BTreeMap::new();
                    //     unsafe { CONTAINER_TYPE_REGISTRY = Some(registry); }                    
                    // }    
        
                    
        
                    let mut fn_names = Vec::new();
                    let keys = js_sys::Reflect::own_keys(&pkg)?;
                    let keys_vec = keys.to_vec();
                    for idx in 0..keys_vec.len() {
                        let name: String = keys_vec[idx].as_string().unwrap_or("".into());
                        if name.starts_with("container_declaration_register") {
                            // log_trace!("init_bindings() - found one: {}", name);
                            fn_names.push(keys_vec[idx].clone());
                        }
                    }
        
                    if fn_names.len() == 0 {
                        panic!("workflow_allocator::container::registry::with_containers(): no registered containers found!");
                    }
        
                    for fn_name in fn_names.iter() {
                        let fn_jsv = js_sys::Reflect::get(&pkg,fn_name)?;
                        let args = Array::new();
                        let _ret_jsv = js_sys::Reflect::apply(&fn_jsv.into(),&pkg,&args.into())?;
                    }
        
                    // let epfns = keys.filter(|v,idx,arr| {
                    //     true
                    // });
        
                    // let transport = self.inner_mut().ok_or(
                    //     JsValue::from("workflow::Transport - failed to acquire write lock")
                    // )?;//borrow();
                    // let mut entry_points = transport.entry_points.borrow_mut();
                    // router::PIEP.with(|list_ref| {
                    //     let list = list_ref.borrow();
                    //     for (ident,id,piep) in list.iter() {
                    //         log_trace!("binding program {} ▷ {}",id.to_string(),ident);
                    //         entry_points.insert(id.clone(),piep.clone());
                    //     }
                    // });
                    Ok(())
                    // Ok(JsValue::from(true)) //JsValue::from(self.clone()))
                }
        
            }
        
        
        }

    }    
}    
