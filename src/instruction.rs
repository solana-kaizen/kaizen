use solana_program::pubkey::Pubkey;
use solana_program::instruction::AccountMeta;

#[inline(always)]
pub fn readonly(pubkey : Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(pubkey,false)
}

#[inline(always)]
pub fn writable(pubkey : Pubkey) -> AccountMeta {
    AccountMeta::new(pubkey,false)
}
