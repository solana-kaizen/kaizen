use cfg_if::cfg_if;
use std::cell::RefCell;
use std::rc::Rc;

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
// use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

use kaizen::accounts::{AllocationPayer, LamportAllocation};
use kaizen::address::{AddressDomain, ProgramAddressData};
use kaizen::container;
use kaizen::error::*;
use kaizen::identity::program::Identity;
use kaizen::payload::Payload;
use kaizen::rent::RentCollector;
use kaizen::result::*;
use workflow_log::*;
pub enum AccountType {
    Token,
    Index,
    Handler,
}
pub struct AccountAllocationArgs<'info, 'refs, 'seed> {
    /// Seed prefix: None, Authority key, Identity key, or Identity key w/sequence
    pub domain: AddressDomain,
    pub seed: Option<&'seed [u8]>,
    pub lamports: LamportAllocation,
    pub payer: AllocationPayer<'info, 'refs>,
    // reserve_data_len : usize
}

impl<'info, 'refs, 'seed> Default for AccountAllocationArgs<'info, 'refs, 'seed> {
    fn default() -> AccountAllocationArgs<'info, 'refs, 'seed> {
        AccountAllocationArgs {
            lamports: LamportAllocation::Auto,
            payer: AllocationPayer::Authority,
            domain: AddressDomain::Default,
            seed: None,
            // reserve_data_len : 0,
        }
    }
}

impl<'info, 'refs, 'seed> AccountAllocationArgs<'info, 'refs, 'seed> {
    pub fn new(domain: AddressDomain) -> AccountAllocationArgs<'info, 'refs, 'seed> {
        AccountAllocationArgs {
            domain, // : AddressDomain::Identity,
            seed: None,
            lamports: LamportAllocation::Auto,
            payer: AllocationPayer::Authority,
            // reserve_data_len : 0,
        }
    }

    pub fn new_with_payer(
        domain: AddressDomain,
        payer: AllocationPayer<'info, 'refs>,
    ) -> AccountAllocationArgs<'info, 'refs, 'seed> {
        AccountAllocationArgs {
            domain,
            seed: None,
            lamports: LamportAllocation::Auto,
            payer,
        }
    }

    pub fn new_with_args(
        domain: AddressDomain,
        lamports: Option<LamportAllocation>,
        payer: Option<AllocationPayer<'info, 'refs>>,
        seed: Option<&'seed [u8]>,
    ) -> AccountAllocationArgs<'info, 'refs, 'seed> {
        AccountAllocationArgs {
            domain,
            seed,
            lamports: lamports.unwrap_or(LamportAllocation::Auto),
            payer: payer.unwrap_or(AllocationPayer::Authority),
            // reserve_data_len : 0,
        }
    }

    // pub fn with_data_len(mut self, data_len : usize) -> Self {
    //     self.reserve_data_len = data_len;
    //     self
    // }
}

pub type ContextReference<'info, 'refs, 'pid, 'instr> =
    Rc<Box<Context<'info, 'refs, 'pid, 'instr>>>;
pub type SimulationHandlerFn = fn(ctx: &ContextReference) -> Result<()>;
pub type HandlerFn = fn(ctx: &ContextReference) -> ProgramResult;
pub type HandlerFnCPtr = *const fn(ctx: &ContextReference) -> ProgramResult;

#[derive(Debug)]
pub struct SyncRent<'info, 'refs> {
    pub account: &'refs AccountInfo<'info>,
    pub collector: RentCollector<'info, 'refs>,
}

#[derive(Debug)]
pub struct ContextMeta<'info, 'refs> {
    pub generic_template_accounts_consumed: usize,
    pub generic_template_data_bytes_consumed: usize,
    pub collection_template_accounts_consumed: usize,
    pub collection_template_data_bytes_consumed: usize,

    // TODO - implement automatic rent syncing
    pub sync_rent: Vec<SyncRent<'info, 'refs>>,
}

#[derive(Debug)]
pub struct Context<'info, 'refs, 'pid, 'instr> {
    //    payer: &'refs AccountInfo<'info>,
    pub program_id: &'pid Pubkey,

    pub accounts: &'refs [AccountInfo<'info>],
    // pub remaining_accounts : &'refs [AccountInfo<'info>],
    pub authority: &'refs AccountInfo<'info>,
    // pub identity : Option<&'refs AccountInfo<'info>>,
    pub identity: Option<Identity<'info, 'refs>>,

    pub system_accounts: &'refs [AccountInfo<'info>],
    pub token_accounts: &'refs [AccountInfo<'info>],
    pub index_accounts: &'refs [AccountInfo<'info>],
    pub collection_accounts: &'refs [AccountInfo<'info>],
    pub handler_accounts: &'refs [AccountInfo<'info>],

    pub incoming_data: &'instr [u8],
    pub interface_id: usize,
    pub handler_id: usize,
    pub instruction_data: &'instr [u8],

    pub generic_template_accounts: &'refs [AccountInfo<'info>],
    pub generic_template_data: &'instr [u8],
    pub collection_template_accounts: &'refs [AccountInfo<'info>],
    pub collection_template_data: &'instr [u8],

    pub meta: RefCell<ContextMeta<'info, 'refs>>,

    rent: Rent,
}

impl<'info, 'refs, 'pid, 'instr> TryFrom<(&'pid Pubkey, &'refs [AccountInfo<'info>], &'instr [u8])>
    for Context<'info, 'refs, 'pid, 'instr>
{
    type Error = crate::error::Error;

    fn try_from(
        value: (&'pid Pubkey, &'refs [AccountInfo<'info>], &'instr [u8]),
    ) -> Result<Context<'info, 'refs, 'pid, 'instr>> {
        let (program_id, accounts, incoming_data) = value;

        if incoming_data.len() < std::mem::size_of::<Payload>() {
            log_trace!(
                "bytecode must be at least {} bytes, range supplied is: {}",
                std::mem::size_of::<Payload>(),
                incoming_data.len()
            );
            return Err(ErrorCode::NotEnoughAccounts.into());
        }

        let payload: &Payload = Payload::try_from(incoming_data)?;

        let interface_id = payload.interface_id as usize;
        let handler_id = payload.handler_id as usize;

        let flags = payload.flags;
        let has_identity = flags & crate::payload::PAYLOAD_HAS_IDENTITY_ACCOUNT != 0;

        let incoming_accounts_len = accounts.len();
        let payload_accounts_len = payload.total_accounts();

        if has_identity && incoming_accounts_len < 2 {
            log_trace!(
                "FATAL: Invalid number of context accounts - expecting: {} received: {}",
                payload_accounts_len + 2,
                incoming_accounts_len
            );
            return Err(ErrorCode::ContextAccounts.into());
        } else if incoming_accounts_len < 1 {
            log_trace!(
                "FATAL: Invalid number of context accounts - expecting: {} received: {}",
                payload_accounts_len + 1,
                incoming_accounts_len
            );
            return Err(ErrorCode::ContextAccounts.into());
        }

        let mut offset = 0;
        let authority = &accounts[offset];
        if !authority.is_signer {
            return Err(error_code!(ErrorCode::AuthorityMustSign));
        }

        log_trace!("");
        log_trace!(
            "{} | authority: {} len: {} lamports: {}",
            style("CTX").magenta(),
            authority.key,
            authority.data.borrow().len(),
            authority.lamports.borrow()
        );

        let identity = if has_identity {
            offset += 1;
            let identity_account_info = &accounts[offset];
            log_trace!(
                "{} |  identity: {} len: {} lamports: {}",
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
        } else {
            log_trace!("{} |  identity: N/A", style("CTX").magenta());
            None
        };
        offset += 1;

        let len = payload.system_accounts_len as usize;
        let system_accounts = &accounts[offset..offset + len];
        offset += len;

        let len = payload.token_accounts_len as usize;
        let token_accounts = &accounts[offset..offset + len];
        offset += len;

        let len = payload.index_accounts_len as usize;
        let index_accounts = &accounts[offset..offset + len];
        offset += len;

        let len = payload.collection_accounts_len as usize;
        let collection_accounts = &accounts[offset..offset + len];
        offset += len;

        let len = payload.generic_template_accounts_len as usize;
        let generic_template_accounts = &accounts[offset..offset + len];
        offset += len;

        let len = payload.collection_template_accounts_len as usize;
        let collection_template_accounts = &accounts[offset..offset + len];
        offset += len;

        let marker = if has_identity { 2 } else { 1 };
        assert_eq!(payload_accounts_len + marker, offset);

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

        // let template_address_data_finish = payload.collection_data_offset as usize;
        // let template_address_data_len = payload.instruction_data_offset as usize;
        // log_trace!("{} - instruction data - total: {} template data {} handler instruction buffer len {}",
        //     style("| Context").magenta(),
        //     style(instruction_data.len()).cyan(),
        //     style(template_address_data_len).cyan(),
        //     style(instruction_data.len() - template_address_data_len).cyan(),
        // );
        // log_trace!("instruction data offset: {}", );
        // let template_address_data = &incoming_data[std::mem::size_of::<Payload>()..template_address_data_finish];
        // let instruction_data_offset = payload.collection_data_offset +
        // let instruction_data = &incoming_data[template_address_data_len..];

        let instruction_data_offset = payload.instruction_data_offset as usize;
        let collection_template_data_offset = payload.collection_data_offset as usize;
        let generic_template_data =
            &incoming_data[std::mem::size_of::<Payload>()..collection_template_data_offset];
        let collection_template_data =
            &incoming_data[collection_template_data_offset..instruction_data_offset];
        let instruction_data = &incoming_data[instruction_data_offset..];

        let meta = ContextMeta {
            generic_template_accounts_consumed: 0,
            generic_template_data_bytes_consumed: 0,
            collection_template_accounts_consumed: 0,
            collection_template_data_bytes_consumed: 0,
            sync_rent: Vec::new(),
        };

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
            collection_accounts,
            handler_accounts,

            generic_template_accounts,
            generic_template_data,
            collection_template_accounts,
            collection_template_data,

            meta: RefCell::new(meta),

            // TODO use Rent::get()? in bpf
            rent: Rent::default(),
        };

        #[cfg(not(target_os = "solana"))]
        {
            log_trace!("");
            ctx.view_info();
            log_trace!("{} |", style("CTX").magenta());
            ctx.view_hex();
            log_trace!("");
        }

        ctx.validate()?;

        Ok(ctx)
    }
}

impl<'info, 'refs, 'pid, 'instr> Context<'info, 'refs, 'pid, 'instr> {
    pub fn try_identity(&self) -> Result<&Identity<'info, 'refs>> {
        match &self.identity {
            None => Err(error_code!(ErrorCode::IdentityMissing)),
            Some(identity) => Ok(identity),
        }
    }

    #[cfg(not(target_os = "solana"))]
    pub fn view_info(&self) {
        let authority_accounts = 1;
        let identity_accounts = i32::from(self.identity.is_some());
        // if self.identity.is_some() { 1 } else { 0 };
        // let execution_accounts = self.accounts.len() - user_accounts;
        // log_trace!("");
        // log_trace!("\n{}",
        //     style("+---").magenta()
        // );

        let total_bytes = self.accounts.len() * 32 + self.incoming_data.len();

        log_trace!(
            "{} | context payload {} bytes",
            style("CTX").magenta(),
            total_bytes
        );
        log_trace!("{} | accounts - total: {} ▷ auth: {} ident: {} token: {} index: {} collection: {} handler: {} gtpl: {} ctpl: {}",
            style("CTX").magenta(),
            style(self.accounts.len()).cyan(),
            style(authority_accounts).cyan(),
            style(identity_accounts).cyan(),
            // style(execution_accounts).cyan(),
            style(self.token_accounts.len()).cyan(),
            style(self.index_accounts.len()).cyan(),
            style(self.collection_accounts.len()).cyan(),
            style(self.handler_accounts.len()).cyan(),
            style(self.generic_template_accounts.len()).cyan(),
            style(self.collection_template_accounts.len()).cyan(),
        );

        let instruction_data_len = self.incoming_data.len()
            - std::mem::size_of::<Payload>()
            - self.generic_template_data.len()
            - self.collection_template_data.len();

        log_trace!(
            "{} | incoming data - total: {} header: {} gtpl: {} ctpl: {} handler: {}",
            style("CTX").magenta(),
            style(self.incoming_data.len()).cyan(),
            style(std::mem::size_of::<Payload>()).cyan(),
            style(self.generic_template_data.len()).cyan(),
            style(self.collection_template_data.len()).cyan(),
            style(instruction_data_len).cyan(),
        );

        // log_trace!("\n");
    }

    #[cfg(not(target_os = "solana"))]
    pub fn view_hex(&self) {
        let instruction_data_view = hexplay::HexViewBuilder::new(self.incoming_data)
            .force_color()
            .add_colors(vec![(hexplay::color::yellow(), 0..2)])
            .address_offset(0)
            .row_width(16)
            .finish();

        log_trace!(
            "{} | incoming data {} bytes:",
            style("CTX").magenta(),
            self.incoming_data.len()
        );

        let string = instruction_data_view.to_string();
        let lines: Vec<String> = string
            .split('\n')
            .map(|l| format!("{} | {}", style("CTX").magenta(), l))
            .collect();
        log_trace!("{}", lines.join("\n"));
    }

    pub fn validate(&self) -> Result<()> {
        match &self.identity {
            Some(identity) => {
                if identity.account().owner != self.program_id {
                    return Err(error_code!(ErrorCode::AccountOwnership));
                }
            }
            None => {}
        }

        for index_account in self.index_accounts {
            if index_account.owner != self.program_id {
                return Err(error_code!(ErrorCode::AccountOwnership));
            }
        }

        for handler_account in self.handler_accounts {
            if handler_account.owner != self.program_id {
                return Err(error_code!(ErrorCode::AccountOwnership));
            }
        }

        Ok(())
    }

    pub fn try_consume_collection_template_address_data(
        &self,
    ) -> Result<(ProgramAddressData<'instr>, &'refs AccountInfo<'info>)> {
        let mut meta = self.meta.borrow_mut();
        let account_index = meta.collection_template_accounts_consumed;
        let byte_offset = meta.collection_template_data_bytes_consumed;

        let (program_address_data_ref, bytes_used) =
            ProgramAddressData::try_from(&self.collection_template_data[byte_offset..])?;

        if byte_offset + bytes_used > self.collection_template_data.len() {
            return Err(ErrorCode::PDAAccountArgumentData.into());
        }

        meta.collection_template_accounts_consumed += 1;
        meta.collection_template_data_bytes_consumed += bytes_used;

        let account_info = &self.collection_template_accounts[account_index];

        Ok((program_address_data_ref, account_info))
    }

    pub fn try_consume_generic_template_address_data(
        &self,
    ) -> Result<(ProgramAddressData<'instr>, &'refs AccountInfo<'info>)> {
        let mut meta = self.meta.borrow_mut();
        let account_index = meta.generic_template_accounts_consumed;
        let byte_offset = meta.generic_template_data_bytes_consumed;

        let (program_address_data_ref, bytes_used) =
            ProgramAddressData::try_from(&self.generic_template_data[byte_offset..])?;

        if byte_offset + bytes_used > self.generic_template_data.len() {
            return Err(ErrorCode::PDAAccountArgumentData.into());
        }

        meta.generic_template_accounts_consumed += 1;
        meta.generic_template_data_bytes_consumed += bytes_used;

        let account_info = &self.generic_template_accounts[account_index];

        Ok((program_address_data_ref, account_info))
    }

    pub fn try_create_pda_with_args(
        &self,
        data_len: usize,
        allocation_args: &AccountAllocationArgs<'info, '_, '_>,
        tpl_seeds: &[&[u8]],
        tpl_account_info: &AccountInfo<'info>,
        validate_pda: bool,
    ) -> Result<()> {
        cfg_if! {
            if #[cfg(not(target_os = "solana"))] {
                if !self.system_accounts.iter().any(|account_info| account_info.key == &solana_sdk::system_program::id()) {
                    return Err(error_code!(ErrorCode::SystemProgramAccountMissing));
                }
            }
        }

        if let Ok(container_type) = container::try_get_container_type(tpl_account_info) {
            if container_type != 0 {
                return Err(ErrorCode::TplAccountHasData.into());
            }
        }

        let lamports = match allocation_args.lamports {
            LamportAllocation::Auto => {
                let rent = Rent::default();
                rent.minimum_balance(data_len)
            }
            LamportAllocation::Lamports(lamports) => lamports,
        };

        let payer = match allocation_args.payer {
            AllocationPayer::Authority => self.authority,
            AllocationPayer::Identity => match &self.identity {
                Some(identity) => identity.account(),
                None => return Err(error_code!(ErrorCode::IdentityMissingForAlloc)),
            },
            AllocationPayer::Account(account_info) => account_info,
        };

        crate::allocate_pda(
            payer,
            self.program_id,
            tpl_seeds,
            tpl_account_info,
            data_len,
            lamports,
            validate_pda,
        )?;

        Ok(())
    }

    pub fn try_create_pda(
        &self,
        // &'ctx self,
        data_len: usize,
        allocation_args: &AccountAllocationArgs<'info, '_, '_>,
    ) -> Result<&'refs AccountInfo<'info>> {
        // log_trace!("[pda] ... create_pda() starting for {} bytes", data_len);
        let (tpl_program_address_data, tpl_account_info) =
            self.try_consume_generic_template_address_data()?;
        // log_trace!("[pda] ... create_pda() for account {}", tpl_account_info.key.to_string());

        let mut advance_pda_sequence = false;
        let domain_seed = match &allocation_args.domain {
            AddressDomain::None => {
                vec![]
            }
            AddressDomain::Default => {
                match &self.identity {
                    Some(identity) => {
                        advance_pda_sequence = true;
                        // let bytes =
                        identity.pubkey().to_bytes().to_vec()
                        // bytes.as_slice()
                    }
                    None => self.authority.key.to_bytes().to_vec(),
                }
            }
            AddressDomain::Authority => self.authority.key.to_bytes().to_vec(),
            AddressDomain::Identity => {
                if let Some(identity) = &self.identity {
                    advance_pda_sequence = true;
                    identity.pubkey().to_bytes().to_vec()
                } else {
                    return Err(error_code!(ErrorCode::IdentityMissingForAlloc));
                }
            }
        };

        self.try_create_pda_with_args(
            data_len,
            allocation_args,
            // &user_seed,
            &[&domain_seed, tpl_program_address_data.seed],
            tpl_account_info,
            true,
        )?;

        if advance_pda_sequence {
            self.identity.as_ref().unwrap().advance_pda_sequence()?;
        }

        Ok(tpl_account_info)
    }

    pub fn sync_rent(
        &self,
        account_info: &'refs AccountInfo<'info>,
        _rent_collector: &RentCollector<'info, 'refs>,
    ) -> Result<()> {
        let data_len = account_info.data_len();
        let minimum_balance = self.rent.minimum_balance(data_len);
        let lamports = account_info.lamports();

        if lamports < minimum_balance {
            let delta = minimum_balance - lamports;
            log_trace!(
                "... transferring additional {} lamports to compensate rent",
                delta
            );
            kaizen::transfer_sol(
                self.authority,
                account_info,
                self.authority,
                self.authority, //TODO @MATOO
                delta,
            )?;
        }

        // FIXME implement rent collector

        Ok(())
    }

    pub fn purge(
        &self,
        _account_info: &'refs AccountInfo<'info>,
        _rent_collector: &RentCollector<'info, 'refs>,
    ) -> Result<()> {
        // FIXME: implement rent collector
        Ok(())
    }

    #[inline(always)]
    pub fn locate_system_account(&self, pubkey: &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self
            .system_accounts
            .iter()
            .position(|account| account.key == pubkey)
        {
            Some(&self.system_accounts[index])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn locate_token_account(&self, pubkey: &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self
            .token_accounts
            .iter()
            .position(|account| account.key == pubkey)
        {
            Some(&self.token_accounts[index])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn locate_index_account(&self, pubkey: &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self
            .index_accounts
            .iter()
            .position(|account| account.key == pubkey)
        {
            Some(&self.index_accounts[index])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn locate_collection_account(&self, pubkey: &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self
            .collection_accounts
            .iter()
            .position(|account| account.key == pubkey)
        {
            Some(&self.collection_accounts[index])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn locate_handler_account(&self, pubkey: &Pubkey) -> Option<&'refs AccountInfo<'info>> {
        if let Some(index) = self
            .handler_accounts
            .iter()
            .position(|account| account.key == pubkey)
        {
            Some(&self.handler_accounts[index])
        } else {
            None
        }
    }
}
