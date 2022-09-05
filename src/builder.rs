use std::rc::Rc;
use std::cell::RefCell;
use crate::accounts::{
    IsSigner,
    Access,
    SeedSuffix
};
use crate::result::*;
use crate::error;
use crate::payload::Payload;
use solana_program::pubkey::Pubkey;
use solana_program::instruction::AccountMeta;
use workflow_allocator::context::{ HandlerFn, HandlerFnCPtr };
use workflow_allocator::container::AccountAggregator;
use workflow_allocator::instruction::{
    // readonly,
    writable
};

pub fn find_interface_id(program_fn : HandlerFn, handlers: &[HandlerFn]) -> usize {
    handlers.iter()
    .position(|&hfn| hfn as HandlerFnCPtr == program_fn as HandlerFnCPtr )
    .expect("handler is not registered")
}

// ~~~

// this is currently for testing only
#[derive(Debug)]
pub struct InstructionBuilderConfig {
    pub authority : Option<AccountMeta>,
    pub identity : Option<AccountMeta>,
    pub program_id : Pubkey,
    pub suffix_seed_seq : Option<Rc<RefCell<u64>>>,
}

impl InstructionBuilderConfig {
    pub fn new(program_id : Pubkey) -> Self {
        InstructionBuilderConfig {
            authority : None,
            identity : None,
            program_id,
            suffix_seed_seq : None,
        }
    }

    pub fn with_authority(mut self, authority : &Pubkey) -> Self {
        self.authority = Some(AccountMeta::new(*authority,true));
        self
    }

    pub fn with_identity(mut self, identity : &Pubkey) -> Self {
        self.identity = Some(AccountMeta::new(*identity,false));
        self
    }

    pub fn with_sequence(mut self, sequence : u64) -> Self {
        self.suffix_seed_seq = Some(Rc::new(RefCell::new(sequence)));
        self
    }

}


pub type TemplateAccessDescriptor = (IsSigner,Access,SeedSuffix);

/// # Helper for creating a structured program Instruction 
/// Structured program instructions are used by the Workflow Allocator framework.
/// 
/// Accounts and instruction data are serialized into the Solana instruction buffer
/// and deserialized into the [crate::context::Context] during the program execution.
#[derive(Debug)]
pub struct InstructionBuilder {
    /// Operation authority (user wallet account)
    pub authority : Option<AccountMeta>,
    /// Operation identity (user identity account)
    pub identity : Option<AccountMeta>,
    /// Program id for instruction execution
    pub program_id : Pubkey,
    /// Program interface receiving the instruction (should be `SomeStruct::program`)
    pub interface_id : u16,
    /// Program handler functoin receiving the instruction (should be `SomeStruct::handler_fn`)
    pub handler_id : u16,

    /// List of included system accounts (if templates exist System Program account is added automatically)
    system_accounts : Vec<AccountMeta>,
    /// List of accounts used for token operations (SOL/SPL token accounts)
    token_accounts : Vec<AccountMeta>,
    /// List of accounts for used for indexing purposes (BTree accounts)
    index_accounts : Vec<AccountMeta>,
    /// List of accounts intended for the handler function (specific to the current operation)
    handler_accounts : Vec<AccountMeta>,
    /// Instruction buffer received by the handler function (excludes the [Payload] header)
    handler_instruction_data : Vec<u8>,
    /// List of template accounts used for PDA creation (The instruction buffer Payload header will  contain the corresponding PDA seed information)
    template_accounts : Vec<AccountMeta>,
    /// List of seeds used for PDA creation
    template_address_data : Vec<Vec<u8>>,
    /// Chunk of serialized PDA seed data (used for Instruction Buffer assembly)
    template_instruction_data : Vec<u8>,

    // signals if the Instruction Builder has been sealed (meaning the final Instruction Buffer has been integrated)
    is_sealed : bool,
    // starting value for the PDA seed sequence used in this operation
    suffix_seed_seq : u64,
    // TODO
    template_access_descriptors : Vec<TemplateAccessDescriptor>,

    // Reference to an external seed that should be used during PDA creation (allowing PDA seed 
    // sequence value to be tracked in an external object such as `InstructionBufferConfig`)
    track_suffix_seed_seq : Option<Rc<RefCell<u64>>>,
}

impl InstructionBuilder {

    
    /// Creates a default [InstructionBuilder] copying Authority, optional Identity and an optional PDA seed sequence
    /// from the InstructionBuilderConfig 
    /// 
    /// The receiver Interface and Program Instruction are set to zero.
    /// 
    /// This function is used for testing purposes.
    pub fn new_with_config_for_testing(config : &InstructionBuilderConfig) -> InstructionBuilder {
        Self::new_with_config(config,0,0u16)
    }

    /// Creates a default InstructionBuilder copying Authority, optional Identity and an optional PDA seed sequence
    /// from the InstructionBuilderConfig
    pub fn new_with_config<T : Into<u16>>(config : &InstructionBuilderConfig, interface_id: usize, program_instruction: T) -> InstructionBuilder {

        // let track_suffix_seed_seq = match config.suffix_seed_seq {
        //     Some(ref) => 
        // }

        let track_suffix_seed_seq = config.suffix_seed_seq.clone();
        let suffix_seed_seq = match &track_suffix_seed_seq {
            None => 0u64,
            Some(suffix_seed_seq_refcell) => {
                *suffix_seed_seq_refcell.borrow()
            }
        };

        let builder = InstructionBuilder {
            authority : config.authority.clone(),
            identity : config.identity.clone(),
            program_id : config.program_id,

            interface_id : interface_id as u16,
            handler_id : program_instruction.into(),
            system_accounts : Vec::new(),
            token_accounts : Vec::new(),
            index_accounts : Vec::new(),
            handler_accounts : Vec::new(),
            handler_instruction_data : Vec::new(),
            template_accounts : Vec::new(),
            template_address_data : Vec::new(),
            template_instruction_data : Vec::new(),
            
            is_sealed : false,
            suffix_seed_seq, // : 0u64,
            template_access_descriptors : Vec::new(),

            track_suffix_seed_seq
        };

        builder

    }

    pub fn new_for_testing(program_id: &Pubkey) -> InstructionBuilder {
        Self::new(program_id,0,0u16)
    }
    
    pub fn new<T : Into<u16>>(program_id: &Pubkey, interface_id: usize, handler_id: T) -> InstructionBuilder {
        InstructionBuilder {
            authority : None,
            identity : None,
            program_id : program_id.clone(),

            interface_id : interface_id as u16,
            handler_id : handler_id.into(),
            system_accounts : Vec::new(),
            token_accounts : Vec::new(),
            index_accounts : Vec::new(),
            handler_accounts : Vec::new(),
            handler_instruction_data : Vec::new(),
            template_accounts : Vec::new(),
            template_address_data : Vec::new(),
            template_instruction_data : Vec::new(),

            is_sealed : false,
            suffix_seed_seq : 0u64,
            template_access_descriptors : Vec::new(),

            track_suffix_seed_seq : None
        }
    }

    pub fn template_accounts(&self) -> Vec<AccountMeta> {
        self.template_accounts.clone()
    }

    pub fn payload(&self) -> Payload {

        let instruction_data_offset = std::mem::size_of::<Payload>() + self.template_instruction_data.len();
        //  log_trace!("* * * INSTRUCTION DATA OFFSET {}", instruction_data_offset);

        let mut flags: u16 = 0;
        if self.identity.is_some() {
            flags |= crate::payload::PAYLOAD_HAS_IDENTITY_ACCOUNT;
        }

        Payload {
            version : Payload::version(),
            system_accounts_len : self.system_accounts.len() as u8,
            token_accounts_len : self.token_accounts.len() as u8,
            index_accounts_len : self.index_accounts.len() as u8,
            template_accounts_len : self.template_accounts.len() as u8,

            // NOTE: handler accounts are the remaining accounts supplied to the program

            flags,

            instruction_data_offset : instruction_data_offset as u16,

            interface_id : self.interface_id,
            handler_id : self.handler_id,
        }
    }

    pub fn with_system_program_account(mut self) -> Self {
        self.system_accounts.push(AccountMeta::new(solana_sdk::system_program::id(), false));
        self
    }

    pub fn with_system_accounts(mut self, system_accounts: &[AccountMeta]) -> Self {
        self.system_accounts.extend_from_slice(system_accounts);
        self
    }

    pub fn with_token_accounts(mut self, token_accounts: &[AccountMeta]) -> Self {
        self.token_accounts.extend_from_slice(token_accounts);
        self
    }

    pub fn with_index_accounts(mut self, index_accounts: &[AccountMeta]) -> Self {
        self.index_accounts.extend_from_slice(index_accounts);
        self
    }

    pub fn with_handler_accounts(mut self, handler_accounts: &[AccountMeta]) -> Self {
        self.handler_accounts.extend_from_slice(handler_accounts);
        self
    }

    pub fn with_instruction_data(mut self, instruction_data : &[u8]) -> Self {
        self.handler_instruction_data.extend(instruction_data);
        self
    }

    pub fn with_authority(mut self, authority : &Pubkey) -> Self {
        self.authority = Some(AccountMeta::new(*authority,true));
        self
    }

    pub fn with_identity(mut self, identity : &Pubkey) -> Self {
        self.identity = Some(AccountMeta::new(*identity,false));
        self
    }

    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn template_instruction_data(&self) -> Vec<u8> {
        let mut template_address_data = Vec::new();
        for data in self.template_address_data.iter() {
            let bytes = data.to_vec();
            let len = bytes.len();
            // log_trace!(" - - - > processing address data len: {}", len);
            // TODO remove address check
            assert!(len < 0xff);
            // log_trace!("{}:{}",style("TPL DATA PACKET LEN:").white().on_red(),len);
            template_address_data.push(len as u8);
            // log_trace!("*** PACKAGING LENGTH: {:?}", len as u8);
            // log_trace!("*** PACKAGING BYTES: {:?}", bytes);
            template_address_data.extend(bytes);
        }
        template_address_data
    }
    
    pub fn instruction_data(&self) -> Vec<u8> {

        let mut data = Vec::new();
        data.extend(&self.template_instruction_data);
        data.extend(&self.handler_instruction_data);
        data
    }
    
    pub fn try_accounts(&self) -> Result<Vec<AccountMeta>> {
        let mut vec = Vec::new();

        if let Some(authority) = &self.authority {
            vec.push(authority.clone());
        }

        if let Some(identity) = &self.identity {
            vec.push(identity.clone());

            if self.authority.is_none() {
                return Err(error!("InstructionBuilder::try_account(): missing authority - required when using identity"));
            }
        }

        vec.extend_from_slice(&self.system_accounts);
        vec.extend_from_slice(&self.token_accounts);
        vec.extend_from_slice(&self.index_accounts);
        vec.extend_from_slice(&self.template_accounts);
        vec.extend_from_slice(&self.handler_accounts);
        Ok(vec)
    }

    pub fn with_account_templates(mut self, n : usize) -> Self {
        for _ in 0..n {
            self.template_access_descriptors.push((IsSigner::NotSigner,Access::Write,SeedSuffix::Sequence))
        }
        self
    }

    pub fn with_account_templates_with_custom_suffixes(mut self, suffixes : &[&str]) -> Self {
        for n in 0..suffixes.len() {
            self.template_access_descriptors.push((IsSigner::NotSigner,Access::Write,SeedSuffix::Custom(suffixes[n].to_string())))
        }
        self
    }

    pub fn with_custom_account_templates(mut self, templates : &[(IsSigner,Access)]) -> Self {
        let template_access_descriptors: Vec<TemplateAccessDescriptor> = 
            templates
                .iter()
                .map(|t|(t.0,t.1,SeedSuffix::Sequence))
                .collect();
        self.template_access_descriptors.extend(template_access_descriptors);
        self
    }

    pub fn with_custom_account_templates_and_seeds(mut self, templates : &[(IsSigner,Access,SeedSuffix)]) -> Self {
        // let template_access_descriptors: Vec<TemplateAccessDescriptor> = 
        //     templates
        //         .iter()
        //         .map(|t|(t.0,t.1,SeedSuffix::Custom(t.2)))
        //         .collect();
        // self.template_access_descriptors.extend(template_access_descriptors);
        self.template_access_descriptors.extend(templates.to_vec());
        self
    }

    pub fn with_sequence(mut self, seq : u64) -> Self {
        assert_eq!(self.track_suffix_seed_seq,None);
        self.suffix_seed_seq = seq;
        self
    }

    pub fn sequence(&self) -> u64 {
        self.suffix_seed_seq
    }

    #[inline(always)]
    pub async fn with_account_aggregator<A>(self, key: &<A as AccountAggregator>::Key, aggregator : &A) -> Result<Self> 
    where A: AccountAggregator
    {

        let list = aggregator.locate_account_pubkeys(key).await?;
        let list : Vec<_> = list.iter().map(|pk| writable(*pk)).collect();
        
        // Ok(self)
        Ok(self.with_index_accounts(&list))
    }


    pub fn seal(mut self) -> Result<Self> {

        if self.is_sealed {
            return Err(error!("InstructionBuilder::seal(): has already been invoked"));
        }
        self.is_sealed = true;

        if self.template_access_descriptors.is_empty() {
            return Ok(self);
        }

        // if we have templates, automatically include system account if not added by the user
        if self.system_accounts.iter().position(|meta| meta.pubkey == solana_sdk::system_program::id()).is_none() {
            self.system_accounts.insert(0, AccountMeta::new(solana_sdk::system_program::id(), false));
        }

        // @seeds
        let user_seed = match &self.identity {
            Some(identity) => identity.pubkey.clone(),
            None => {
                match &self.authority {
                    Some(authority) => authority.pubkey.clone(),
                    None => {
                        return Err(error!("InstructionBuilder::seal(): missing identity and/or authority"))
                    }
                }
            }
        };

        for (is_signer,is_writable, seed_suffix) in self.template_access_descriptors.iter() {
        
            // @seeds
            // let mut seeds = vec![self.program_id.as_ref(), user_seed.as_ref()];
            let mut seeds = vec![user_seed.as_ref()];
            // let mut seeds = vec![];

            let seed_suffix = match seed_suffix {
                SeedSuffix::Blank => { 
                    vec![]
                },
                SeedSuffix::Sequence => {
                    self.suffix_seed_seq += 1;
                    let bytes: [u8; 8] = unsafe { std::mem::transmute(self.suffix_seed_seq.to_le()) };
                    let mut bytes = bytes.to_vec();

                    loop {
                        match bytes[bytes.len()-1] {
                            0 => { bytes.pop(); },
                            _ => break
                        }
                    }

                    bytes
                },
                SeedSuffix::Custom(seed_suffix_str) => {
                    let bytes = seed_suffix_str.as_bytes();
                    // seeds.push(bytes);
                    bytes.to_vec()
                }
            };

            seeds.push(&seed_suffix[..]);

            let (pda, seed_bump) = Pubkey::find_program_address(
                &seeds[..],
                &self.program_id
            );

            let descriptor = match is_writable {
                Access::Write => { AccountMeta::new(pda,(*is_signer).into()) },
                Access::Read => { AccountMeta::new_readonly(pda,(*is_signer).into()) },
            };

            let seed_bump = &[seed_bump];
            seeds.push(seed_bump);            
            seeds.remove(0);

            self.template_accounts.push(descriptor);
            self.template_address_data.push(seeds.concat());
        }

        self.template_instruction_data = self.template_instruction_data();
        
        match &self.track_suffix_seed_seq {
            None => {},
            Some(suffix_seed_seq_refcell) => {
                let mut suffix_seed_seq = suffix_seed_seq_refcell.borrow_mut();
                *suffix_seed_seq = self.suffix_seed_seq;
            }
        }

        Ok(self)
    }

    pub fn templates(&self) -> Vec<AccountMeta> {
        self.template_accounts.to_vec()
    }

}



impl TryFrom<InstructionBuilder> for solana_program::instruction::Instruction {
    type Error = crate::error::Error;

    fn try_from(builder: InstructionBuilder) -> std::result::Result<Self,Self::Error> {

        if !builder.is_sealed {
            return Err(error!("InstructionBuilder is not sealed!"));
        }

        let program_id = builder.program_id();

        let mut instruction_data: Vec<u8> = Vec::new();
        instruction_data.extend(builder.payload().to_vec());
        instruction_data.extend(builder.instruction_data().to_vec());
        let accounts = builder.try_accounts()?;

        let instruction = solana_program::instruction::Instruction {
            program_id,
            data : instruction_data,
            accounts,                
        };

        Ok(instruction)
    }
}
