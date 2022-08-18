use std::rc::Rc;
use std::sync::Arc;
use ahash::AHashSet;
use async_std::sync::RwLock;
use derivative::Derivative;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::slot_history::AccountInfo;
use solana_program::account_info::IntoAccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::entrypoint::ProcessInstruction;
use workflow_log::*;
use workflow_allocator::realloc::account_info_realloc;
use workflow_allocator::context::SimulationHandlerFn;
use workflow_allocator::utils::generate_random_pubkey;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
// use workflow_allocator::console::style;
use workflow_allocator::address::ProgramAddressData;
use workflow_allocator::context::Context;
use workflow_allocator::accounts::*;
use workflow_allocator::builder::{
    InstructionBuilder,
    InstructionBuilderConfig
};
use workflow_allocator::store::Store;
use crate::accounts::AccountData;
use crate::container::try_get_container_type;




#[derive(Debug)]
pub struct KeyStore {
    pub store: Vec<Pubkey>,
}

impl KeyStore {
    pub fn new(len: usize) -> KeyStore {
        let store = (0..len).map(|_| generate_random_pubkey()).collect();
        KeyStore { store }
    }
}




pub fn allocate_pda<'info, 'refs, 'payer_info, 'payer_refs, 'pid>(
    payer: &'payer_refs AccountInfo<'payer_info>,
    program_id: &'pid Pubkey,
    user_seed: &[u8],
    tpl_adderss_data: &ProgramAddressData,
    tpl_account_info: &'refs AccountInfo<'info>,
    space: usize,
    lamports: u64,
) -> Result<&'refs AccountInfo<'info>> {

    if space > ACCOUNT_DATA_TEMPLATE_SIZE {
        panic!("create_pda() account size is too large (current limit is: {} bytes", ACCOUNT_DATA_TEMPLATE_SIZE);
    }

    // log_trace!("* * * RECEIVING SEED: {:?}", tpl_adderss_data.seed);
    // let seeds = [user_seed, tpl_adderss_data.seed].concat();
    // let seeds_hex = crate::utils::hex(&seeds[..]);
    // log_trace!("* * * program pda seeds:\n{}\n", seeds_hex);

    match Pubkey::create_program_address(
        &[user_seed, tpl_adderss_data.seed],
        &program_id
    ) {
        Ok(address)=>{
            if address != *tpl_account_info.key {
                log_trace!("| pda: PDA ADDRESS MISMATCH {} vs {}", address, tpl_account_info.key);
                return Err(error_code!(ErrorCode::PDAAddressMatch));
            }

            // log_trace!("| pda: PDA ADDRESS OK");
        },
        Err(_e)=>{
            log_trace!("| pda: PDA ADDRESS MATCH failure");
            //TODO handle this pubkey error
            return Err(error_code!(ErrorCode::PDAAddressMatch));
        }
    };

    // ---

    let buffer_size = unsafe {
        let ptr = tpl_account_info
            .try_borrow_mut_data()
            .ok()
            .unwrap()
            .as_mut_ptr()
            .offset(-8) as *mut u64;
        *ptr
    };
    
    log_trace!("| pda: account realloc - buffer: {} slice: {} target: {}",buffer_size,tpl_account_info.data_len(),space);
    account_info_realloc(tpl_account_info, space, true, true)?;
    log_trace!("+ pda: simulator realloc done");
    // log_trace!("| TODO: adjust lamports");
    
    let mut ref_payer_lamports = payer.lamports.borrow_mut();
    let mut payer_lamports = **ref_payer_lamports;

    if payer_lamports < lamports {
        // log_trace!()
        return Err(error_code!(ErrorCode::InsufficientAllocBalance));
    }

    payer_lamports = payer_lamports.saturating_sub(lamports);
    **ref_payer_lamports = payer_lamports;

    let mut ref_tpl_account_info_lamports = tpl_account_info.lamports.borrow_mut();
    **ref_tpl_account_info_lamports = (**ref_tpl_account_info_lamports).saturating_add(lamports);

    Ok(tpl_account_info)
    // list.push(tpl.account_info);
    //? TODO - replace lamports value
    // let mut lamports_ref = tpl.account_info.lamports.borrow();
    // *lamports_ref = lamports;
    // }

    // Ok(list)
}

pub fn allocate_multiple_pda<'info, 'refs, 'payer_info, 'payer_refs, 'pid>(
    _payer: &'payer_refs AccountInfo<'payer_info>,
    _program_id: &'pid Pubkey,
    _user_seed : &[u8],
    account_templates: &[(&ProgramAddressData, &'refs AccountInfo<'info>)],
    // account_templates: &[AccountInfoTemplate<'info, 'refs>],
    settings: &[(usize, u64)],
) -> Result<Vec<&'refs AccountInfo<'info>>> {
    if account_templates.len() < settings.len() {
        log_trace!("======================================================");
        log_trace!(
            "Not enough account templates: {} vs settings: {} ...",
            account_templates.len(),
            settings.len()
        );
        log_trace!("======================================================");
        // return Err(Error::ErrorCode(ErrorCode::NotEnoughAccountTemplates));
        return Err(program_error_code!(ErrorCode::NotEnoughAccountTemplates));
    }

    let mut list = Vec::new();
    for idx in 0..settings.len() {
        let (space, _lamports) = settings[idx];
        let (_tpl_address_data, tpl_account_info) = account_templates[idx];

        {
            let buffer_size = unsafe {
                let ptr = tpl_account_info
                    .try_borrow_mut_data()
                    .ok()
                    .unwrap()
                    .as_mut_ptr()
                    .offset(-8) as *mut u64;
                *ptr
            };
            log_trace!(
                "| pda realloc - buffer: {} slice: {} target: {}",
                buffer_size,
                tpl_account_info.data_len(),
                space
            );
        }

        // if
        log_trace!("{}", style("in allocate_multiple_pda...").white().on_red());
        account_info_realloc(tpl_account_info, space, true, true)?;
        // .is_err() {
        //     return Err(program_error_code!(ErrorCode::ReallocFailure));
        // }

        // log_trace!("| TODO: adjust lamports");
        list.push(tpl_account_info);
        //? TODO - replace lamports value
        // let mut lamports_ref = tpl.account_info.lamports.borrow();
        // *lamports_ref = lamports;
    }

    Ok(list)
}

pub fn transfer_sol<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    _system_program_account: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let mut lamports_src = from.lamports.borrow_mut();
    if **lamports_src < amount {
        return Err(program_error_code!(ErrorCode::InsufficientBalance));
    }

    let mut lamports_dest = to.lamports.borrow_mut();
    **lamports_dest = lamports_dest.saturating_add(amount);
    **lamports_src = lamports_src.saturating_sub(amount);

    // TODO: validate authority authority
    log_trace!(
        "\n--: transfer_sol:\nfrom: {}\n\tto: {}\n\tauthority: {}\n\tamount: {}\n\n",
        from.key,
        to.key,
        authority.key,
        amount
    );

    Ok(())
}

pub fn transfer_spl<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
    _signers: &[&[&[u8]]]
) -> Result<()> {
    log_trace!(
        "\n--: transfer_tokens:\nprogram: {}\n\tfrom: {}\n\tto: {}\n\tauthority: {}\n\tamount: {}\n\n",
        token_program.key,
        from.key,
        to.key,
        authority.key,
        amount
    );

    /*  TODO
        let ctx = CpiContext::new(
            token_program, //self.token_program.to_account_info(),
            Transfer { from, to, authority
                // from: self.sender_token.to_account_info(),
                // to: self.receiver_token.to_account_info(),
                // authority: self.sender.to_account_info(),
            },
        );

        // msg!("starting tokens: {}", ctx.accounts.sender_token.amount);
        token::transfer(ctx, amount)?;
        // ctx.accounts.sender_token.reload()?;
        // msg!("remaining tokens: {}", ctx.accounts.sender_token.amount);
    */

    Ok(())
}




#[derive(Derivative)]
#[derivative(Debug)]
// #[wasm_bindgen]
pub struct Simulator {
    // lamports: u64,
    authority : Pubkey,
    identity: Pubkey,
    program_id: Pubkey,

    // #[wasm_bindgen(skip)]
    #[derivative(Debug="ignore")]
    pub store: Store,
}

// declare_async_rwlock!(Simulator, SimulatorInner);

// #[wasm_bindgen]
impl Simulator {

    // pub fn store(&self) -> Arc<Store> {
    //     self.store
    // }

    // #[wasm_bindgen(constructor)]
    pub fn try_new() -> Result<Arc<Simulator>> {
        let store = Store::new_local()?;
        let lamports = crate::utils::u64sol_to_lamports(500_000_000);

        let authority = generate_random_pubkey();
        let identity = generate_random_pubkey();
        let program_id = generate_random_pubkey();

        {
            // let mut store_inner = store.inner_mut().unwrap();
            // let store = self.store.;

            let authority_account_data = AccountData::new_static_with_size(
                authority.clone(),
                program_id.clone(),
                0
            ).with_lamports(lamports);
            store.try_store(authority_account_data)?;//.await?;
            // let authority_account_data = Arc::new(RwLock::new(authority_account_data));
            //map.write()?.insert(authority.clone(),authority_account_data);
            
            let identity_account_data = AccountData::new_static_with_size(
                identity.clone(),
                program_id.clone(),
                0
            );
            // .with_lamports(lamports);
            store.try_store(identity_account_data)?;//map.write()?.insert(identity.clone(),identity_account_data);
            // let identity_account_data = Arc::new(RwLock::new(identity_account_data));
        }

        let simulator = Arc::new(Simulator {//}::new_with_inner(SimulatorInner {
            store,
            // lamports,
            authority,
            identity,
            program_id,
        });

        Ok(simulator)
    }
}

impl Simulator {


    pub fn with_simulated_identity(self : &Arc<Simulator>) -> Result<()> {
        // let allocation_args = AccountAllocationArgs::default();
        // let identity_account = ctx.create_pda(Identity::initial_data_len(), &allocation_args)?;
        // let mut identity = Identity::try_create(identity_account)?;

        Ok(())
    }

    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    pub fn authority(&self) -> Pubkey {
        self.authority
    }

    pub fn identity(&self) -> Pubkey {
        self.identity
    }

    pub fn new_instruction_builder_config(
        &self,
    ) -> InstructionBuilderConfig {

        // let inner = self.inner().unwrap();
        let config = InstructionBuilderConfig::new(
            self.program_id,
        )
        .with_authority(&self.authority)
        .with_identity(&self.identity);

        config
    }

    pub fn new_instruction_builder(
        &self,
    ) -> InstructionBuilder {

        // let inner = self.inner().unwrap();

        let builder = InstructionBuilder::new(
            self.program_id,
            0,
            0u16,
        )
        .with_authority(&self.authority)
        .with_identity(&self.identity);

        builder
    }

    // pub fn with_lamports(self, lamports: u64) -> Simulator {
    //     {
    //         // let mut simulator = self.inner_mut().unwrap();
    //         self.lamports = lamports;
    //     }
    //     self
    // }

    // pub fn with_sol(self, sol: f64) -> Simulator {
    //     {
    //         let lamports = (sol * LAMPORTS_PER_SOL as f64) as u64;
    //         // let mut simulator = self.inner_mut().unwrap();
    //         self.lamports = lamports;
    //     }
    //     self
    // }

    pub fn store(&self) -> Store {
        self.store.clone()
        // self.inner()
        //     .expect("Simulator::store() read lock fail")
        //     .store
        //     .clone()
    }

    pub async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<AccountDataReference>> {
        Ok(self.store().lookup(pubkey).await?)
    }

    pub async fn execute_entrypoint(
        &self,
        program_id: &Pubkey,
        accounts: &[AccountMeta],
        instruction_data: &[u8],

        entrypoint: ProcessInstruction,
    ) -> Result<()> {
        // let store = self.store();
        let mut account_data = self.program_local_load(program_id, accounts).await?;
        let mut accounts = Vec::new();
        for (pubkey, account_data) in account_data.iter_mut() {
            let is_signer = account_data.is_signer;
            let is_writable = account_data.is_writable;
            let mut account_info = (&*pubkey, account_data).into_account_info();

            // pass signer and writer flags from the source account
            account_info.is_signer = is_signer;
            account_info.is_writable = is_writable;

            accounts.push(account_info);
        }

        log_trace!("▷ entrypoint begin");
        match entrypoint(program_id, &accounts[..], instruction_data) {
            Ok(_) => {}
            Err(e) => return Err(error!("entrypoint error: {:?}", e)),
        }
        log_trace!("◁ entrypoint end");
        self.program_local_store(&accounts).await?;

        Ok(())
    }

    pub async fn execute_handler(
        &self,
        builder: InstructionBuilder,
        handler: SimulationHandlerFn,
    ) -> Result<()> {

        // let store = self.store();
        let ec: Instruction = builder.try_into()?;
        let mut account_data = self.program_local_load(&ec.program_id, &ec.accounts).await?;
        let mut accounts = Vec::new();
        for (pubkey, account_data) in account_data.iter_mut() {
            let is_signer = account_data.is_signer;
            let is_writable = account_data.is_writable;
            let mut account_info = (&*pubkey, account_data).into_account_info();

            // pass signer and writer flags from the source account
            account_info.is_signer = is_signer;
            account_info.is_writable = is_writable;

            accounts.push(account_info);
        }
        let ctx: Context = (
            &ec.program_id,
            &accounts[..],
            ec.data.as_slice(),
        )
            .try_into()
            .expect("Unable to create context");
        match handler(&Rc::new(ctx)) {
            //?;//.map_err(|err| format!("(handler) program error: {:?}", err).to_string())?;
            Ok(_) => {}
            Err(err) => {
                log_trace!("{}", err);
                return Err(err);
                // return Err(err.message());
            }
        }
        self.program_local_store(&accounts).await?;

        Ok(())
    }

    // pub fn get_account_data(&self, pubkey: &Pubkey) -> Result<Option<Arc<RwLock<AccountData>>>> {
    //     Ok(self.store.lookup(pubkey)?)
    //     // let store = self.store();
    //     // let account_data = store.lookup(pubkey)?;
    //     // let account_data = match account_data {
    //     //     Some(account_data) => Some(account_data.clone()),
    //     //     None => None,
    //     // };
    //     // Ok(account_data)
    // }



    /// Load multiple accounts from local store for test program usage
    pub async fn program_local_load(&self, program_id : &Pubkey, accounts : &[AccountMeta]) -> Result<Vec<(Pubkey,AccountData)>> {

        let mut keyset = AHashSet::<Pubkey>::new();

        let mut account_data_vec = Vec::new();
        for descriptor in accounts.iter() {
            let pubkey = descriptor.pubkey;

            if keyset.contains(&pubkey) {
                // log_trace!("Duplicate account supplied to local load: {}", pubkey.to_string());
                return Err(
                    error!("[store] Store::program_local_load(): duplicate account supplied to program: {}",pubkey.to_string())
                );
            } else {
                keyset.insert(pubkey.clone());
            }

            let mut account_data = match self.store.lookup(&pubkey).await? {
                Some(account_data) => {
                    let account_data = account_data.try_read().ok_or(error!("account read lock failed"))?.clone_for_program();

                    log_trace!("[store] ...  loading: {}", account_data.info()?);

                    // log_trace!("... loading account: {} data len: {} lamports: {}", 
                    //     account_data.key.to_string(),
                    //     account_data.data.len(),
                    //     account_data.lamports
                    // );
    
                    account_data
                },
                None => {
                    let account_data = AccountData::new_template_for_program(
                    // let account_data = AccountData::new_allocated_for_program(
                        pubkey.clone(),
                        program_id.clone(),
                        //pubkey.clone(),
                        0
                    );

                    if pubkey == Pubkey::default() {
                        log_trace!("[store] ...   system: {}", account_data.info()?);
                    } else {
                        log_trace!("[store] ... template: {}", account_data.info()?);
                    }

                    // log_trace!("... template account: {} data len: {} lamports: {}", 
                    //     account_data.key.to_string(),
                    //     account_data.data.len(),
                    //     account_data.lamports
                    // );

                    account_data
                }
            };

            account_data.is_signer = descriptor.is_signer;
            account_data.is_writable = descriptor.is_writable;

            account_data_vec.push((pubkey,account_data));
        }

        // let mut account_data = Vec::new();
        // for (pubkey,account_ref_cell) in account_ref_cells.iter() {
        //     account_data.push((pubkey.clone(),account_ref_cell.borrow().clone()));
        // }

        Ok(account_data_vec)
    }


    pub async fn program_local_store<'t>(&self, accounts : &[AccountInfo<'t>]) -> Result<()> {
        // pub async fn program_local_store(&self, accounts : Vec<AccountInfo>) -> Result<()> {
        // pub async fn program_local_store<'info>(&self, accounts : Vec<AccountInfo>) -> Result<()> {

        for account_info in accounts.iter() {
            // if false 
            {
                let rent = Rent::default();
                let account_data_len = account_info.data_len();
                let minimum_balance = rent.minimum_balance(account_data_len);
                let lamports = account_info.lamports.borrow();
                if **lamports < minimum_balance {
                    if *account_info.key != Pubkey::default() {
                        log_trace!("{} {}",style("purging account (below minimum balance):").white().on_red(),account_info.key.to_string());
                        log_trace!("data len: {} balance needed: {}  balance in the account: {}", account_data_len, minimum_balance, **lamports);
                        log_trace!("account type: 0x{:08x}",try_get_container_type(account_info)?);
                        self.store.purge_if_exists(account_info.key).await?;
                    }
                    // log_trace!("[store] skipping store for blank account {}", account_info.key.to_string());
                    continue;
                }
            }
            let account_data = AccountData::from_account_info(account_info,AccountDisposition::Storage);
            // log_trace!("... saving account: {} data len: {} lamports: {}  ... {}", 
            //     account_data.key.to_string(),
            //     account_data.data.len(),
            //     account_data.lamports,
            //     account_data.info(),
            // );
            log_trace!("[store] ...   saving: {}", account_data.info()?);
            // log_trace!("... account data: {:#?}", account_data);
            match self.store.lookup(&account_data.key).await? {
                Some(existing_account_data) => {
                    // let mut dest = account_data_reference.write()?;

                    let mut save = true;
                    let existing_account_data = existing_account_data.read().await;//.ok_or(error!("account read lock failed"))?;
                    if !existing_account_data.is_writable {
                        if account_data.data[..] != existing_account_data.data[..] {
                            log_trace!("WARNING: data changed in non-mutable account");
                            save = false;
                        }
                        // TODO: check if account was changed
                    }
                    if save {
                        self.store.store(Arc::new(RwLock::new(account_data))).await?;
                        // *dest = account_data;
                    }
                },
                None => {
                    self.store.store(Arc::new(RwLock::new(account_data))).await?;
                }
            }

        }

        Ok(())
    }
    






}


/* 
#[cfg(not(target_arch = "bpf"))]
pub mod client {
    use super::AccountData;
    use js_sys::*;
    use wasm_bindgen::prelude::*;

    //  This is not a bingen function!  AccountData is not exposed to bindgen
    pub fn account_data_to_jsv(
        account_data: &AccountData,
    ) -> std::result::Result<JsValue, JsValue> {
        let resp = js_sys::Object::new();
        unsafe {
            js_sys::Reflect::set(
                &resp,
                &"data".into(),
                &JsValue::from(Uint8Array::view(&account_data.data)),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"owner".into(),
                &JsValue::from(Uint8Array::view(&account_data.owner.to_bytes())),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"lamports".into(),
                &JsValue::from_f64(account_data.lamports as f64),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"rentEpoch".into(),
                &JsValue::from_f64(account_data.rent_epoch as f64),
            )?;
            js_sys::Reflect::set(
                &resp,
                &"executable".into(),
                &JsValue::from_bool(account_data.executable),
            )?;
        }
        Ok(resp.into())
    }

}
*/

