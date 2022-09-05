use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub fn shorten_pubkey(pubkey: &Pubkey) -> String {
    let key_str = pubkey.to_string();
    let key_str = key_str.as_str();
    let key_str = format!(
        "{}....{}",
        &key_str[0..8],
        &key_str[key_str.len() - 8..key_str.len()]
    );
    key_str
}

pub const LAMPORTS_PER_SOL: u64 = 1000000000;
#[inline(always)]
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}
#[inline(always)]
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL as f64) as u64
}
#[inline(always)]
pub fn u64sol_to_lamports(sol: u64) -> u64 {
    sol * LAMPORTS_PER_SOL
}

#[cfg(not(target_arch = "bpf"))]
pub fn generate_random_pubkey() -> Pubkey {
    Pubkey::new(&rand::random::<[u8; 32]>())
}

#[cfg(target_arch = "bpf")]
pub fn generate_random_pubkey() -> Pubkey {
    Pubkey::new_unique()
}

#[inline(always)]
pub fn fill_buffer_u8(buffer: &mut [u8], v: u8) {
    for ptr in buffer.iter_mut() { *ptr = v }
}

#[inline(always)]
pub fn fill_account_buffer_u8(account: &AccountInfo, range: std::ops::Range<usize>, v: u8) {
    let mut buffer = account.data.borrow_mut();
    fill_buffer_u8(&mut buffer[range],v)
}


pub fn account_buffer_as_struct_ref<'refs,'info, T>(
    account: &'refs AccountInfo<'info>,
    byte_offset: usize,
) -> &'info T {
    let data = account.data.borrow();
    unsafe {
        std::mem::transmute::<_,&T>(
            data.as_ptr().offset(byte_offset as isize),
        )
    }
}

pub fn account_buffer_as_struct_mut<'refs,'info, T>(
    account: &'refs AccountInfo<'info>,
    byte_offset: usize,
) -> &'info mut T {
    let data = account.data.borrow();
    unsafe {
        std::mem::transmute::<_,&mut T>(
            data.as_ptr().offset(byte_offset as isize),
        )
    }
}

pub fn account_buffer_as_slice<'refs, 'info, T>(
    // account: &'refs AccountInfo<'info>,
    account: &'refs AccountInfo<'info>,
    byte_offset: usize,
    elements: usize
) -> &'info [T] {
    let data = account.data.borrow();
    unsafe {
        std::slice::from_raw_parts::<T>(
            std::mem::transmute::<_,*const T>(
                data.as_ptr().offset(byte_offset as isize),
            ),
        elements)
    }
}

pub fn account_buffer_as_slice_mut<'info, T> (
    account: &AccountInfo<'info>,
    byte_offset: usize,
    elements: usize
) -> &'info mut [T] {
    let mut data = account.data.borrow_mut();
    unsafe {
        std::slice::from_raw_parts_mut::<T>(
            std::mem::transmute::<_,*mut T>(
                data.as_mut_ptr().offset(byte_offset as isize),
            ),
        elements)
    }
}

pub trait FromU64 {
    fn from_u64(v: u64) -> Self;
}

macro_rules! impl_from_u64 {
    ($($ty:ty)*) => {
        $(
            impl FromU64 for $ty {
                #[inline]
                fn from_u64(v: u64) -> $ty {
                    v as $ty
                }
            }
        )*
    }
}

impl_from_u64!(u8 u16 u32 u64 usize);


pub trait FromUsize {
    fn from_usize(v: usize) -> Self;
    fn as_usize(v: Self) -> usize;
}

macro_rules! impl_from_usize {
    ($($ty:ty)*) => {
        $(
            impl FromUsize for $ty {
                #[inline]
                fn from_usize(v: usize) -> $ty {
                    v as $ty
                }
                #[inline]
                fn as_usize(v:$ty) -> usize {
                    v as usize
                }

                // fn 
            }
        )*
    }
}

impl_from_usize!(u8 u16 u32 u64 usize);

