// use js_sys::{Array, Object};
use std::str::FromStr;
use crate::{trace, transport::Transport};
// use wasm_bindgen::prelude::*;
use solana_program::pubkey::Pubkey;
// use solana_program::{program_pack::Pack, pubkey::Pubkey};
use metaplex_meta_decoder::{Metadata, borsh, get_metadata_pda };
// use metaplex_meta_decoder::{Metadata, borsh, PREFIX, get_metadata_pda };
use crate::result::Result;
use crate::error::*;
use std::collections::HashMap;
use serde_json;
use serde::{Deserialize, Serialize};
use crate::transport::Interface;

const TOKEN_BYTES:&str = include_str!("../../root/tokens.json");

static mut TOKENS : Option<Tokens> = None;


#[derive(Debug, Deserialize, Serialize)]
pub struct Token{
    #[serde(alias = "a", rename(serialize="a"))]
    pub address:String,
    #[serde(alias = "chainId", rename(serialize="c", deserialize="c"))]
    pub chain_id:usize,
    #[serde(alias = "d", rename(serialize="d"))]
    pub decimals:usize,
    #[serde(alias = "n", rename(serialize="n"))]
    pub name:String,
    #[serde(alias = "s", rename(serialize="s"))]
    pub symbol:String,
    #[serde(alias = "logoURI", rename(serialize="l", deserialize="l"))]
    pub logo_uri:String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags:Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions:Option<HashMap<String, String>>
}

impl std::fmt::Display for Token{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, 
            "Token:({}\t{:10}\t{})",
                self.address, self.symbol, self.name
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tokens{
    #[serde(alias = "tokens")]
    pub list:Vec<Token>
}

impl std::fmt::Display for Tokens{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Tokens:\n")?;
        for token in &self.list{
            write!(f, "{}\n", token)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TokenInfo{
    pub address:Pubkey,
    pub name:String,
    pub symbol:String,
    pub logo_uri:String
}

impl TokenInfo{
    pub fn new(address:Pubkey, name:String, symbol:String, logo_uri:String)->Self{
        Self { address, name, symbol, logo_uri }
    }
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<TokenInfo> for wasm_bindgen::JsValue{
    type Error = Error;
    fn try_from(info: TokenInfo) -> std::result::Result<Self, Self::Error> {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"address".into(), &info.address.to_string().into())?;
        js_sys::Reflect::set(&obj, &"name".into(), &info.name.into())?;
        js_sys::Reflect::set(&obj, &"symbol".into(), &info.symbol.into())?;
        js_sys::Reflect::set(&obj, &"logoURI".into(), &info.logo_uri.into())?;

        Ok(obj.into())
    }
}

pub fn get_tokens_list()->std::result::Result<&'static Tokens, serde_json::Error>{
    let tokens:&Tokens = unsafe{
        if let Some(_tokens) = &TOKENS{
            TOKENS.as_ref().unwrap()
        }else{
            let tokens:Tokens = serde_json::from_str(TOKEN_BYTES)?;
            TOKENS = Some(tokens);
            TOKENS.as_ref().unwrap()
        }
    };

    Ok(tokens)
}

pub fn get_tokens()->Vec<Pubkey>{
    let mut pubkeys:Vec<Pubkey> = Vec::new();

    pubkeys.push(Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap());
    pubkeys.push(Pubkey::from_str("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R").unwrap());
    pubkeys.push(Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap());
    pubkeys.push(Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap());
    pubkeys.push(Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap());
    pubkeys.push(Pubkey::from_str("9MwGzSyuQRqmBHqmYwE6wbP3vzRBj4WWiYxWns3rkR7A").unwrap());
    //BUGuuhPsHpk8YZrL2GctsCtXGneL1gmT5zYb7eMHZDWf
    

    
    //let tokens = get_tokens_list().unwrap();
    //println!("tokens:{}", tokens);

    pubkeys
}


pub async fn get_tokens_info(keys:Vec<Pubkey>)->Result<Vec<TokenInfo>>{
    let mut list:Vec<TokenInfo> = vec![];

    let transport = Transport::global()?;

    
    for pubkey in keys{
        log_trace!("get_metadata_pda for: {:?} .....", pubkey);
        let metakey = get_metadata_pda(pubkey);
        let account_data = match transport.clone().lookup(&metakey).await {
            Ok(account_data) => account_data.unwrap(),//.read().await,
            Err(err) => {
                let error = format!("tokens::get_tokens_info() error in Transport::get_account_data() while fetching {}: err:{:?}", pubkey.to_string(), err);
                //return Err(JsValue::from(error));
                log_trace!("error: {}", error);
                continue;
            }
        };

        let account_data = account_data.write().await;

        //log_trace!("account_data: {:?}", account_data);
        let meta:Metadata = match borsh::BorshDeserialize::deserialize(&mut account_data.data.as_slice()){
            Ok(meta)=>{
                meta
            }
            Err(err)=>{
                return Err(error!("meta_deser:error: {:?}", err).into())
            }
        };

        log_trace!("meta: {:#?}", meta);

        let name = meta.data.name.replace("\x00", "");
        let symbol = meta.data.symbol.replace("\x00", "");
        let uri = meta.data.uri.replace("\x00", "");

        log_trace!("meta.data.name: {}", name);
        log_trace!("meta.data.symbol: {}", symbol);
        log_trace!("meta.data.uri: {}", uri);

        let logo_uri = format!("https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/{}/logo.png", pubkey.to_string());
        list.push(
            TokenInfo::new(
                pubkey,
                name,
                symbol,
                logo_uri
            )
        );
    }

    Ok(list)
}


#[cfg(target_arch = "wasm32")]
pub async fn get_tokens_info_array(keys:Vec<Pubkey>)->Result<js_sys::Array>{

    let infos = get_tokens_info(keys).await?;
    let list = js_sys::Array::new();
    for info in infos{
        match info.try_into(){
            Ok(obj)=>{
                list.push(&obj);
            },
            Err(err)=>{
                log_trace!("tokens::get_tokens_info_array(), error in parsing info:{:?}", err);
            }
        };
        
    }
    
    Ok(list)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_tokens_info_array(keys:Vec<Pubkey>)->Result<Vec<TokenInfo>>{
    Ok(get_tokens_info(keys).await?)
}
