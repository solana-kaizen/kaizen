use crate::accounts::AccountDataStore;
use crate::accounts::AccountDescriptor;

use super::*;
use async_std::path::Path;
use async_std::path::PathBuf;
use std::sync::Arc;
use async_std::fs;
use async_trait::async_trait;
use borsh::*;
use workflow_log::log_error;
use workflow_allocator::accounts::AccountData;
use workflow_allocator::result::Result;
use workflow_allocator::cache::Cache;
use workflow_log::*;

#[derive(Clone)]
pub struct FileStore {
    data_folder : PathBuf,
    cache : Option<Arc<Cache>>
}

impl FileStore {

    pub fn default_data_folder() -> PathBuf {
        let home_dir: PathBuf = home::home_dir().unwrap().into();
        Path::new(&home_dir).join("workflow").join("accounts")
    }

    pub fn try_new() -> Result<FileStore> {
        Self::try_new_with_folder_and_cache(None,None)
    }
    pub fn try_new_with_cache(cache : Arc<Cache>) -> Result<FileStore> {
        Self::try_new_with_folder_and_cache(None,Some(cache))
    }
    pub fn try_new_with_folder_and_cache(data_folder : Option<PathBuf>, cache : Option<Arc<Cache>>) -> Result<FileStore> {
        let data_folder = match data_folder {
            Some(data_folder) => data_folder,
            None => {
                Self::default_data_folder()
            }
        };
        log_trace!("init FileStore at {}",data_folder.clone().into_os_string().into_string()?);
        std::fs::create_dir_all(&data_folder)?;

        Ok(FileStore { data_folder, cache })
    }
}

#[async_trait]
impl Store for FileStore {

    async fn list(&self) -> Result<AccountDescriptorList> { 
    
        let mut entries = std::fs::read_dir(&self.data_folder)?
            .map(|res| res.map(|e| e.path()))
            .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;

        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.

        entries.sort();

        let mut account_descriptors = Vec::new();
        for entry in entries {
            let data = fs::read(entry).await?;
            let account_data_store = AccountDataStore::try_from_slice(&data)?;
            let account_data = AccountData::from(&account_data_store);
            let descriptor: AccountDescriptor = account_data.into();
            account_descriptors.push(descriptor);
        }
        
        Ok(AccountDescriptorList::new(account_descriptors)) 
    }

    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        if let Some(cache) = &self.cache {
            if let Ok(Some(reference)) = cache.lookup(pubkey) {

                // {
                //     log_trace!("~~~ lookup ");
                //     let account_data = &reference.account_data.read().await;
                //     trace_hex(&*account_data.data);
                //     log_trace!("~~~ lookup ");
                // }

                return Ok(Some(reference));
            }
        }

        let filename = self.data_folder.join(pubkey.to_string());
        if filename.exists().await {
            let data = fs::read(&self.data_folder.join(pubkey.to_string())).await?;
            let account_data_store = AccountDataStore::try_from_slice(&data)?;
            let account_data = AccountData::from(&account_data_store);

            // log_trace!("~~~ load data {}",account_data.key);
            // let account_data = &reference.account_data.read().await;
            // trace_hex(&account_data.data);
            // log_trace!("~~~ load data");

            let reference = Arc::new(AccountDataReference::new(account_data));

            if let Some(cache) = &self.cache {
                cache.store(&reference)?;
            }

            Ok(Some(reference))
        } else {
            Ok(None)
        }
    }
    async fn store(&self, reference : &Arc<AccountDataReference>) -> Result<()> {

        // {
        //     log_trace!("~~~ store {}",reference.key);
        //     let account_data = &reference.account_data.read().await;
        //     trace_hex(&*account_data.data);
        //     log_trace!("~~~ store");
        // }

        if let Some(cache) = &self.cache {
            cache.store(&reference)?;
        }

        let data = AccountDataStore::from(&*reference.account_data.lock()?);
        let data_vec = data.try_to_vec()?;
        //log_error!("AccountDataStore: {:?}\nVec: {:02x?}", data, data_vec);
        //log_trace!("storing: {}",reference.key);
    
        //color name : black, blue green red cyan magenta yellow    
        /*
        let mut i = 0;
        let mut r = |len:usize|->core::ops::Range<usize>{
            let end = i+len;
            let range = i..end;
            i = end;
            range
        };
        */

        let mut colors = vec![
            ("214", 1),//container type : 1
            ("147", 32),//key : 32
            ("12", 32),//owner pubkey : 32
            ("13", 8),//lamports : 8
            ("14", 4),//data length : 4
            ("5", 4),//container type : 4
            ("37,188,36", 4),//store magic : 4
            ("169", 4),//store version : 4
            ("161", 2),//store payload_len : 2
            ("cyan", 2),//store index_unit_size : 2
            ("blue", 4),//store segments count : 4
            //("0xcc", 2),//store Index.offset : 2/4 ? // use index_unit_size
            //("0xdc", 2),//store Index.size : 2/4 ? // use index_unit_size
            //("0xcc", 2),//store Index.offset : 2/4 ? // use index_unit_size
            //("0xdc", 2),//store Index.size : 2/4 ? // use index_unit_size
            //("0xcc", 2),//store Index.offset : 2/4 ? // use index_unit_size
            //("0xdc", 2),//store Index.size : 2/4 ? // use index_unit_size
        ];

        let data_index = 1+32+32+8+4;
        let seg_count_index = data_index+4+4+4+2+2;
        let segments_count = unsafe { std::mem::transmute::<_, &u32>(data_vec.as_ptr().offset(seg_count_index)) }+0;
        let seg_count_length = 4;
        //log_trace!("segments_count: {segments_count}");
        if segments_count > 0 && segments_count < 100{
            for _ in 0..segments_count{
                colors.push(("0xcc", 2));
                colors.push(("0xdc", 2));
            }
            let mut odd = true;
            for index in 1..segments_count{
                let index_offset = seg_count_index + seg_count_length + (index as isize * 4 );
                let offset = unsafe {
                    std::mem::transmute::<_, &u16>(data_vec.as_ptr()
                    .offset(index_offset))
                }+0;
                let size = unsafe {
                    std::mem::transmute::<_, &u16>(data_vec.as_ptr()
                    .offset(index_offset+2))
                }+0;
                log_trace!("offset: {offset}, size:{size}");
                
                if odd {
                    odd = false;
                    colors.push(("red", size as usize));
                }else{
                    odd = true;
                    colors.push(("green", size as usize));
                }
            }
        }
        
        let view = format_hex_with_colors(&data_vec, colors);

        if let Err(_) = view.try_print(){
            trace_hex(&data_vec);
        }


        fs::write(&self.data_folder.join(reference.key.to_string()), data_vec).await?;
        Ok(())
    }
    async fn purge(&self, pubkey : &Pubkey) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.purge(Some(pubkey))?;
        }

        let filename = self.data_folder.join(pubkey.to_string());
        match fs::remove_file(&filename).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log_error!("unable to remove file '{}': {}", filename.into_os_string().into_string()?, e);
                Ok(())
            }
        }
    }
}
