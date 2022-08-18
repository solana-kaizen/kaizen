#[cfg(not(target_arch = "bpf"))]
pub mod registry {
    use std::sync::Arc;
    use std::sync::RwLock;
    // use serde_json::map::Entry;
    use workflow_allocator::prelude::*;
    use workflow_allocator::result::Result;
    use workflow_log::log_trace;
    // use std::collections::BTreeMap;
    use ahash::AHashMap;
    use derivative::Derivative;
    use wasm_bindgen::prelude::*;

    #[derive(Derivative)]
    #[derivative(Clone, Debug)]
    pub struct EntrypointDeclaration {
        pub program_id : Pubkey,
        pub name : &'static str,
        #[derivative(Debug="ignore")]
        // pub entrypoint_fn : Arc<&'static ProcessInstruction>,
        pub entrypoint_fn : ProcessInstruction,
    }
    
    impl EntrypointDeclaration {
        pub const fn new(
            program_id : Pubkey,
            name: &'static str,
            entrypoint_fn: ProcessInstruction,
        ) -> Self {
            EntrypointDeclaration {
                program_id,
                name,
                entrypoint_fn
            }
        }
    }

    impl std::fmt::Display for EntrypointDeclaration {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{:>20} {}", self.program_id.to_string(),self.name)
        }    
    }

    #[cfg(not(target_arch = "wasm32"))]
    inventory::collect!(EntrypointDeclaration);

    pub type EntrypointDeclarationRegistry = Arc<RwLock<AHashMap<Pubkey,EntrypointDeclaration>>>;

    static mut ENTRYPOINT_REGISTRY : Option<EntrypointDeclarationRegistry> = None;

    pub fn global() -> EntrypointDeclarationRegistry {
        let registry = unsafe { (&ENTRYPOINT_REGISTRY).as_ref()};
        match registry {
            Some(registry) => registry.clone(),
            None => {
                let registry = Arc::new(RwLock::new(AHashMap::new()));
                unsafe { ENTRYPOINT_REGISTRY = Some(registry.clone()); }
                registry
            }
        }
    }

    pub fn lookup(program_id: &Pubkey) -> Result<Option<EntrypointDeclaration>> {
        // let registry = global()?;
        // let registry = registry.read()?;
        Ok(global().read()?.get(program_id).cloned())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn init() -> Result<()> {
        let registry = global();
        let mut map = registry.write()?;
        if map.len() != 0 {
            panic!("entrypoint type registry is already initialized");
        }

        for entrypoint_declaration in inventory::iter::<crate::program::registry::EntrypointDeclaration> {
            // log_trace!("[program] registering: {} {}", 
            //     entrypoint_declaration.program_id.to_string(), 
            //     entrypoint_declaration.name
            // );
            if let Some(previous_declaration) = map.insert(entrypoint_declaration.program_id, entrypoint_declaration.clone()) {
                panic!("duplicate entrypoint declaration for program {} - {}:\n{:#?}\n~vs~\n{:#?}", 
                    entrypoint_declaration.program_id,
                    entrypoint_declaration.name,
                    entrypoint_declaration,
                    previous_declaration
                );
            }
        }

        Ok(())
    }

    pub fn register_entrypoint_declaration(entrypoint_declaration: EntrypointDeclaration) -> Result<()> {
        // log_trace!("[program] registering: {} {}", 
        //         entrypoint_declaration.program_id.to_string(), 
        //         entrypoint_declaration.name
        // );
        if let Some(_previous_declaration) = global().write()?.insert(entrypoint_declaration.program_id, entrypoint_declaration.clone()) {
            panic!("duplicate entrypoint declaration for program {}:\n{:#?}\n~vs~\n{:#?}", entrypoint_declaration.program_id, entrypoint_declaration,_previous_declaration);
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn list_entrypoints() -> Result<()> {
        let registry = global();
        let map = registry.read()?;
        for (_,entrypoint) in map.iter() {
            log_trace!("[program] {}", entrypoint);
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub mod wasm {

        use super::*;
        use js_sys::Array;
        // use wasm_bindgen::prelude::*;
        // use workflow_allocator::trace;

        // #[wasm_bindgen]
        pub fn load_program_registry(pkg: &JsValue) -> Result<()> {

            // let registry = unsafe { (&ENTRYPOINT_REGISTRY).as_ref()};
            // if registry.is_none() {
            //     let mut registry : EntrypointDeclarationRegistry = BTreeMap::new();
            //     unsafe { ENTRYPOINT_REGISTRY = Some(registry); }                    
            // }    
// log_trace!("init");
            let mut fn_names = Vec::new();
            let keys = js_sys::Reflect::own_keys(&pkg)?;
            let keys_vec = keys.to_vec();
            for idx in 0..keys_vec.len() {
                let name: String = keys_vec[idx].as_string().unwrap_or("".into());
                if name.starts_with("entrypoint_declaration_register_") {
                    // crate::log_trace!("program::init() - found one: {}", name);
                    fn_names.push(keys_vec[idx].clone());
                }
            }

            if fn_names.len() == 0 {
                panic!("workflow_allocator::entrypoint::registry::with_entrypoints(): no registered entrypoints found!");
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

            Ok(()) //JsValue::from(self.clone()))
        }

    }


}

