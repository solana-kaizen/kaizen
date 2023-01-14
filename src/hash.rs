use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct PubkeyHasher {
    state: u64,
}

impl std::hash::Hasher for PubkeyHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.state = unsafe {
            std::mem::transmute::<[u8; 8], u64>(
                bytes[0..8].try_into().expect("pubkey hasher transmute"),
            )
        };
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

impl std::hash::BuildHasher for PubkeyHasher {
    type Hasher = PubkeyHasher;
    fn build_hasher(&self) -> PubkeyHasher {
        PubkeyHasher { state: 0 }
    }
}

impl std::default::Default for PubkeyHasher {
    fn default() -> PubkeyHasher {
        PubkeyHasher { state: 0 }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct PubkeyHashMap<V, K = Pubkey, S = PubkeyHasher>(std::collections::HashMap<K, V, S>)
where
    K: BorshSerialize + BorshDeserialize + Hash + Eq + PartialEq + PartialOrd,
    V: BorshSerialize + BorshDeserialize,
    S: Default + BuildHasher;

impl<V> PubkeyHashMap<V, Pubkey, PubkeyHasher>
where
    V: BorshSerialize + BorshDeserialize,
{
    pub fn new() -> PubkeyHashMap<V> {
        PubkeyHashMap(std::collections::HashMap::with_hasher(
            PubkeyHasher::default(),
        ))
    }

    #[inline]
    pub fn get(&self, k: &Pubkey) -> Option<&V> {
        self.0.get(k)
    }

    #[inline]
    pub fn get_mut(&mut self, k: &Pubkey) -> Option<&mut V> {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn insert(&mut self, k: Pubkey, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    #[inline]
    pub fn remove(&mut self, k: &Pubkey) -> Option<V> {
        self.0.remove(k)
    }
}

impl<V> Deref for PubkeyHashMap<V>
where
    V: BorshSerialize + BorshDeserialize,
{
    type Target = HashMap<Pubkey, V, PubkeyHasher>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for PubkeyHashMap<V>
where
    V: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
