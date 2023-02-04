use solana_program::account_info::AccountInfo;

#[derive(Debug, Copy, Clone)]
pub enum LamportAllocation {
    Lamports(u64),
    Auto,
}

#[derive(Debug, Copy, Clone)]
pub enum AllocationPayer<'info, 'data> {
    Authority,
    Identity,
    Account(&'data AccountInfo<'info>),
}

#[derive(Debug, Copy, Clone)]
pub enum IsSigner {
    Signer,
    NotSigner,
}

impl From<IsSigner> for bool {
    fn from(is_signer: IsSigner) -> Self {
        match is_signer {
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

impl From<Access> for bool {
    fn from(access: Access) -> bool {
        match access {
            Access::Write => true,
            Access::Read => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SeedSuffix {
    Blank,
    Sequence,
    Custom(Vec<u8>),
}

pub type SeedBump = u8;

#[cfg(not(target_os = "solana"))]
mod client {

    use crate::container::Container;
    use crate::generate_random_pubkey;

    use super::*;
    use std::cell::UnsafeCell;
    use std::cmp::Ordering;
    use std::sync::atomic::AtomicBool;
    use std::sync::{
        Arc,
        Mutex,
        MutexGuard,
        // RwLock, RwLockReadGuard
    };
    // use async_std::sync::RwLock;
    use borsh::{BorshDeserialize, BorshSerialize};
    use owning_ref::OwningHandle;
    use serde::{Deserialize, Serialize};
    //use std::time::Instant;
    use kaizen::container::*;
    use kaizen::result::Result;
    use kaizen::time::Instant;
    use solana_program::account_info;
    use solana_program::account_info::IntoAccountInfo;
    use solana_program::clock::Epoch;
    use solana_program::pubkey::Pubkey;
    use solana_program::rent::Rent;
    use workflow_log::*;

    const ACCOUNT_DATA_OFFSET: usize = 8;
    const ACCOUNT_DATA_PADDING: usize = 1024;
    pub static ACCOUNT_DATA_TEMPLATE_SIZE: usize = 1024 * 512; //1024 * 1; // 1mb

    #[derive(
        Copy, Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq,
    )]
    #[repr(u32)]
    pub enum AccountType {
        Container = 0,
        // TODO
        Unknown,
        SplToken,
        SplToken2022,
        MetalplexFT,
        MetalplexNFT,
    }

    #[derive(Debug)]
    pub struct AccountDataReference {
        pub key: Arc<Pubkey>,
        pub timestamp: Arc<Mutex<Instant>>,
        pub container_type: u32,
        pub data_type: AccountType,
        pub data_len: usize,
        pub lock: AtomicBool,
        pub account_data: Arc<Mutex<AccountData>>,
    }

    impl AccountDataReference {
        pub fn new(account_data: AccountData) -> Self {
            let key = Arc::new(account_data.key);
            let timestamp = Arc::new(Mutex::new(Instant::now().unwrap()));
            let data_len = account_data.data.len() - ACCOUNT_DATA_OFFSET;
            let data_type = account_data.data_type;
            let container_type = if data_type == AccountType::Container {
                account_data.container_type().unwrap_or(0)
            } else {
                0
            };

            AccountDataReference {
                key,
                timestamp,
                container_type,
                data_type,
                data_len,
                lock: AtomicBool::new(false),
                account_data: Arc::new(Mutex::new(account_data)),
            }
        }

        pub fn pubkey(&self) -> &Pubkey {
            &self.key
        }

        pub fn container_type(&self) -> u32 {
            self.container_type
        }

        pub fn lamports(&self) -> Result<u64> {
            Ok(self.account_data.lock()?.lamports)
        }

        pub fn set_lamports(&self, lamports: u64) -> Result<()> {
            self.account_data.lock().unwrap().lamports = lamports;
            Ok(())
        }

        pub fn clone_for_program(&self) -> Result<AccountData> {
            Ok(self.account_data.lock()?.clone_for_program())
        }

        pub fn clone_for_storage(&self) -> Result<AccountData> {
            Ok(self.account_data.lock()?.clone_for_storage())
        }

        pub fn replicate(&self) -> Result<Arc<AccountDataReference>> {
            let account_data = self.clone_for_storage()?;
            let replica = AccountDataReference {
                key: self.key.clone(),
                timestamp: self.timestamp.clone(),
                container_type: self.container_type,
                data_type: self.data_type,
                data_len: self.data_len,
                lock: AtomicBool::new(false),
                account_data: Arc::new(Mutex::new(account_data)),
            };
            Ok(Arc::new(replica))
        }

        pub fn try_into_container<'this, T>(
            self: &Arc<Self>,
        ) -> Result<ContainerReference<'this, T>>
        where
            T: kaizen::container::Container<'this, 'this>,
        {
            self.try_into_container_replica::<T>(true)
        }

        pub fn try_into_container_cache<'this, T>(
            self: &Arc<Self>,
        ) -> Result<ContainerReference<'this, T>>
        where
            T: kaizen::container::Container<'this, 'this>,
        {
            self.try_into_container_replica::<T>(false)
        }

        pub fn try_into_container_replica<'this, T>(
            self: &Arc<Self>,
            replicate: bool,
        ) -> Result<ContainerReference<'this, T>>
        where
            T: kaizen::container::Container<'this, 'this>,
        {
            let target = if replicate {
                self.replicate()?
            } else {
                self.clone()
            };

            let account_data_ref_account_data_lock =
                OwningHandle::<
                    Arc<AccountDataReference>,
                    Box<UnsafeCell<MutexGuard<'this, AccountData>>>,
                >::new_with_fn(target, |reference| {
                    Box::new(unsafe {
                        let reference = reference.as_ref().unwrap();
                        UnsafeCell::new(reference.account_data.lock().unwrap())
                    })
                });

            let account_data_guard =
                OwningHandle::<
                    OwningHandle<
                        Arc<AccountDataReference>,
                        Box<UnsafeCell<MutexGuard<'this, AccountData>>>,
                    >,
                    Box<UnsafeCell<&mut AccountData>>,
                >::new_with_fn(account_data_ref_account_data_lock, |cell| {
                    Box::new(unsafe {
                        let cell = cell.as_ref().unwrap();
                        let guard = cell.get().as_mut().unwrap();
                        UnsafeCell::new(&mut *guard)
                    })
                });

            let account_info = OwningHandle::<
                OwningHandle<
                    OwningHandle<
                        Arc<AccountDataReference>,
                        Box<UnsafeCell<MutexGuard<'this, AccountData>>>,
                    >,
                    Box<UnsafeCell<&mut AccountData>>,
                >,
                Box<AccountInfo>,
            >::new_with_fn(account_data_guard, |x| {
                Box::new(unsafe {
                    let cell = x.as_ref().unwrap();
                    let account_data = (*cell).get().as_mut().unwrap();
                    account_data.into_account_info()
                })
            });

            let container_result = OwningHandle::<
                OwningHandle<
                    OwningHandle<
                        OwningHandle<
                            Arc<AccountDataReference>,
                            Box<UnsafeCell<MutexGuard<'this, AccountData>>>,
                        >,
                        Box<UnsafeCell<&'this mut AccountData>>,
                    >,
                    Box<AccountInfo<'this>>,
                >,
                Box<UnsafeCell<Option<Result<<T as Container<'this, 'this>>::T>>>>,
            >::new_with_fn(account_info, |x| {
                Box::new(unsafe {
                    let account_info: &'this AccountInfo<'this> = x.as_ref().unwrap();
                    let t = T::try_load(account_info);
                    UnsafeCell::new(Some(t))
                })
            });

            if unsafe {
                container_result
                    .get()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .is_err()
            } {
                let err = unsafe {
                    container_result
                        .get()
                        .as_mut()
                        .unwrap()
                        .take()
                        .unwrap()
                        .err()
                        .unwrap()
                };
                return Err(err);
            }

            let container = OwningHandle::<
                OwningHandle<
                    OwningHandle<
                        OwningHandle<
                            OwningHandle<
                                Arc<AccountDataReference>,
                                Box<UnsafeCell<MutexGuard<'this, AccountData>>>,
                            >,
                            Box<UnsafeCell<&'this mut AccountData>>,
                        >,
                        Box<AccountInfo<'this>>,
                    >,
                    Box<UnsafeCell<Option<Result<<T as Container<'this, 'this>>::T>>>>,
                >,
                Box<<T as Container<'this, 'this>>::T>,
            >::new_with_fn(container_result, |x| {
                Box::new(unsafe {
                    let cell = x.as_ref().unwrap();
                    let option = cell.get().as_mut().unwrap();
                    let result = option.take().unwrap();
                    result.ok().unwrap()
                })
            });

            Ok(ContainerReference::new(container))
        }

        // pub fn try_load_container_clone<'this,T> (self : &Arc<Self>) -> Result<AccountDataContainer<'this,T>>
        // where T: kaizen::container::Container<'this,'this>
        // {
        //     let account_data = self.clone_for_storage()?;

        //     let cell = UnsafeCell::new(account_data);
        //     let account_info =
        //         OwningHandle::<Box<UnsafeCell<AccountData>>,Box<AccountInfo>>::new_with_fn(Box::new(cell), |x| {
        //             Box::new( unsafe {
        //                 let r = x.as_ref().unwrap();
        //                 let m = r.get().as_mut().unwrap();
        //                 m.into_account_info()
        //             })
        //         });

        //     let container_result =
        //     OwningHandle::<
        //         OwningHandle::<Box<UnsafeCell<AccountData>>,Box<AccountInfo<'this>>>,
        //         Box<UnsafeCell<Option<Result<<T as Container<'this,'this>>::T>>>>
        //     >::new_with_fn(account_info, |x| {
        //         Box::new( unsafe {
        //             let account_info : &'this AccountInfo<'this> = x.as_ref().unwrap();
        //             let t = T::try_load(account_info);
        //             UnsafeCell::new(Some(t))
        //         })
        //     });

        //     if unsafe { container_result.get().as_ref().unwrap().as_ref().unwrap().is_err() } {
        //         let err = unsafe { container_result.get().as_mut().unwrap().take().unwrap().err().unwrap() };
        //         return Err(err);
        //     }

        //     let container =
        //     OwningHandle::<
        //         OwningHandle::<
        //             OwningHandle::<
        //                 Box<UnsafeCell<AccountData>>,
        //                 Box<AccountInfo<'this>>>,
        //             Box<UnsafeCell<Option<Result<<T as Container<'this,'this>>::T>>>>>,
        //         Box<<T as Container<'this,'this>>::T>
        //     >::new_with_fn(container_result, |x| {
        //         Box::new( unsafe {
        //             let cell = x.as_ref().unwrap();
        //             let option = cell.get().as_mut().unwrap();
        //             let result = option.take().unwrap();
        //             result.ok().unwrap()
        //         })
        //     });

        //     Ok(container)
        // }
    }

    impl From<&AccountDataStore> for AccountDataReference {
        fn from(account_data_store: &AccountDataStore) -> Self {
            AccountDataReference::new(AccountData::from(account_data_store))
        }
    }

    #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
    pub struct AccountDataStore {
        pub data_type: AccountType,
        pub key: Pubkey,
        pub owner: Pubkey,
        pub lamports: u64,
        pub data: Vec<u8>,
        pub rent_epoch: Epoch,
        pub executable: bool,
    }

    impl ColoLogTrace for AccountDataStore {
        fn log_data(&self) -> Vec<u8> {
            if let Ok(data) = self.try_to_vec() {
                return data;
            }

            Vec::new()
        }

        fn log_index_length_color<'a>(
            &self,
        ) -> Option<Vec<(color_log::Index, color_log::Length, color_log::Color)>> {
            let header_size = 77; //1 + 32 + 32 + 8 + 4;
            let container_type_size = 4; //4 bytes
            let store_meta_size = std::mem::size_of::<SegmentStoreMeta>();

            let mut index_length_color = vec![
                (0, 1, "8"),                    //container type : 1
                (1, 32, "2"),                   //key : 32
                (33, 32, "3"),                  //owner pubkey : 32
                (65, 8, "4"),                   //lamports : 8
                (73, 4, "6"),                   //data length : 4
                (77, container_type_size, "8"), //container type : 4
                (81, 4, "168"),                 //store magic : 4
                (85, 4, "169"),                 //store version : 4
                (87, 2, "161"),                 //store payload_len : 2
                (89, 2, "cyan"),                //store index_unit_size : 2
                (93, 4, "blue"),                //store segments count : 4
            ];

            let data_offset = 4; //77;//1+32+32+8+4;
            let mut account_data: AccountData = self.into();
            let account_info = account_data.into_account_info();

            if let Ok(store) = SegmentStore::try_load(&account_info, data_offset) {
                let len = store.len();
                //let meta = store.get_meta();
                let mut index_offset = header_size + container_type_size + store_meta_size;
                let mut odd = false;
                for seg_index in 0..len {
                    let info = store.get_index_info_at(seg_index);
                    let offset = info.offset;
                    let size = info.size;
                    //log_trace!("index_offset:{index_offset}, Index{{offset:{offset}, size:{size}}}");
                    index_length_color.push((index_offset, 2, "0xcc"));
                    index_length_color.push((index_offset + 2, 2, "0xdc"));
                    index_offset += 4;

                    if seg_index > 0 {
                        if odd {
                            odd = false;
                            index_length_color.push((header_size + offset, size, "red"));
                        } else {
                            odd = true;
                            index_length_color.push((header_size + offset, size, "green"));
                        }
                    }
                }
            } else {
                //should we return None to make complete hex view as colorless?
            }

            Some(index_length_color)
        }
    }

    impl From<&AccountData> for AccountDataStore {
        fn from(account_data: &AccountData) -> Self {
            Self {
                data_type: account_data.data_type,
                key: account_data.key,
                owner: account_data.owner,
                lamports: account_data.lamports,
                data: account_data.data().to_vec(),
                rent_epoch: account_data.rent_epoch,
                executable: account_data.executable,
            }
        }
    }

    impl From<&AccountDataStore> for AccountData {
        fn from(account_data_store: &AccountDataStore) -> Self {
            let data_len = account_data_store.data.len();
            let buffer_len = data_len + ACCOUNT_DATA_OFFSET;
            // let mut data = Vec::with_capacity(buffer_len);
            // data.resize(buffer_len, 0);
            let mut data = vec![0; buffer_len];
            AccountData::init_data_len(&mut data, data_len);
            data[ACCOUNT_DATA_OFFSET..].copy_from_slice(&account_data_store.data);
            AccountData {
                data_type: account_data_store.data_type,
                key: account_data_store.key,
                owner: account_data_store.owner,
                data,
                lamports: account_data_store.lamports,
                rent_epoch: account_data_store.rent_epoch,
                executable: account_data_store.executable,
                is_signer: false,
                is_writable: false,
            }
        }
    }

    #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
    pub struct AccountDescriptor {
        pub key: Pubkey,
        pub owner: Pubkey,
        pub lamports: u64,
        pub data_len: u64,
        pub rent_epoch: Epoch,
        pub executable: bool,
        pub is_signer: bool,
        pub is_writable: bool,
        pub container_type: Option<u32>,
    }

    impl From<AccountData> for AccountDescriptor {
        fn from(account_data: AccountData) -> Self {
            (&account_data).into()
        }
    }

    impl From<&AccountData> for AccountDescriptor {
        fn from(account_data: &AccountData) -> Self {
            AccountDescriptor {
                key: account_data.key,
                owner: account_data.owner,
                lamports: account_data.lamports,
                data_len: account_data.data_len() as u64,
                rent_epoch: account_data.rent_epoch,
                executable: account_data.executable,
                is_signer: account_data.is_signer,
                is_writable: account_data.is_writable,
                container_type: account_data.container_type(),
            }
        }
    }

    impl AccountDescriptor {
        pub fn info(&self) -> String {
            let rent = Rent::default();
            let sol = format!("{:>20.10}", crate::utils::lamports_to_sol(self.lamports));
            let minimum_balance = rent.minimum_balance(self.data_len as usize);
            let (sol, status) = match self.lamports.cmp(&minimum_balance) {
                Ordering::Equal => (style(sol).green(), style("").green()),
                Ordering::Less => (style(sol).red(), style("~").red()),
                Ordering::Greater => (style(sol).yellow(), style("").yellow()),
            };

            let (container_type, container_type_name) = match self.container_type {
                Some(container_type) => match kaizen::container::registry::lookup(container_type) {
                    Ok(Some(declaration)) => {
                        let container_type = format!("0x{container_type:08x}");
                        (container_type, declaration.name)
                    }
                    _ => ("n/a".to_string(), "n/a"),
                },
                None => match self.key.to_string().as_str() {
                    "11111111111111111111111111111111" => ("-".to_string(), "□ System Program"),
                    "Config1111111111111111111111111111111111111" => ("-".to_string(), "□ Config"),
                    "Stake11111111111111111111111111111111111111" => ("-".to_string(), "□ Stake"),
                    "Vote111111111111111111111111111111111111111" => ("-".to_string(), "□ Vote"),
                    "BPFLoaderUpgradeab1e11111111111111111111111" => {
                        ("-".to_string(), "□ BPFLoaderUpgradeable")
                    }
                    "Ed25519SigVerify111111111111111111111111111" => {
                        ("-".to_string(), "□ Ed25519SigVerify")
                    }
                    "KeccakSecp256k11111111111111111111111111111" => {
                        ("-".to_string(), "□ KeccakSecp256k")
                    }
                    "SysvarC1ock11111111111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Clock")
                    }
                    "SysvarEpochSchedu1e111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Epoch Schedule")
                    }
                    "SysvarFees111111111111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Fees")
                    }
                    "Sysvar1nstructions1111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Instructions")
                    }
                    "SysvarRecentB1ockHashes11111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Recent Block Hashes")
                    }
                    "SysvarRent111111111111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Rent")
                    }
                    "SysvarS1otHashes111111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Slot Hashes")
                    }
                    "SysvarS1otHistory11111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Slot History")
                    }
                    "SysvarStakeHistory1111111111111111111111111" => {
                        ("-".to_string(), "□ Sysvar Stake History")
                    }
                    _ => ("-".to_string(), "-"),
                },
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
                style(self.data_len).cyan(),
                sol,
                status
            );
            v
        }
    }

    #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
    pub struct AccountDescriptorList {
        pub list: Vec<AccountDescriptor>,
    }

    impl AccountDescriptorList {
        pub fn new(list: Vec<AccountDescriptor>) -> AccountDescriptorList {
            AccountDescriptorList { list }
        }

        pub fn to_log(&self) {
            for (seq, descriptor) in self.list.iter().enumerate() {
                log_info!("[store] [{:>8}] {}", seq, descriptor.info());
            }
        }
    }

    #[cfg(not(target_os = "solana"))]
    #[derive(Debug, Clone)]
    pub struct AccountData {
        pub data_type: AccountType,
        pub key: Pubkey,
        pub owner: Pubkey,
        pub lamports: u64,
        pub data: Vec<u8>,
        pub rent_epoch: Epoch,
        pub executable: bool,
        pub is_signer: bool,
        pub is_writable: bool,
        // pub account_type : u64,
    }

    impl Default for AccountData {
        fn default() -> AccountData {
            let key = Pubkey::default();
            let owner = Pubkey::default();
            AccountData::new_static(key, owner)
        }
    }

    impl AccountData {
        pub fn into_account_info(&mut self) -> AccountInfo<'_> {
            AccountInfo::new(
                &self.key,
                self.is_signer,
                self.is_writable,
                &mut self.lamports,
                &mut self.data[ACCOUNT_DATA_OFFSET..],
                &self.owner,
                self.executable,
                self.rent_epoch,
            )
        }

        pub fn container_type(&self) -> Option<u32> {
            if self.data_len() < 4 {
                //|| self.space < 4 {
                None
            } else {
                let header = unsafe {
                    &*self
                        .data
                        .as_ptr()
                        .add(ACCOUNT_DATA_OFFSET)
                        .cast::<ContainerHeader>()
                    // std::mem::transmute::<_, &mut ContainerHeader>(
                    //     self.data.as_ptr().add(ACCOUNT_DATA_OFFSET),
                    // )
                };
                Some(header.container_type)
            }
        }

        pub fn info(&self) -> String {
            let descriptor: AccountDescriptor = self.into();
            descriptor.info()
        }

        pub fn with_lamports(mut self, lamports: u64) -> Self {
            self.lamports = lamports;
            self
        }

        pub fn new_static(key: Pubkey, owner: Pubkey) -> AccountData {
            // AccountData::new_static_with_size(key, owner, 32)
            AccountData::new_static_with_size(key, owner, 0)
        }

        pub fn new_static_with_size(key: Pubkey, owner: Pubkey, data_len: usize) -> AccountData {
            let buffer_len = data_len + ACCOUNT_DATA_OFFSET;
            // let mut data = Vec::with_capacity(buffer_len);
            // data.resize(buffer_len, 0);
            let mut data = vec![0; buffer_len];
            AccountData::init_data_len(&mut data, data_len);
            AccountData {
                data_type: AccountType::Container,
                key,
                owner,
                data,
                lamports: 0,
                rent_epoch: 0,
                executable: false,
                is_signer: false,
                is_writable: false,
            }
        }

        pub fn new_static_with_args(
            key: Pubkey,
            owner: Pubkey,
            lamports: u64,
            src_data: &[u8],
            rent_epoch: u64,
            // data_len: usize
        ) -> AccountData {
            let data_len = src_data.len();
            let buffer_len = data_len + ACCOUNT_DATA_OFFSET;
            // let mut data = Vec::with_capacity(buffer_len);
            // data.resize(buffer_len, 0);
            let mut data = vec![0; buffer_len];
            AccountData::init_data_len(&mut data, data_len);
            data[ACCOUNT_DATA_OFFSET..].copy_from_slice(src_data);

            AccountData {
                data_type: AccountType::Container,
                key,
                owner,
                data,
                lamports,
                rent_epoch,
                executable: false,
                is_signer: false,
                is_writable: false,
            }
        }

        pub fn clone_for_program(&self) -> AccountData {
            // log_trace!("clong_for_program: **********************");
            // trace_hex(&self.data);
            // log_trace!("clong_for_program: **********************");

            let data_len = self.data_len();
            let buffer_len = data_len + ACCOUNT_DATA_OFFSET + ACCOUNT_DATA_PADDING;
            // let mut data = Vec::with_capacity(buffer_len);
            // data.resize(buffer_len, 0);
            let mut data = vec![0; buffer_len];

            AccountData::init_data_len(&mut data, data_len);
            // *size_ptr = space as u64;
            data[ACCOUNT_DATA_OFFSET..ACCOUNT_DATA_OFFSET + data_len]
                .copy_from_slice(&self.data[ACCOUNT_DATA_OFFSET..ACCOUNT_DATA_OFFSET + data_len]);
            AccountData {
                data_type: AccountType::Container,
                key: self.key,
                owner: self.owner,
                data,
                lamports: self.lamports,
                rent_epoch: self.rent_epoch,
                executable: self.executable,
                is_signer: self.is_signer,
                is_writable: self.is_writable,
            }
        }

        pub fn clone_for_storage(&self) -> AccountData {
            let data_len = self.data_len();
            let buffer_len = data_len + ACCOUNT_DATA_OFFSET;
            // let mut data = Vec::with_capacity(buffer_len);
            // data.resize(buffer_len, 0);
            let mut data = vec![0; buffer_len];

            AccountData::init_data_len(&mut data, data_len);
            // *size_ptr = space as u64;
            data[ACCOUNT_DATA_OFFSET..ACCOUNT_DATA_OFFSET + data_len]
                .copy_from_slice(&self.data[ACCOUNT_DATA_OFFSET..ACCOUNT_DATA_OFFSET + data_len]);
            AccountData {
                data_type: AccountType::Container,
                key: self.key,
                owner: self.owner,
                data,
                lamports: self.lamports,
                rent_epoch: self.rent_epoch,
                executable: self.executable,
                is_signer: self.is_signer,
                is_writable: self.is_writable,
            }
        }

        // pub fn new_template_for_program(key: Pubkey, owner: Pubkey, data_len: usize) -> AccountData {
        pub fn new_template_for_program(key: Pubkey, owner: Pubkey) -> AccountData {
            // Self::new_allocated_for_program(key,owner,data_len)
            Self::new_allocated_for_program(key, owner, ACCOUNT_DATA_TEMPLATE_SIZE)
        }

        pub fn new_allocated_for_program(
            key: Pubkey,
            owner: Pubkey,
            data_len: usize,
        ) -> AccountData {
            let buffer_len = data_len + ACCOUNT_DATA_OFFSET + ACCOUNT_DATA_PADDING;
            // let mut data = Vec::with_capacity(buffer_len);
            // data.resize(buffer_len, 0);
            let mut data = vec![0; buffer_len];

            AccountData::init_data_len(&mut data, data_len);
            // *size_ptr = space as u64;
            AccountData {
                data_type: AccountType::Container,
                key,
                owner,
                data,
                lamports: 0,
                rent_epoch: 0,
                executable: false,
                is_signer: false,
                is_writable: false,
            }
        }

        pub fn clone_from_account_info(account_info: &AccountInfo<'_>) -> AccountData {
            let lamports: u64 = **account_info.lamports.borrow();
            let src = account_info.data.borrow();
            let space = src.len();
            let buffer_len = src.len() + ACCOUNT_DATA_OFFSET;

            // let mut data = Vec::with_capacity(buffer_len);
            let mut data = vec![0; buffer_len];
            // data.resize(buffer_len, 0);
            let data_begin = ACCOUNT_DATA_OFFSET;
            let data_end = ACCOUNT_DATA_OFFSET + space;
            data[data_begin..data_end].clone_from_slice(&src[..]);

            let size_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
            *size_ptr = space as u64;

            AccountData {
                data_type: AccountType::Container,
                key: *account_info.key,
                owner: *account_info.owner,
                rent_epoch: account_info.rent_epoch,
                executable: account_info.executable,
                is_signer: account_info.is_signer,
                is_writable: account_info.is_writable,
                lamports,
                data,
            }
        }

        pub fn get_available_data_len(&self) -> usize {
            // pub fn data_len(&self) -> usize {
            self.data.len() - ACCOUNT_DATA_OFFSET
        }

        pub fn data(&self) -> &[u8] {
            &self.data[ACCOUNT_DATA_OFFSET..]
        }

        pub fn data_mut(&mut self) -> &mut [u8] {
            &mut self.data[ACCOUNT_DATA_OFFSET..]
        }

        cfg_if::cfg_if! {
            if #[cfg(target_pointer_width = "64")] {

                pub fn data_len(&self) -> usize {
                    let space : &u64 = unsafe { std::mem::transmute(&self.data[0]) };
                    *space as usize
                }

                pub fn init_data_len(data : &mut [u8], data_len : usize) {
                    let data_len_ptr: &mut u64 = unsafe { std::mem::transmute(&mut data[0]) };
                    *data_len_ptr = data_len as u64;
                }

            } else if #[cfg(target_pointer_width = "32")] {

                pub fn data_len(&self) -> usize {
                    let space : &u32 = unsafe { std::mem::transmute(&self.data[0]) };
                    *space as usize
                }

                pub fn init_data_len(data : &mut Vec<u8>, data_len : usize) {
                    let data_len_ptr: &mut u32 = unsafe { std::mem::transmute(&mut data[0]) };
                    *data_len_ptr = data_len as u32;
                }

            }
        }
    }

    impl account_info::Account for AccountData {
        fn get(&mut self) -> (&mut u64, &mut [u8], &Pubkey, bool, u64) {
            let rent_epoch = 0;
            let data_begin = ACCOUNT_DATA_OFFSET;
            let data_end = ACCOUNT_DATA_OFFSET + self.data_len();
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

#[cfg(not(target_os = "solana"))]
pub use client::*;
