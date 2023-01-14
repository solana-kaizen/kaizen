use crate::address::ProgramAddressData;
use crate::result::Result;
use crate::{error::*, error_code, program_error};
//use anchor_spl::token::{self, Transfer};
use solana_program::account_info::AccountInfo;
use solana_program::program::{invoke, invoke_signed};
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;
use solana_program::{msg, system_instruction};

pub fn allocate_pda<'info, 'refs, 'payer, 'pid>(
    payer: &'payer AccountInfo<'info>,
    program_id: &'pid Pubkey,
    tpl_seeds: &[&[u8]],
    tpl_account_info: &'refs AccountInfo<'info>,
    space: usize,
    lamports: u64,
    validate_pda: bool,
) -> Result<&'refs AccountInfo<'info>> {
    // msg!("| pda: inside solana allocate_pda()");
    // msg!("| pda: executing create_program_address()");
    if validate_pda {
        match Pubkey::create_program_address(tpl_seeds, &program_id) {
            Ok(address) => {
                if address != *tpl_account_info.key {
                    // msg!("| pda: PDA ADDRESS MISMATCH {} vs {}", address, tpl_account_info.key);
                    return Err(error_code!(ErrorCode::PDAAddressMatch));
                }
                // msg!("| pda: PDA ADDRESS OK");
            }
            Err(_e) => {
                // msg!("| pda: PDA ADDRESS MATCH failure");
                //TODO handle this pubkey error
                return Err(error_code!(ErrorCode::PDAAddressMatch));
            }
        };
    }
    // msg!("creating: {:?}", tpl_account_info.key);
    // msg!("payer.key: {:?}", payer.key);
    // msg!("program_id: {:?}", program_id);
    // msg!("system_program id: {:?}", solana_program::system_program::id());
    // msg!("seed: {:?}", tpl_address_data.seed);
    // msg!("seed_suffix: {:?}", seed_suffix);
    // msg!("info.bump: {:?}", tpl_address_data.bump);
    let ins = create_account(
        payer.key,
        tpl_account_info.key,
        lamports,
        space as u64,
        // info.space,
        program_id,
    );
    msg!("system ins : {:?}", ins);
    let result = invoke_signed(
        &ins,
        &[payer.clone(), tpl_account_info.clone()],
        &[tpl_seeds],
    );
    // msg!("invoke_signed:result: {:?}", result);
    match result {
        Ok(_r) => Ok(tpl_account_info),
        Err(e) => {
            // msg!("allocate_pda:AllocatorError");
            return Err(program_error!(e));
        }
    }
}

pub fn allocate_multiple_pda<'info, 'refs, 'payer, 'pid, 'instr>(
    payer: &'payer AccountInfo<'info>,
    program_id: &'pid Pubkey,
    user_seed: &[u8],
    account_templates: &[(&ProgramAddressData, &'refs AccountInfo<'info>)],
    settings: &[(usize, u64)],
) -> Result<Vec<&'refs AccountInfo<'info>>> {
    let mut vec: Vec<&AccountInfo<'info>> = Vec::new();
    for idx in 0..settings.len() {
        let (tpl_address_data, tpl_account_info) = account_templates[idx];
        let (space, lamports) = settings[idx]; // as u64;
                                               //let program_id = ctx.program_id;
                                               // let seed = &tpl_address_data.seed;
                                               // let seed_suffix = &tpl_address_data.seed_suffix;
                                               // let seed_suffix = &seed_suffix_bytes;
                                               //let seed = b"acc-5";
        match Pubkey::create_program_address(&[user_seed, tpl_address_data.seed], &program_id) {
            Ok(address) => {
                if &address != tpl_account_info.key {
                    return Err(error_code!(ErrorCode::PDAAddressMatch));
                }
            }
            Err(_e) => {
                //TODO handle this pubkey error
                return Err(error_code!(ErrorCode::PDAAddressMatch));
            }
        };
        msg!("creating: {:?}", tpl_account_info.key);
        msg!("payer.key: {:?}", payer.key);
        msg!("program_id: {:?}", program_id);
        msg!("seed: {:?}", tpl_address_data.seed);
        // msg!("seed_suffix: {:?}", seed_suffix);
        // msg!("info.bump: {:?}", tpl_address_data.bump);
        let ins = create_account(
            payer.key,
            tpl_account_info.key,
            lamports,
            space as u64,
            // info.space,
            program_id,
        );
        msg!("system ins : {:?}", ins);
        let result = invoke_signed(
            &ins,
            &[payer.clone(), tpl_account_info.clone()],
            // A slice of seed slices, each seed slice being the set
            // of seeds used to generate one of the PDAs required by the
            // callee program, the final seed being a single-element slice
            // containing the `u8` bump seed.
            &[&[
                user_seed,
                tpl_address_data.seed,
                // seed_suffix,
                //payer.key.as_ref(),
                // &[tpl_address_data.bump]
            ]],
        );
        msg!("invoke_signed:result: {:?}", result);
        match result {
            Ok(_r) => {
                vec.push(tpl_account_info); //.clone());
            }
            Err(e) => {
                // msg!("allocate_multiple_pda:AllocatorError");
                return Err(program_error!(e));
            }
        };
    }

    Ok(vec)
}

pub fn transfer_sol<'info>(
    source: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    _authority: &AccountInfo<'info>, // TODO: is this needed?
    system_program_account: &AccountInfo<'info>,
    lamports: u64,
) -> Result<()> {
    msg!(
        "transfer_sol:sol transfering from {} to: {}",
        source.key,
        destination.key
    );
    //let signers : &[&[&[u8]]] = &[];

    let ix = system_instruction::transfer(source.key, destination.key, lamports);

    let result = invoke(
        &ix,
        //signers,
        &[
            source.clone(),
            destination.clone(),
            system_program_account.clone(),
        ],
    );

    match result {
        Ok(res) => {
            msg!("invoke():success: {:?}", res);
        }
        Err(err) => {
            msg!("invoke():err: {:?}", err);
            return Err(err.into());
        }
    }

    Ok(())
}

pub fn transfer_spl<'info>(
    token_program: &AccountInfo<'info>,
    source: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
    signers: &[&[&[u8]]],
) -> Result<()> {
    let ix = spl_token::instruction::transfer(
        token_program.key,
        source.key,
        destination.key,
        authority.key,
        &[&authority.key],
        amount,
    )?;
    invoke_signed(
        &ix,
        &[
            source.clone(),
            destination.clone(),
            authority.clone(),
            token_program.clone(),
        ],
        signers,
    )?;

    Ok(())
}
