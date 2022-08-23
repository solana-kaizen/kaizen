use std::*;
use std::str::FromStr;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::Path;
use crate::accounts::AccountData;
use crate::store::MemoryStore;
use solana_program::account_info::{AccountInfo, IntoAccountInfo};
use solana_program::pubkey::Pubkey;
use workflow_log::*;
use crate::result::Result;

pub async fn store_accounts(path_str : &str, store : &MemoryStore) -> Result<()> {
    let mut path = String::from(path_str);
    if path.ends_with('/') { path.pop(); }
    fs::remove_dir_all(&path).ok();//.unwrap();
    fs::create_dir(&path).unwrap();
    let map = &mut store.write().await.map;
    for (pubkey, account_data) in map.iter() {
        let mut account_data = account_data.write().await; //*account_data.clone();
        store_account_data(&path, &(pubkey, &mut *account_data).into_account_info());
    }
    Ok(())
}

pub fn store_account_data<'info>(path : &String, account_info : &AccountInfo<'info>) {

    let filename = format!("{}/{}",path,account_info.key.to_string());
    let path = Path::new(&filename);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    match fs::File::create(&path) {
        Err(why) => {
            log_trace!("couldn't create {}: {}", display, why);
            //return;
        },
        Ok(mut file) => {
            //let j = serde_json::to_string(&data).unwrap();
            //if let Err(error) = file.write_all(j.as_ref()) {

            let data = account_info.data.borrow();
            if let Err(error) = file.write_all(&data[..]) {
                log_trace!("unable to write to {} error: {}",display,error);
            }
        },
    };
}

pub fn load_account_data(path_str : &str) -> AccountData {
    let mut f = fs::File::open(path_str).expect(&format!("file not found: {}", path_str));
    let metadata = fs::metadata(path_str).expect(&format!("unable to read metadata for: {}", path_str));
    //let mut buffer = vec![0; metadata.len() as usize];

    let filename = basename(path_str);
    let key = Pubkey::from_str(&filename).unwrap();
    let owner = Pubkey::new_unique();
    //let key = Pubkey::new_unique();
    let mut account_data = AccountData::new_allocated_for_program(key,owner,metadata.len() as usize);
    f.read(account_data.get_data()).expect(&format!("buffer overflow while reading: {}", filename));
    //f.read(&mut account_data.data[8..]).expect("buffer overflow");

    account_data
}

pub fn basename<'a>(path: &'a str) -> Cow<'a, str> {
    let mut pieces = path.rsplitn(2, |c| c == '/' || c == '\\');
    match pieces.next() {
        Some(p) => p.into(),
        None => path.into(),
    }
}