use solana_program::account_info::AccountInfo;

#[derive(Debug, Copy, Clone)]
pub enum LamportAllocation {
    Lamports(u64),
    Auto,
}

#[derive(Debug, Copy, Clone)]
pub enum AllocationPayer<'info,'refs> {
    Authority,
    Account(&'refs AccountInfo<'info>)
}

#[derive(Debug, Copy, Clone)]
pub enum IsSigner {
    Signer,
    NotSigner
}

impl Into<bool> for IsSigner {
    fn into(self) -> bool {
        match self {
            IsSigner::Signer => true,
            IsSigner::NotSigner => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Access {
    Read,
    Write,
}

impl Into<bool> for Access {
    fn into(self) -> bool {
        match self {
            Access::Write => true,
            Access::Read => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SeedSuffix {
    Blank,
    Sequence,
    Custom(String)
}


// #[derive(Debug, Clone)]
// pub enum AccountDisposition {
//     Storage,
//     Program,
// }

#[cfg(not(target_arch = "bpf"))]
mod client {

    use crate::generate_random_pubkey;

    use super::*;
    // use std::ops::Range;
    use std::sync::{ Arc, Mutex };
    use async_std::sync::RwLock;
    use borsh::{BorshDeserialize, BorshSerialize};
    use std::time::Instant;
    // use atomic_instant::AtomicInstant;
    use solana_program::account_info::IntoAccountInfo;
    use solana_program::account_info;
    use solana_program::clock::Epoch;
    use solana_program::pubkey::Pubkey;
    use solana_program::rent::Rent;
    // use solana_program::*;
    use workflow_log::*;
    // use workflow_allocator::utils::generate_random_pubkey;
    // use workflow_allocator::console::style;
    use workflow_allocator::container::ContainerHeader;
    use workflow_allocator::result::Result;
    // use workflow_allocator::error::*;
    
    const ACCOUNT_DATA_OFFSET: usize = 8;
    const ACCOUNT_DATA_PADDING: usize = 1024;
    pub static ACCOUNT_DATA_TEMPLATE_SIZE: usize = 1024 * 512; //1024 * 1; // 1mb
    
    // pub type AccountDataReference = Arc<async_std::sync::RwLock<AccountData>>;

    // pub struct Inner {
    // }
    
    #[derive(Debug, Clone)]
    pub struct AccountDataReference {
        pub key : Arc<Pubkey>,
        pub timestamp : Arc<Mutex<Instant>>,
        pub container_type : u32,
        pub data_len : usize,
        pub account_data : Arc<RwLock<AccountData>>
    }

    impl AccountDataReference {
        pub fn new(account_data : AccountData) -> Self {
            let key = Arc::new(account_data.key.clone());
            let timestamp = Arc::new(Mutex::new(Instant::now()));
            let data_len = account_data.data.len();
            let container_type = account_data.container_type().unwrap_or(0);

            AccountDataReference {
                key,
                timestamp,
                container_type,
                data_len,
                account_data : Arc::new(RwLock::new(account_data))
            }
        }

        pub async fn lamports(&self) -> u64 {
            self.account_data.read().await.lamports
        }

        pub async fn clone_for_program(&self) -> AccountData {
            self.account_data.read().await.clone_for_program()
        }

        // pub async fn new_static(&self, key : &Pubkey, owner: &Pubkey) -> Arc<AccountDataReference> {
        //     let account_data = AccountData::new_static(key,owner);
        //     Arc::new(AccountDataReference::new(account_data))
        // }
    }


    #[cfg(not(target_arch = "bpf"))]
    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    pub struct AccountData {
        pub key: Pubkey,
        pub owner: Pubkey,
        pub lamports: u64,
        pub data: Vec<u8>,
        pub space: usize,
        pub rent_epoch: Epoch,
        pub executable: bool,
        pub is_signer: bool,
        pub is_writable: bool,
        // pub range: Option<Range<usize>>,
        // pub disposition: AccountDisposition,
    }

    impl Default for AccountData {
        fn default() -> AccountData {
            let key = Pubkey::default();
            let owner = Pubkey::default();
            AccountData::new_static(key, owner)
        }
    }

    impl AccountData {

        // pub fn try_to_vec(&self) -> Vec<u8> {


        pub fn into_account_info<'info>(&'info mut self) -> AccountInfo<'info> {
            AccountInfo::new(
                &self.key,
                self.is_signer,
                self.is_writable,
                &mut self.lamports,
                &mut self.data[ACCOUNT_DATA_OFFSET..],
                // &mut self.data,
                &self.owner,
                self.executable,
                self.rent_epoch
            )
        }

        // pub fn get_container_type(&self) -> Option<u32> {
        pub fn container_type(&self) -> Option<u32> {
            if self.data.len() < ACCOUNT_DATA_OFFSET + 4 || self.space < 4 {
                None
            } else {
                let header = unsafe {
                    std::mem::transmute::<_, &mut ContainerHeader>(
                        // &self.data[SIMULATOR_ACCOUNT_DATA_OFFSET]//.as_ptr()
                        self.data
                            .as_ptr()
                            .offset(ACCOUNT_DATA_OFFSET as isize),
                    )
                };
                Some(header.container_type)
            }
        }

        pub fn info(&self) -> Result<String> {
            let rent = Rent::default();
            let sol = format!("{:>20.10}",crate::utils::lamports_to_sol(self.lamports));
            let minimum_balance = rent.minimum_balance(self.space);
            let (sol, status) = if self.lamports == minimum_balance {
                (style(sol).green(), style("").green())
            } else if self.lamports < minimum_balance {
                (
                    style(sol).red(),
                    style("~").red(),
                )
            } else {
                (style(sol).yellow(), style("").yellow())
            };

            let container_type = self.container_type();
            let (container_type, container_type_name) = match container_type {
                Some(container_type) => {
                    match workflow_allocator::container::registry::lookup(container_type)? {
                        Some(declaration) => {
                            let container_type = format!("0x{:08x}", container_type);
                            (container_type, declaration.name)
                        }
                        None => ("n/a".to_string(), "n/a"),
                    }
                }
                None => ("-".to_string(), "-"),
            };

            let key_str = self.key.to_string();
            let key_str = key_str.as_str();
            let key_str = format!(
                "{}....{}",
                &key_str[0..8],
                &key_str[key_str.len() - 8..key_str.len()]
            );

            let v = format!(
                "{:>20} {:>10} {:<32} space: {:>6} {:>8} SOL {}",
                style(&key_str).yellow(),
                container_type,
                container_type_name,
                style(self.space).cyan(),
                sol,
                status
            );
            Ok(v.into())
        }

        pub fn with_lamports(mut self, lamports: u64) -> Self {
            self.lamports = lamports;
            self
        }

        pub fn new_static(key: Pubkey, owner: Pubkey) -> AccountData {
            // AccountData::new_static_with_size(key, owner, 32)
            AccountData::new_static_with_size(key, owner, 0)
        }

        pub fn new_static_with_size(key: Pubkey, owner: Pubkey, space: usize) -> AccountData {
            let buffer_len = space + ACCOUNT_DATA_OFFSET;
            let mut data = Vec::with_capacity(buffer_len);
            data.resize(buffer_len, 0);

            let size_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
            *size_ptr = space as u64;
            AccountData {
                key,
                owner,
                data,
                space,
                lamports: 0,
                rent_epoch: 0,
                executable: false,
                is_signer: false,
                is_writable: false,
                // range: None,
                // disposition: AccountDisposition::Storage,
            }
        }

        // pub fn new_static_with_range(
        //     key: Pubkey,
        //     owner: Pubkey,
        //     lamports: u64,
        //     data: Vec<u8>,
        //     rent_epoch: u64,
        //     range: Range<usize>,
        // ) -> AccountData {
        //     let space = data.len();
        //     AccountData {
        //         key,
        //         owner,
        //         data,
        //         lamports,
        //         space,
        //         rent_epoch,
        //         executable: false,
        //         is_signer: false,
        //         is_writable: false,
        //         range: Some(range),
        //         disposition: AccountDisposition::Storage,
        //     }
        // }

        pub fn new_static_for_storage(
            key: Pubkey,
            owner: Pubkey,
            lamports: u64,
            data: Vec<u8>,
            rent_epoch: u64,
        ) -> AccountData {
            let space = data.len();
            AccountData {
                key,
                owner,
                data,
                lamports,
                space,
                rent_epoch,
                executable: false,
                is_signer: false,
                is_writable: false,
                // range: None,
                // disposition: AccountDisposition::Storage,
            }
        }

        // Copy a slice of AccountData into a new instance
        // pub fn try_from_range(
        //     src: &AccountData,
        //     offset: usize,
        //     length: usize,
        // ) -> std::result::Result<AccountData, &str> {
        //     if src.data.len() < offset + length {
        //         return Err("AccountData::try_from_range - out of range access");
        //     }
        //     let data = Vec::<u8>::from(&src.data[offset..(offset + length)]);
        //     let mut account_data = AccountData::new_static_with_range(
        //         src.key,
        //         src.owner,
        //         0,
        //         data,
        //         src.rent_epoch,
        //         offset..(offset + length),
        //     );
        //     account_data
        //         .data
        //         .copy_from_slice(&src.data[offset..(offset + length)]);
        //     account_data.lamports = src.lamports;
        //     Ok(account_data)
        // }

        pub fn clone_for_program(&self) -> AccountData {
            let space = self.space; //self.data.len();
            let buffer_len = space + ACCOUNT_DATA_OFFSET + ACCOUNT_DATA_PADDING;
            let mut data = Vec::with_capacity(buffer_len);
            data.resize(buffer_len, 0);

            let size_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
            *size_ptr = space as u64;
            data[ACCOUNT_DATA_OFFSET..ACCOUNT_DATA_OFFSET + space].copy_from_slice(
                &self.data[ACCOUNT_DATA_OFFSET..ACCOUNT_DATA_OFFSET + space],
            );
            AccountData {
                key: self.key,
                owner: self.owner,
                data,
                space,
                lamports: self.lamports,
                rent_epoch: self.rent_epoch,
                executable: self.executable,
                is_signer: self.is_signer,
                is_writable: self.is_writable,
                // range: None,
                // disposition: AccountDisposition::Program,
            }
        }

        pub fn new_template_for_program(key: Pubkey, owner: Pubkey, space: usize) -> AccountData {
            Self::new_allocated_for_program(key,owner,space)
        }

        pub fn new_allocated_for_program(key: Pubkey, owner: Pubkey, space: usize) -> AccountData {
            let buffer_len = space + ACCOUNT_DATA_OFFSET + ACCOUNT_DATA_PADDING;
            let mut data = Vec::with_capacity(buffer_len);
            data.resize(buffer_len, 0);

            let size_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
            *size_ptr = space as u64;
            AccountData {
                key,
                owner,
                data,
                space,
                lamports: 0,
                rent_epoch: 0,
                executable: false,
                is_signer: false,
                is_writable: false,
                // range: None,
                // disposition: AccountDisposition::Program,
            }
        }

        // pub fn new_template_for_program(key: Pubkey, owner: Pubkey, space: usize) -> AccountData {
        //     let buffer_len = space + ACCOUNT_DATA_OFFSET + ACCOUNT_DATA_TEMPLATE_SIZE;
        //     let mut data = Vec::with_capacity(buffer_len);
        //     data.resize(buffer_len, 0);

        //     let size_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
        //     *size_ptr = space as u64;
        //     AccountData {
        //         key,
        //         owner,
        //         data,
        //         space,
        //         lamports: 0,
        //         rent_epoch: 0,
        //         executable: false,
        //         is_signer: false,
        //         is_writable: false,
        //         // range: None,
        //         // disposition: AccountDisposition::Program,
        //     }
        // }

        pub fn clone_from_account_info<'info>(
            account_info: &AccountInfo<'info>,
            // disposition: AccountDisposition,
        ) -> AccountData {
            let lamports: u64 = **account_info.lamports.borrow();
            let src = account_info.data.borrow();
            let space = src.len();
            let buffer_len = src.len();
            // match disposition {
            //     AccountDisposition::Storage => src.len() + ACCOUNT_DATA_OFFSET,
            //     AccountDisposition::Program => {
            //         src.len() + ACCOUNT_DATA_OFFSET + ACCOUNT_DATA_PADDING
            //     }
            // };

            let mut data = Vec::with_capacity(buffer_len);
            data.resize(buffer_len, 0);
            let data_begin = ACCOUNT_DATA_OFFSET;
            let data_end = ACCOUNT_DATA_OFFSET + space;
            data[data_begin..data_end].clone_from_slice(&src[..]);

            let size_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
            *size_ptr = space as u64;

            AccountData {
                key: account_info.key.clone(),
                owner: account_info.owner.clone(),
                rent_epoch: account_info.rent_epoch,
                executable: account_info.executable,
                is_signer: account_info.is_signer,
                is_writable: account_info.is_writable,
                lamports,
                data,
                space,
                // range: None,
                // disposition,
            }
        }

        pub fn get_available_data_len(&self) -> usize {
            self.data.len() - ACCOUNT_DATA_OFFSET
        }

        pub fn get_data(&mut self) -> &mut [u8] {
            &mut self.data[ACCOUNT_DATA_OFFSET..]
        }

    }

    pub fn get_account_info_serialized_data_len(
        account_info: &AccountInfo,
    ) -> std::result::Result<u64, solana_program::program_error::ProgramError> {
        let marker_value = unsafe {
            let ptr = account_info.try_borrow_mut_data()?.as_mut_ptr().offset(-8) as *mut u64;
            *ptr as u64
        };
        Ok(marker_value)
    }

    impl<'info> account_info::Account for AccountData {
        fn get(&mut self) -> (&mut u64, &mut [u8], &Pubkey, bool, u64) {
            let rent_epoch = 0;
            let data_begin = ACCOUNT_DATA_OFFSET;
            let data_end = ACCOUNT_DATA_OFFSET + self.space as usize;
            (
                &mut self.lamports,
                &mut self.data[data_begin..data_end],
                &self.owner,
                false,
                rent_epoch,
            )
        }
    }

    pub struct MockAccountDataInstance {
        pub key: Pubkey,
        account_data: AccountData,
    }

    impl MockAccountDataInstance {
        pub fn new(space: usize) -> MockAccountDataInstance {
            let key = generate_random_pubkey();
            let owner = generate_random_pubkey();
            MockAccountDataInstance {
                key,
                account_data: AccountData::new_allocated_for_program(key, owner, space),
            }
        }

        pub fn into_account_info(&mut self) -> AccountInfo {
            (&self.key, &mut self.account_data).into_account_info()
        }
    }
}

#[cfg(not(target_arch = "bpf"))]
pub use client::*;

