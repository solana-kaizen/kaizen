//!
//! Identity client-side helper functions
//! 

use borsh::*;
use kaizen::prelude::*;
use kaizen::result::Result;
use solana_program::pubkey::Pubkey;
use workflow_log::*;

use crate::emulator::Simulator;
use crate::identity::program::*;

pub async fn locate_identity_pubkey(
    transport: &Arc<Transport>,
    program_id: &Pubkey,
    authority: &Pubkey,
) -> Result<Option<Pubkey>> {
    let proxy_pubkey = find_identity_proxy_pubkey(program_id, authority)?;
    if let Some(proxy_ref) = transport.lookup(&proxy_pubkey).await? {
        let mut proxy_account_data = proxy_ref.account_data.lock()?;
        let proxy_account_info = proxy_account_data.into_account_info();
        let proxy = IdentityProxy::try_load(&proxy_account_info)?;
        let identity_pubkey = proxy.meta.borrow().get_identity_pubkey();
        Ok(Some(identity_pubkey))
    } else {
        log_warning!("Identity: missing identity proxy account {}", proxy_pubkey);
        Ok(None)
    }
}

pub async fn load_identity(
    program_id: &Pubkey,
    authority: &Pubkey,
) -> Result<Option<Arc<AccountDataReference>>> {
    let transport = kaizen::transport::Transport::global()?;
    if let Some(identity_pubkey) = locate_identity_pubkey(&transport, program_id, authority).await?
    {
        Ok(transport.lookup(&identity_pubkey).await?)
    } else {
        log_trace!("ERROR: identity pubkey not found!");
        Ok(None)
    }
}

pub async fn reload_identity(
    program_id: &Pubkey,
    authority: &Pubkey,
) -> Result<Option<Arc<AccountDataReference>>> {
    let transport = kaizen::transport::Transport::global()?;
    if let Some(identity_pubkey) = locate_identity_pubkey(&transport, program_id, authority).await?
    {
        Ok(transport.lookup_remote(&identity_pubkey).await?)
    } else {
        log_trace!("ERROR: identity pubkey not found!");
        Ok(None)
    }
}

pub async fn create_identity(
    program_id: &Pubkey,
    authority: &Pubkey,
    interface_id: usize,
    handler_id: usize,
    instructions: Instr,
) -> Result<TransactionList> {
    let instruction_data = instructions.try_to_vec()?;

    let builder = InstructionBuilder::new(program_id, interface_id, handler_id as u16)
        .with_authority(authority)
        .with_generic_account_templates_with_seeds(&[(AddressDomain::Authority, b"proxy")])
        .with_generic_account_templates(1 + instructions.get_collection_count())
        .with_sequence(0u64)
        .with_instruction_data(&instruction_data)
        .seal()?;

    let accounts = builder.gather_accounts(None, None)?;

    let transaction = Transaction::new_with_accounts(
        format!("Creating generic container {}", accounts[0]).as_str(),
        accounts,
        builder.try_into()?,
    );

    Ok(TransactionList::new(vec![transaction]))
}

pub async fn create_identity_for_unit_tests(
    simulator: &Simulator,
    authority: &Pubkey,
    program_id: &Pubkey,
) -> Result<Pubkey> {
    let config = InstructionBuilderConfig::new(*program_id)
        .with_authority(authority)
        .with_sequence(0u64);

    let builder = InstructionBuilder::new_with_config_for_testing(&config)
        .with_generic_account_templates_with_seeds(&[(AddressDomain::Authority, b"proxy")])
        .with_generic_account_templates(2)
        .seal()?;

    let accounts = builder.generic_template_accounts();
    let identity = accounts[1].clone();

    simulator
        .execute_handler(builder, |ctx: &ContextReference| {
            log_trace!("create identity");
            Identity::create(ctx)?;
            Ok(())
        })
        .await?;

    Ok(identity.pubkey)
}
