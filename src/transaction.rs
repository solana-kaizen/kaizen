use solana_program::pubkey::Pubkey;
use solana_program::instruction::Instruction;

use workflow_allocator::container::ContainerReference;
use workflow_allocator::result::Result;

use crate::transport::*;

pub struct Transaction {
    pub name : String,
    pub pubkey : Option<Pubkey>,
    pub instruction : Instruction,
}

impl Transaction {
    pub fn new(name: &str, pubkey: Option<Pubkey>, instruction: Instruction) -> Transaction {
        Transaction {
            name : name.into(),
            pubkey,
            instruction,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        let transport = Transport::global()?;
        transport.execute(&self.instruction).await?;
        Ok(())
    }

    pub async fn execute_and_load<'this,T> (&self) -> Result<Option<ContainerReference<'this,T>>> 
    where T: workflow_allocator::container::Container<'this,'this> 
    {
        if self.pubkey.is_none() {
            panic!("Transaction::execute_and_load - missing pubkey");
        }        

        let transport = Transport::global()?;
        transport.execute(&self.instruction).await?;
        load_container_with_transport::<T>(&transport,self.pubkey.as_ref().unwrap()).await
    }
}

