#[allow(unused_imports)]
use kaizen::error::*;
use kaizen::result::Result;
use solana_program::{
    account_info::AccountInfo, entrypoint::MAX_PERMITTED_DATA_INCREASE, program_memory::sol_memset,
};

// ^ WARNING: This code is lifted from Solana SDK
#[cfg(target_pointer_width = "64")]
pub fn account_info_headers<'info>(account_info: &AccountInfo<'info>) -> Result<(u64, u64)> {
    unsafe {
        // First set new length in the serialized data
        let ptr = account_info.try_borrow_mut_data()?.as_mut_ptr().offset(-8) as *mut u64;
        let serialized_len = *ptr;

        // Then set the new length in the local slice
        let ptr = &mut *(((account_info.data.as_ptr() as *const u64).offset(1) as u64) as *mut u64);
        // *ptr = new_len as u64;
        let slice_len = *ptr;

        Ok((serialized_len, slice_len))
    }
}

#[cfg(target_pointer_width = "64")]
pub fn account_info_realloc<'info>(
    account_info: &AccountInfo<'info>,
    new_len: usize,
    zero_init: bool,
    is_alloc: bool,
) -> Result<()> {
    let orig_len = account_info.data_len();

    if is_alloc == false {
        if new_len > orig_len && new_len - orig_len > MAX_PERMITTED_DATA_INCREASE {
            #[cfg(target_os = "solana")]
            return Err(error_code!(ErrorCode::MaxPermittedAccountDataIncrease));
            #[cfg(not(target_os = "solana"))]
            panic!(
                "maximum permitted account data increase - orig len: {} new len: {}",
                orig_len, new_len
            );
        }
    }

    unsafe {
        // First set new length in the serialized data
        let ptr = account_info.try_borrow_mut_data()?.as_mut_ptr().offset(-8) as *mut u64;
        *ptr = new_len as u64;

        // Then set the new length in the local slice
        let ptr = &mut *(((account_info.data.as_ptr() as *const u64).offset(1) as u64) as *mut u64);
        *ptr = new_len as u64;
    }

    // zero-init if requested
    if zero_init && new_len > orig_len {
        sol_memset(
            &mut account_info.try_borrow_mut_data()?[orig_len..],
            0,
            new_len.saturating_sub(orig_len),
        );
    }

    Ok(())
}

#[cfg(target_pointer_width = "32")]
pub fn account_info_headers<'info>(account_info: &AccountInfo<'info>) -> Result<(u64, u64)> {
    unsafe {
        // First set new length in the serialized data
        let ptr = account_info.try_borrow_mut_data()?.as_mut_ptr().offset(-4) as *mut u32;
        let serialized_len = *ptr;

        // Then set the new length in the local slice
        let ptr = &mut *(((account_info.data.as_ptr() as *const u32).offset(1) as u32) as *mut u32);
        // *ptr = new_len as u64;
        let slice_len = *ptr;

        Ok((serialized_len as u64, slice_len as u64))
    }
}

#[cfg(target_pointer_width = "32")]
pub fn account_info_realloc<'info>(
    account_info: &AccountInfo<'info>,
    new_len: usize,
    zero_init: bool,
    is_alloc: bool,
) -> Result<()> {
    let orig_len = account_info.data_len();

    if is_alloc == false {
        if new_len > orig_len && new_len - orig_len > MAX_PERMITTED_DATA_INCREASE {
            #[cfg(target_os = "solana")]
            return Err(error_code!(ErrorCode::MaxPermittedAccountDataIncrease));
            #[cfg(not(target_os = "solana"))]
            panic!(
                "maximum permitted account data increase - orig len: {} new len: {}",
                orig_len, new_len
            );
        }
    }

    unsafe {
        // First set new length in the serialized data
        let ptr = account_info.try_borrow_mut_data()?.as_mut_ptr().offset(-4) as *mut u32;
        *ptr = new_len as u32;

        // Then set the new length in the local slice
        let ptr = &mut *(((account_info.data.as_ptr() as *const u32).offset(1) as u32) as *mut u32);
        *ptr = new_len as u32;
    }

    // zero-init if requested
    if zero_init && new_len > orig_len {
        sol_memset(
            &mut account_info.try_borrow_mut_data()?[orig_len..],
            0,
            new_len.saturating_sub(orig_len),
        );
    }

    Ok(())
}
