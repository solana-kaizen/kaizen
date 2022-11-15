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
use kaizen::accounts::AccountData;
use kaizen::result::Result;
use kaizen::cache::Cache;
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

        let data = AccountDataStore::from(&*reference.account_data.lock()?).try_to_vec()?;

        log_trace!("storing: {}",reference.key);
        trace_hex(&data);

        fs::write(&self.data_folder.join(reference.key.to_string()),data).await?;
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
