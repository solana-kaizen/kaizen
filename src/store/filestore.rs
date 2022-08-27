use crate::accounts::AccountDataStore;

use super::*;
use async_std::path::Path;
use async_std::path::PathBuf;
use std::sync::Arc;
use async_std::fs;
use async_trait::async_trait;
// use std::env ;
use borsh::*;
// use crate::result::Result;
use workflow_log::log_error;
// use workflow_allocator::error::*;
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

    async fn list(&self) -> Result<()> { 
    
        let mut entries = std::fs::read_dir(&self.data_folder)?
            .map(|res| res.map(|e| e.path()))
            .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;

        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.

        entries.sort();

        for entry in entries {
            let data = fs::read(entry).await?;
            let account_data_store = AccountDataStore::try_from_slice(&data)?;
            let account_data = AccountData::from(&account_data_store);
            let info = account_data.info()?;
            log_info!("{}", info);

            // println!("{}", entry.into_os_string().into_string()?);
        }
        
        Ok(()) 
    }

    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {

        if let Some(cache) = &self.cache {
            if let Ok(Some(reference)) = cache.lookup(pubkey).await {
                return Ok(Some(reference));
            }
        }

        let filename = self.data_folder.join(pubkey.to_string());
        if filename.exists().await {
            let data = fs::read(&self.data_folder.join(pubkey.to_string())).await?;
            let account_data_store = AccountDataStore::try_from_slice(&data)?;
            // let account_data = AccountData::try_from_slice(&data)?;
            let account_data = AccountData::from(&account_data_store);
            let reference = Arc::new(AccountDataReference::new(account_data));

            if let Some(cache) = &self.cache {
                cache.store(&reference).await?;
            }

            Ok(Some(reference))
        } else {
            Ok(None)
        }
    }
    async fn store(&self, reference : &Arc<AccountDataReference>) -> Result<()> {
        // log_trace!("storing account: {} size: {} lamports: {}", reference.key, reference.data_len, reference.lamports().await);
        // if reference.data_len == 0 {
        //     log_error!("WARNING - skipping zero size account storage: {}", reference.key);
        //     return Ok(());
        // }
        if let Some(cache) = &self.cache {
            cache.store(&reference).await?;
        }

        let data = AccountDataStore::from(&*reference.account_data.read().await).try_to_vec()?;
        // let data = reference.account_data.read().await.try_to_vec()?;
        fs::write(&self.data_folder.join(reference.key.to_string()),data).await?;
        Ok(())
    }
    async fn purge(&self, pubkey : &Pubkey) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.purge(pubkey).await?;
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
