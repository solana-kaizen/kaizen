use solana_program::pubkey::Pubkey;
use solana_program::sysvar::slot_history::AccountInfo;
use workflow_log::*;
use workflow_allocator::realloc::account_info_realloc;
use workflow_allocator::result::Result;
use workflow_allocator::error::*;
use workflow_allocator::address::ProgramAddressData;
use workflow_allocator::accounts::*;

pub fn allocate_pda<'info, 'refs, 'payer_info, 'payer_refs, 'pid>(
    payer: &'payer_refs AccountInfo<'payer_info>,
    program_id: &'pid Pubkey,
    base_seed: &[u8],
    tpl_adderss_data: &ProgramAddressData,
    tpl_account_info: &'refs AccountInfo<'info>,
    space: usize,
    lamports: u64,
    validate_pda : bool,
) -> Result<&'refs AccountInfo<'info>> {

    if space > ACCOUNT_DATA_TEMPLATE_SIZE {
        panic!("create_pda() account size is too large (current limit is: {} bytes", ACCOUNT_DATA_TEMPLATE_SIZE);
    }

    // let tpl_data = tpl_account_info.try_borrow_data()?;
    // if tpl_data.len() > 4 {

    // }

    // log_trace!("* * * RECEIVING SEED: {:?}", tpl_adderss_data.seed);
    // let seeds = [user_seed, tpl_adderss_data.seed].concat();
    // let seeds_hex = crate::utils::hex(&seeds[..]);
    // log_trace!("* * * program pda seeds:\n{}\n", seeds_hex);

    if validate_pda {
        match Pubkey::create_program_address(
            &[base_seed, tpl_adderss_data.seed],
            &program_id
        ) {
            Ok(address)=>{
                if address != *tpl_account_info.key {
                    // log_trace!("| pda: PDA ADDRESS MISMATCH {} vs {}", address, tpl_account_info.key);
                    return Err(error_code!(ErrorCode::PDAAddressMatch));
                }

                // log_trace!("| pda: PDA ADDRESS OK");
            },
            Err(_e)=>{
                // log_trace!("| pda: PDA ADDRESS MATCH failure");
                //TODO handle this pubkey error
                return Err(error_code!(ErrorCode::PDAAddressMatch));
            }
        };
    }
    // ---

    // let buffer_size = unsafe {
    //     let ptr = tpl_account_info
    //         .try_borrow_mut_data()
    //         .ok()
    //         .unwrap()
    //         .as_mut_ptr()
    //         .offset(-8) as *mut u64;
    //     *ptr
    // };
    
    // log_trace!("| pda: account realloc - buffer: {} slice: {} target: {}",buffer_size,tpl_account_info.data_len(),space);
    account_info_realloc(tpl_account_info, space, true, true)?;
    // log_trace!("+ pda: simulator realloc done");
    
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
        "\ntransfer_sol:\n\tfrom: {}\n\tto: {}\n\tauthority: {}\n\tamount: {}\n\n",
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

