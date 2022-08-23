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


#[derive(Debug, Clone)]
pub struct FileStore {
    data_folder : PathBuf,
}

impl FileStore {
    pub fn new(data_folder : Option<PathBuf>) -> Self {
        let data_folder = match data_folder {
            Some(data_folder) => data_folder,
            None => {
                let home_dir: PathBuf = home::home_dir().unwrap().into();
                Path::new(&home_dir).join("workflow").join("chaindata")
            }
        };

        FileStore { data_folder }
    }
}

#[async_trait]
impl Store for FileStore {

    async fn list(&self) -> Result<()> { Ok(()) }

    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        let filename = self.data_folder.join(pubkey.to_string());
        if filename.exists().await {
            let data = fs::read(&self.data_folder.join(pubkey.to_string())).await?;
            let account_data = AccountData::try_from_slice(&data)?;
            Ok(Some(Arc::new(AccountDataReference::new(account_data))))
        } else {
            Ok(None)
        }
    }
    async fn store(&self, reference : &Arc<AccountDataReference>) -> Result<()> {
        let data = reference.account_data.read().await.try_to_vec()?;
        fs::write(&self.data_folder.join(reference.key.to_string()),data).await?;
        Ok(())
    }
    async fn purge(&self, pubkey : &Pubkey) -> Result<()> {
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
