use cfg_if::cfg_if;
use std::rc::Rc;
use std::cell::RefCell;

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
// use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

use workflow_allocator::address::ProgramAddressData;
use workflow_allocator::accounts::{
    LamportAllocation, 
    AllocationPayer,
};
use workflow_allocator::error::*;
use workflow_allocator::result::*;
use workflow_allocator::rent::RentCollector;
use workflow_allocator::identity::Identity;
use workflow_allocator::payload::Payload;
use workflow_allocator::container;
use workflow_log::*;

// use crate::container::AccountAggregator;

pub struct AccountAllocationArgs<'info,'refs> {
    lamports : LamportAllocation,
    payer : AllocationPayer<'info,'refs>,
    // reserve_data_len : usize
}

impl<'info,'refs> Default for AccountAllocationArgs<'info,'refs> {
    fn default() -> AccountAllocationArgs<'info,'refs> {
        AccountAllocationArgs {
            lamports : LamportAllocation::Auto,
            payer : AllocationPayer::Authority,
            // reserve_data_len : 0,
        }
    }
}

impl<'info,'refs> AccountAllocationArgs<'info,'refs> {

    pub fn new() -> AccountAllocationArgs<'info,'refs> {
        AccountAllocationArgs {
            lamports : LamportAllocation::Auto,
            payer : AllocationPayer::Authority,
            // reserve_data_len : 0,
        }
    }

    pub fn new_with_payer(payer : &'refs AccountInfo<'info>) -> AccountAllocationArgs<'info,'refs> {
        AccountAllocationArgs {
            lamports : LamportAllocation::Auto,
            payer : AllocationPayer::Account(payer),
            // reserve_data_len : 0,
        }
    }

    // pub fn with_data_len(mut self, data_len : usize) -> Self {
    //     self.reserve_data_len = data_len;
    //     self
    // }
}


// pub type HandlerFn = fn(ctx: &ContextReference) -> Result<()>;
// pub type HandlerFnCPtr = *const fn(ctx: &ContextReference) -> Result<()>;
// pub type ProgramHandlerFn = fn(ctx: &ContextReference) -> ProgramResult;
// pub type ProgramHandlerFnCPtr = *const fn(ctx: &ContextReference) -> ProgramResult;
pub type ContextReference<'info,'refs,'pid,'instr> = Rc<Box<Context<'info,'refs,'pid,'instr>>>;
pub type SimulationHandlerFn = fn(ctx: &ContextReference) -> Result<()>;
pub type HandlerFn = fn(ctx: &ContextReference) -> ProgramResult;
pub type HandlerFnCPtr = *const fn(ctx: &ContextReference) -> ProgramResult;


#[derive(Debug)]
pub struct ContextMeta {
    pub template_accounts_consumed : usize,
    pub template_address_data_bytes_consumed : usize,
}


#[derive(Debug)]
pub struct Context<'info, 'refs, 'pid, 'instr> {

//    payer: &'refs AccountInfo<'info>,
    pub program_id:&'pid Pubkey,

    pub accounts : &'refs [AccountInfo<'info>],
    // pub remaining_accounts : &'refs [AccountInfo<'info>],

    pub authority : &'refs AccountInfo<'info>,
    // pub identity : Option<&'refs AccountInfo<'info>>,
    pub identity : Option<Identity<'info,'refs>>,

    pub system_accounts : &'refs [AccountInfo<'info>],
    pub token_accounts : &'refs [AccountInfo<'info>],
    pub index_accounts : &'refs [AccountInfo<'info>],
    pub handler_accounts : &'refs [AccountInfo<'info>],
    
    pub incoming_data : &'instr [u8],
    pub interface_id : usize,
    pub handler_id : usize,
    pub instruction_data : &'instr [u8],
    
    pub template_accounts : &'refs [AccountInfo<'info>],
    pub template_address_data : &'instr [u8],
    // container_segment_stores : Vec<Rc<SegmentStore<'info,'refs>>>,

    pub meta : RefCell<ContextMeta>,
    // pub runtime : 

    rent : Rent,
}


impl<'info, 'refs, 'pid, 'instr>
    TryFrom<(&'pid Pubkey, &'refs [AccountInfo<'info>], &'instr [u8])> 
    for Context<'info, 'refs, 'pid, 'instr>
{
    type Error = crate::error::Error;

    fn try_from(value:(&'pid Pubkey, &'refs [AccountInfo<'info>], &'instr [u8])) -> Result<Context<'info, 'refs, 'pid, 'instr>> {
        // fn try_into(self :(program_id, accounts, instruction_data) : (&'pid Pubkey, &'refs [AccountInfo<'info>], &'instr [u8])) -> Result<Context<'info, 'refs, 'pid, 'instr>> {

        let (program_id, accounts, incoming_data) = value;

        if incoming_data.len() < std::mem::size_of::<Payload>() {
            log_trace!("bytecode must be at least {} bytes, range supplied is: {}", std::mem::size_of::<Payload>(), incoming_data.len());
            return Err(ErrorCode::NotEnoughAccounts.into());
        }

        // log_trace!("instruction data len: {}", instruction_data.len());

        let payload : &Payload = Payload::try_from(incoming_data)?;
        //instruction_data.try_into().expect("let payload : &Payload = instruction_data.try_into() - context.rs@121");

        let interface_id = payload.interface_id as usize;
        let handler_id = payload.handler_id as usize;

        let flags = payload.flags;
        let has_identity = if flags & crate::payload::PAYLOAD_HAS_IDENTITY_ACCOUNT != 0 { true } else { false };
        // let has_system_account = if flags & crate::payload::PAYLOAD_HAS_SYSTEM_ACCOUNT != 0 { true } else { false };
        // let non_handler_accounts = payload.non_handler_accounts()+1;
        // if accounts.len() < non_handler_accounts {
        //     log_trace!("not enough accounts - len: {} need non-handler accounts: {}", accounts.len(), non_handler_accounts);
        //     return Err(ErrorCode::NotEnoughAccounts.into());
        // }

        // let handler_accounts = &accounts[non_handler_accounts..];
        let incoming_accounts_len = accounts.len();
        let payload_accounts_len = payload.total_accounts();

        if has_identity {
            if incoming_accounts_len < 2 {
                log_trace!("FATAL: Invalid number of context accounts - expecting: {} received: {}",
                    payload_accounts_len+2,
                    incoming_accounts_len
                );
                return Err(ErrorCode::ContextAccounts.into())
            }
        } else {
            if incoming_accounts_len < 1 {
                log_trace!("FATAL: Invalid number of context accounts - expecting: {} received: {}",
                    payload_accounts_len+1,
                    incoming_accounts_len
                );
                return Err(ErrorCode::ContextAccounts.into())
            }

        }

        // if incoming_accounts_len != 1 && incoming_accounts_len != (payload_accounts_len+2) {
        //     log_trace!("FATAL: Invalid number of context accounts - expecting: 1 or {} received: {}",
        //         payload_accounts_len+2,
        //         incoming_accounts_len
        //     );
        //     return Err(ErrorCode::ContextAccounts.into())
        // }

        // let mut offset : usize = if has_system_account { 1 } else { 0 };
        let mut offset = 0;
        let authority = &accounts[offset];
        if !authority.is_signer {
            return Err(error_code!(ErrorCode::AuthorityMustSign))
        }

        log_trace!("");
        log_trace!("{} | authority: {} len: {} lamports: {}",
            style("CTX").magenta(),
            authority.key,
            authority.data.borrow().len(),
            authority.lamports.borrow()
        );
        
        let identity = if has_identity {
            offset += 1;
            let identity_account_info = &accounts[offset];
            log_trace!("{} |  identity: {} len: {} lamports: {}",
                style("CTX").magenta(),
                identity_account_info.key,
                identity_account_info.data.borrow().len(),
                identity_account_info.lamports.borrow()
            );
            // log_trace!("| CTX identity: {}", accounts[offset].key);

            let identity = Identity::try_load(identity_account_info)?;
            if !identity.try_has_authority(authority.key)? {
                return Err(program_error_code!(ErrorCode::IdentityAccess));
            }

            Some(identity)
            //Some(&accounts[offset])
        } else {
            log_trace!("{} |  identity: N/A", style("CTX").magenta());
            None
        };
        offset += 1;
        // let identity = Identity::load(&accounts[offset])?;

        let len = payload.system_accounts_len as usize;
        let system_accounts = &accounts[offset..offset+len];
        offset += len;

        let len = payload.token_accounts_len as usize;
        let token_accounts = &accounts[offset..offset+len];
        offset += len;

        let len = payload.index_accounts_len as usize;
        let index_accounts = &accounts[offset..offset+len];
        offset += len;

        let len = payload.template_accounts_len as usize;
        let template_accounts = &accounts[offset..offset+len];
        offset += len;

        // log_trace!("| incoming accounts: {}", accounts.len());
        // log_trace!("| token accounts: {}", token_accounts.len());
        // log_trace!("| index accounts: {}", index_accounts.len());
        // log_trace!("| template accounts: {}", template_accounts.len());
        // log_trace!("+---");

        let marker = if has_identity { 2 } else { 1 };
        // if has_system_account { marker += 1 };
        assert_eq!(payload_accounts_len+marker, offset); 

        // if has_system_account {
        //     if has_identity {
        //         assert_eq!(payload_accounts_len+3, offset); 
        //     } else {
        //         assert_eq!(payload_accounts_len+2, offset); 
        //     }
        // } else {
        //     if has_identity {
        //         assert_eq!(payload_accounts_len+2, offset); 
        //     } else {
        //         assert_eq!(payload_accounts_len+1, offset); 
        //     }
        // }
        let handler_accounts = &accounts[offset..];

        // ~~~

        // let user_accounts = if has_identity { 2 } else { 1 };
        // let execution_accounts = accounts.len() - user_accounts;
        // log_trace!("\n{}",
        //     style("+---").magenta()
        // );
        // log_trace!("{} - accounts - total: ({}+{}) ▷ {} token: {} index: {} handler: {} template: {}",
        //     style("| Context").magenta(),
        //     style(user_accounts).cyan(),
        //     style(execution_accounts).cyan(),
        //     style(accounts.len()).cyan(),
        //     style(token_accounts.len()).cyan(),
        //     style(index_accounts.len()).cyan(),
        //     style(handler_accounts.len()).cyan(),
        //     style(template_accounts.len()).cyan(),
        // );



        let template_address_data_len = payload.instruction_data_offset as usize;
        // log_trace!("{} - instruction data - total: {} template data {} handler instruction buffer len {}",
        //     style("| Context").magenta(),
        //     style(instruction_data.len()).cyan(),
        //     style(template_address_data_len).cyan(),
        //     style(instruction_data.len() - template_address_data_len).cyan(),
        // );
        // log_trace!("instruction data offset: {}", );
        let template_address_data = &incoming_data[std::mem::size_of::<Payload>()..template_address_data_len];
        let instruction_data = &incoming_data[template_address_data_len..];

        let meta = ContextMeta {
            template_accounts_consumed : 0,
            template_address_data_bytes_consumed : 0,
        };

        // let instruction_data_view = hexplay::HexViewBuilder::new(&instruction_data)
        //     .force_color()
        //     .add_colors(vec![
        //         // (hexplay::color::red(), 42..72),
        //         (hexplay::color::yellow(), 0..2),
        //         // (hexplay::color::green(), 32..38),
        //         // (hexplay::color::blue(), 200..226),
        //     ])
        //     .address_offset(0)
        //     .row_width(16)
        //     .finish();
        //     // instruction_data_view.print();
        // log_trace!("{}", instruction_data_view);
        // log_trace!("");

        let ctx = Context {
            program_id,
            incoming_data,
            instruction_data,
            interface_id,
            handler_id,
            
            accounts,
            authority,
            identity,
            system_accounts,
            token_accounts,
            index_accounts,
            handler_accounts,

            template_accounts,
            template_address_data,

            meta : RefCell::new(meta),

            rent : Rent::default(),
            // container_segment_stores : Vec::with_capacity(offset)
        };

        #[cfg(not(target_arch = "bpf"))]
        {
            log_trace!("");
            ctx.view_info();
            log_trace!("{} |",style("CTX").magenta());
            ctx.view_hex();
            log_trace!("");
        }

        ctx.validate()?;

        Ok(ctx)
    }
}


impl<'info, 'refs, 'pid, 'instr> Context<'info, 'refs, 'pid, 'instr>
{
    pub fn try_identity(&self) -> Result<&Identity<'info,'refs>> {
        match &self.identity {
            None => { Err(error_code!(ErrorCode::IdentityMissing)) },
            Some(identity) => { Ok(identity) }
        }
    }

    #[cfg(not(target_arch = "bpf"))]
    pub fn view_info(&self) {
        let authority_accounts =1;
        let identity_accounts = if self.identity.is_some() { 1 } else { 0 };
        // let execution_accounts = self.accounts.len() - user_accounts;
        // log_trace!("");
        // log_trace!("\n{}",
        //     style("+---").magenta()
        // );

        let total_bytes = 
            self.accounts.len() * 32 +
            self.incoming_data.len(); 

        log_trace!("{} | context payload {} bytes",
            style("CTX").magenta(),
            total_bytes
        );
        log_trace!("{} | accounts - total: {} ▷ auth: {} ident: {} token: {} index: {} handler: {} tpl: {}",
            style("CTX").magenta(),
            style(self.accounts.len()).cyan(),
            style(authority_accounts).cyan(),
            style(identity_accounts).cyan(),
            // style(execution_accounts).cyan(),
            style(self.token_accounts.len()).cyan(),
            style(self.index_accounts.len()).cyan(),
            style(self.handler_accounts.len()).cyan(),
            style(self.template_accounts.len()).cyan(),
        );

        let template_address_data_len = self.incoming_data.len() - self.instruction_data.len();
        log_trace!("{} | incoming data - total: {} template data {} instruction data {}",
            style("CTX").magenta(),
            style(self.incoming_data.len()).cyan(),
            style(template_address_data_len).cyan(),
            style(self.incoming_data.len() - template_address_data_len).cyan(),
        );

        // log_trace!("\n");

    }

    #[cfg(not(target_arch = "bpf"))]
    pub fn view_hex(&self) {
        let instruction_data_view = hexplay::HexViewBuilder::new(&self.incoming_data)
            .force_color()
            .add_colors(vec![
                // (hexplay::color::red(), 42..72),
                (hexplay::color::yellow(), 0..2),
                // (hexplay::color::green(), 32..38),
                // (hexplay::color::blue(), 200..226),
            ])
            .address_offset(0)
            .row_width(16)
            .finish();
            // instruction_data_view.print();
        log_trace!("{} | incoming data {} bytes:", 
            style("CTX").magenta(),
            self.incoming_data.len()
        );

        let string = instruction_data_view.to_string();
        let lines: Vec<String> = string.split("\n").map(|l|format!("{} | {}",style("CTX").magenta(), l).to_string()).collect();
        log_trace!("{}", lines.join("\n"));
        // log_trace!("{}", instruction_data_view);
    }

    pub fn validate(&self) -> Result<()> {
        match &self.identity {
            Some(identity) => {
                if identity.account().owner != self.program_id {
                    return Err(error_code!(ErrorCode::AccountOwnership))
                }    
            },
            None => { }
        }

        for index_account in self.index_accounts {
            if index_account.owner != self.program_id {
                return Err(error_code!(ErrorCode::AccountOwnership))
            }
        }

        for handler_account in self.handler_accounts {
            if handler_account.owner != self.program_id {
                return Err(error_code!(ErrorCode::AccountOwnership))
            }
        }




        Ok(())
    }

    pub fn try_consume_program_address_data(&self) -> Result<(ProgramAddressData<'instr>, &'refs AccountInfo<'info>)> {

        // log_trace!("try_consume_program_address_data()");
        
        let mut meta = self.meta.borrow_mut();
        let account_index = meta.template_accounts_consumed;
        let byte_offset = meta.template_address_data_bytes_consumed;
        // log_trace!("~~~ current byte offset: {}", byte_offset);
        // log_trace!("try_consume_program_address_data() A: byte_offset:{:?}", byte_offset);
        // log_trace!("self.template_address_data.len: {}", self.template_address_data.len());
        
        // log_trace!("all data:");
        // trace_hex(self.template_address_data);
        // log_trace!("template address data:");
        // trace_hex(&self.template_address_data[byte_offset..]);

        let (program_address_data_ref, bytes_used) = ProgramAddressData::try_from(
            &self.template_address_data[byte_offset..]
        )?;
        // log_trace!("~~~ current bytes used: {}", bytes_used);
        // log_trace!("try_consume_program_address_data() B");
        if byte_offset + bytes_used > self.template_address_data.len() {
            return Err(ErrorCode::PDAAccountArgumentData.into());
        }
        // log_trace!("try_consume_program_address_data() C");
        meta.template_accounts_consumed += 1;
        meta.template_address_data_bytes_consumed += bytes_used;
        
        // log_trace!("try_consume_program_address_data() D");
        // let seed :[u8;32] = self.program_id.to_bytes();
        // let bump: u8 = 255u8;
        // let seed_suffix_str: String = String::from("");
        
        // ^ TODO: deserealize PAD from IB!
        // ? TODO: deserealize PAD from IB!
        
        // let program_address_data = ProgramAddressData {
        //     seed, bump, seed_suffix_str
        // };

        let account_info = &self.template_accounts[account_index];

        Ok((program_address_data_ref, account_info))
    }

    pub fn create_pda(&self, data_len : usize, allocation_args : &AccountAllocationArgs<'info,'refs>) -> Result<&'refs AccountInfo<'info>> {

        // sanity check
        cfg_if! {
            if #[cfg(not(target_arch = "bpf"))] {
                if self.system_accounts.iter().position(|account_info| account_info.key == &solana_sdk::system_program::id()).is_none() {
                    return Err(error_code!(ErrorCode::SystemProgramAccountMissing));
                }
            }
        }

        log_trace!("[pda] ... create_pda() starting ...");
        let (tpl_program_address_data,tpl_account_info) = self.try_consume_program_address_data()?;
        log_trace!("[pda] ... create_pda() for account {}", tpl_account_info.key.to_string());
        
        // log_trace!(" CREATE PDA ACCOUNT DATA: ------------------------- * * *");
        // trace_hex(&*tpl_account_info.data.borrow());
        // log_trace!(" CREATE PDA ACCOUNT DATA: ------------------------- * * *");

        if let Ok(container_type) = container::try_get_container_type(tpl_account_info) {
            if container_type != 0 {
                return Err(ErrorCode::TplAccountHasData.into())
            }
        }    

        let lamports = match allocation_args.lamports {
            LamportAllocation::Auto => {
                let rent = Rent::default();
                rent.minimum_balance(data_len)
            },
            LamportAllocation::Lamports(lamports) => {
                lamports
            }
        };
        
        // log_trace!("| pda: checking allocation args");
        let payer = match allocation_args.payer {
            AllocationPayer::Authority => {
                &self.authority
            },
            // Identity => {
                //     todo!("Identity-based payments are not currently supported")
                // },
                AllocationPayer::Account(account_info) => {
                    account_info
                }
            };
        // log_trace!("D");

        let user_seed = match &self.identity {
            Some(identity) => identity.pubkey().to_bytes(),
            None => {
                self.authority.key.to_bytes()
            }
        };
            
        // log_trace!("| pda: executing allocate_pda()");
        let res = crate::allocate_pda(
            payer, //&self.authority,
            self.program_id,
            &user_seed,
            &tpl_program_address_data,
            tpl_account_info,
            data_len,
            lamports,
            // &[(data_len, lamports)]
        )?;

        // let identity = 
        if let Some(identity) = &self.identity {
            identity.advance_pda_sequence()?;
        }

        Ok(res)
    }

    pub fn sync_rent(&self, account_info : &'refs AccountInfo<'info>, _rent_collector : &RentCollector<'info,'refs>) -> Result<()> {

        // let rent = Rent::default();
        let data_len = account_info.data_len();
        let minimum_balance = self.rent.minimum_balance(data_len);
        let lamports = account_info.lamports();

        // let authority_lamports = self.authority.lamports();
        // if authority_lamports < lamports {
        //     return Err(error_code!(ErrorCode::InsufficientBalanceForRentSync));
        // }

        if lamports < minimum_balance {
            let delta = minimum_balance - lamports;
            log_trace!("... transferring additional {} lamports to compensate rent", delta);
            workflow_allocator::transfer_sol(
                self.authority,
                account_info,
                self.authority,
                self.authority,//TODO @MATOO
                delta,
                // from, to, authority, amount

            )?;
        }

        // FIXME implement rent collector

        Ok(())
    }

    pub fn purge(&self, _account_info : &'refs AccountInfo<'info>, _rent_collector: &RentCollector<'info,'refs>) -> Result<()> {
        // FIXME: implement rent collector
        Ok(())
    }

    #[inline(always)]
    pub fn locate_system_account(&self, pubkey : &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self.system_accounts.iter().position(|account| account.key == pubkey) {
            Some(&self.system_accounts[index])
        } else { None }
    }

    #[inline(always)]
    pub fn locate_token_account(&self, pubkey : &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self.token_accounts.iter().position(|account| account.key == pubkey) {
            Some(&self.token_accounts[index])
        } else { None } 
    }

    #[inline(always)]
    // pub fn locate_index_account<'ctx>(&'ctx self, pubkey : &Pubkey) -> Option<&'ctx AccountInfo<'info>> {
    pub fn locate_index_account(&self, pubkey : &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self.index_accounts.iter().position(|account| account.key == pubkey) {
            Some(&self.index_accounts[index])
        } else { None } 
    }

    #[inline(always)]
    pub fn locate_handler_account(&self, pubkey : &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self.handler_accounts.iter().position(|account| account.key == pubkey) {
            Some(&self.handler_accounts[index])
        } else { None } 
    }


}
