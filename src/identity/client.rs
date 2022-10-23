use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;
use solana_program::pubkey::Pubkey;
// use workflow_allocator::container::*;
use borsh::*;
use workflow_log::*;

use crate::emulator::Simulator;
use crate::identity::program::*;

pub async fn locate_identity_pubkey(transport : &Arc<Transport>, program_id : &Pubkey, authority : &Pubkey) -> Result<Option<Pubkey>> {

    let proxy_pubkey = find_identity_proxy_pubkey(program_id, authority)?;
    // log_trace!("proxy_pubkey: {}", proxy_pubkey);
    if let Some(proxy_ref) = transport.lookup(&proxy_pubkey).await? {
        // log_trace!("got proxy account {}", proxy_pubkey);

        let mut proxy_account_data = proxy_ref.account_data.lock()?;
        let proxy_account_info = proxy_account_data.into_account_info();
        let proxy = IdentityProxy::try_load(&proxy_account_info)?;
        let identity_pubkey = proxy.meta.borrow().get_identity_pubkey();
        // log_trace!("got identity pubkey {}", identity_pubkey);

        Ok(Some(identity_pubkey))
    } else {
        log_warning!("Identity: missing identity proxy account {}", proxy_pubkey);
        Ok(None)
    }
    
}

pub async fn load_identity(program_id: &Pubkey, authority: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    let transport = workflow_allocator::transport::Transport::global()?;
    // let authority = transport.get_authority_pubkey()?;
    if let Some(identity_pubkey) = locate_identity_pubkey(&transport, &program_id, &authority).await? {
        Ok(transport.lookup(&identity_pubkey).await?)
    } else {
        log_trace!("ERROR: identity pubkey not found!");
        Ok(None)
    }
}

pub async fn reload_identity(program_id: &Pubkey, authority: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
    let transport = workflow_allocator::transport::Transport::global()?;
    // let authority = transport.get_authority_pubkey()?;
    if let Some(identity_pubkey) = locate_identity_pubkey(&transport, &program_id, &authority).await? {
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
    handler_id : usize,
    instructions : Instr,
) -> Result<Vec<Transaction>> {

    let instruction_data = instructions.try_to_vec()?;

    // let transport = workflow_allocator::transport::Transport::global()?;

    let builder = InstructionBuilder::new(program_id, interface_id, handler_id as u16)
        .with_authority(authority)
        // .with_account_templates_with_custom_suffixes(&["proxy"]) 
        .with_account_templates_with_seeds(&[(AddressDomain::Authority,"proxy")]) 
        .with_account_templates(1 + instructions.get_collection_count())
        .with_sequence(0u64) 
        .with_instruction_data(&instruction_data)
        .seal()?;

    // let instruction : Instruction = builder.try_into()?;

    let container_pubkey = builder.generic_template_account_at(0).pubkey;

    let transaction = Transaction::new_with_accounts(
        format!("Creating generic container {}", container_pubkey).as_str(),
        &[&container_pubkey],
        builder.try_into()?
    );

    Ok(vec![transaction])

}

// pub async fn create_identity(
//     program_id: &Pubkey,
//     authority: &Pubkey,
//     interface_id: usize,
//     handler_id : usize,
//     instructions : Instr,
// ) -> Result<Arc<AccountDataReference>> {

//     let instruction_data = instructions.try_to_vec()?;

//     let transport = workflow_allocator::transport::Transport::global()?;

//     let builder = InstructionBuilder::new(program_id, interface_id, handler_id as u16)
//         .with_authority(authority)
//         // .with_account_templates_with_custom_suffixes(&["proxy"]) 
//         .with_account_templates_with_seeds(&[(AddressDomain::Authority,"proxy")]) 
//         .with_account_templates(1 + instructions.get_collection_count())
//         .with_sequence(0u64) 
//         .with_instruction_data(&instruction_data)
//         .seal()?;

//     let instruction : Instruction = builder.try_into()?;
//     transport.execute(&instruction).await?;

//     let identity = load_identity(program_id).await?;

//     match identity {
//         Some(identity) => Ok(identity),
//         None => Err(workflow_allocator::error!("Error creating identity").into())
//     }

// }

// pub async fn create_collection(
//     program_id: &Pubkey,
//     authority: &Pubkey,
//     identity : &Pubkey,
//     interface_id: usize,
//     handler_id : usize,
// ) -> Result<Vec<Transport>> {

//     let transport = workflow_allocator::transport::Transport::global()?;

//     let builder = InstructionBuilder::new(program_id, interface_id, handler_id as u16)
//         .with_authority(authority)
//         // .with_account_templates_with_custom_suffixes(&["proxy"]) 
//         .with_account_templates_with_seeds(&[(AddressDomain::Authority,"proxy")]) 
//         .with_account_templates(1)
//         .with_sequence(0u64) 
//         .with_instruction_data(&instruction_data)
//         .seal()?;

//     let instruction : Instruction = builder.try_into()?;
//     transport.execute(&instruction).await?;

//     Ok(())
// }

pub async fn create_identity_for_unit_tests(
    // transport : &Arc<Transport>,
    simulator : &Simulator,
    authority : &Pubkey,
    program_id : &Pubkey,

) -> Result<Pubkey> {

    // Identity::min
    // AccountData::new_static_with_size(key,owner, )
    // let emulator = transport.emulator();
    // let simulator = emulator.clone().downcast_arc::<Simulator>().unwrap();

    let config = InstructionBuilderConfig::new(program_id.clone())
        .with_authority(authority)
        .with_sequence(0u64);

    let builder = InstructionBuilder::new_with_config_for_testing(&config)
        .with_account_templates_with_seeds(&[(AddressDomain::Authority,"proxy")])
        .with_account_templates(2)
        .seal()?;

    let accounts = builder.generic_template_accounts();
    // let proxy = accounts[0].clone(); // PDA0
    let identity = accounts[1].clone();


    simulator.execute_handler(builder,|ctx:&ContextReference| {
        log_trace!("create identity");
        Identity::create(ctx)?;
        Ok(())
    }).await?;

    Ok(identity.pubkey)


}
