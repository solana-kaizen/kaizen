use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))] {
        use solana_client::client_error::ClientError;
        use solana_client::client_error::ClientErrorKind;
        use solana_client::rpc_request;
        use solana_client::rpc_response;
        use std::ffi::OsString;
    }
}

cfg_if! {
    if #[cfg(not(target_arch = "bpf"))] {
        use std::sync::PoisonError;
    }
}

use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::ParsePubkeyError;
use std::convert::From;
use std::cell::{BorrowError,BorrowMutError};
use workflow_log::log_trace;
use std::io::Error as IoError;

// #[cfg(not(target_arch = "bpf"))]
// use caches::lru::CacheError;
// #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]

// pub use crate::result::Result;

#[derive(Debug, Clone)]
#[repr(u32)]
pub enum ErrorCode {

    NotImplemented,
    ErrorMessage,
    IdentityMissing,
    IdentityAccess,
    IdentityMissingForeignAuthority,
    PoisonError,
    BorrowError,
    BorrowMutError,
    IoError,
    OsString,
    ReadOnlyAccess,
    EntryNotFound,
    AuthorityMustSign,
    InsufficientBalance,
    InsufficientAllocBalance,
    InsufficientBalanceForRentSync,
    ContextAccounts,
    AccountOwnership,
    AccountIsBlank,
    MaxPermittedAccountDataIncrease,

    PDAAddressMatch,
    PADDataBufferSizeAvailable,
    PADDataBufferSizeDescriptor,
    PDAAccountArgumentData,
    PDAAccountArgumentMatch,
    NotEnoughAccountTemplates,
    ReallocFailure,

    LinearStorageBounds,
    LinearStoreMetaNotBlank,
    LinearStoreRemoveAtError,
    SegmentStoreMagic,
    SegmentStorageSize,
    SegmentStorageBounds,
    SegmentStoreMetaNotBlank,
    SegmentNotResizable,
    SegmentSizeTooLargeForIndexUnitSize,
    AccountSizeTooSmall,
    LinearStorageSegmentSizeTooSmall,
    SequenceStoreAccountDataNotBlank,
    SequenceStoreMagic,
    NotEnoughAccounts,
    BPTreeUnknownContainerType,
    ContainerTypeMismatch,
    ContainerMetaVersionMismatch,
    BPTreePathEmpty,
    BPTreeCollision,
    BPTreeIndexCellCollision,
    BPTreeIndexIsEmpty,
    BPTreeIndexDereference,
    BPTreeValuesDereference,
    BPTreeIndexNotFound,
    BPTreeCyclicAbort,
    BPTreePathError,
    BPTreeNoSuchRecord,
    ParsePubkeyWrongSize,
    ParsePubkeyInvalid,
    // CacheError,
    LookupError,

    MissingClient,
    ClientError,

    // WebSocketEncoding,
    // WebSocketDataType,
    // WebSocketState,
    // WebSocketNotConnected,
    // WebSocketAlreadyConnected,

    ModuleErrorCodeStart = 0xefff,
    ProgramErrorCodeStart = 0xffff
}



#[derive(Debug)]
pub enum Variant {
    ProgramError(ProgramError),
    // FrameworkError(ErrorCode),
    ErrorCode(ErrorCode),
    // PoisonError(PoisonError),
    BorrowError(BorrowError),
    BorrowMutError(BorrowMutError),

    #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
    ClientError(ClientError),

    // #[cfg(not(target_arch = "bpf"))]
    // CacheError(CacheError),
    
    IoError(IoError),

    #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
    OsString(OsString),
    
    // #[cfg(target_arch = "wasm32")]
    #[cfg(not(target_arch = "bpf"))]
    // #[cfg(not(target_arch = "bpf"))]
    JsValue(wasm_bindgen::JsValue)
}

impl Clone for Variant {
    fn clone(&self) -> Self {

        match self {
            Variant::BorrowError(_) => { Variant::ErrorCode(ErrorCode::BorrowError) },
            Variant::BorrowMutError(_) => { Variant::ErrorCode(ErrorCode::BorrowMutError) },
            #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
            Variant::ClientError(_) => { Variant::ErrorCode(ErrorCode::ClientError) },
            v => v.clone()
        }
    }
}

impl Variant {
    pub fn info(&self) -> String {
        match self {
            Variant::ErrorCode(code) => {
                format!("code: {:?}", code)
            },
            Variant::ProgramError(program_error) => {
                format!("program error: {:?}", program_error)

            },
            // Variant::PoisonError(error) => {
            //     format!("poison error: {:?}", error)

            // },

            // #[cfg(not(target_arch = "bpf"))]
            // Variant::CacheError(error) => {
            //     format!("cache error: {:?}", error)
            // },
            #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
            Variant::OsString(os_str) => {
                format!("OsString error: {:?}", os_str)
            },
            Variant::IoError(error) => {
                format!("I/O error: {:?}", error)
            },
            Variant::BorrowError(error) => {
                format!("borrow error: {:?}", error)

            },
            Variant::BorrowMutError(error) => {
                format!("borrow mut error: {:?}",error)
            },
            // #[cfg(target_arch = "wasm32")]
            #[cfg(not(target_arch = "bpf"))]
            Variant::JsValue(js_value) => {
                format!("{:?}",js_value)
            }
            #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
            Variant::ClientError(client_error) => {
                
                match client_error.kind() {
                    ClientErrorKind::RpcError(rpc_request::RpcError::RpcResponseError {
                        data:
                            rpc_request::RpcResponseErrorData::SendTransactionPreflightFailure(
                                rpc_response::RpcSimulateTransactionResult {
                                    err,
                                    logs,
                                    accounts,
                                    units_consumed,
                                    ..
                                },
                                // ..
                            ),
                        ..
                    }) => {

                        let mut lines : Vec<String> = Vec::new(); 
                        match err {
                            Some(err) => {
                                lines.push(format!("+ error: {:?}", err));
                            },
                            None => { }
                        };
                        match accounts {
                            Some(accounts) => {
                                for n in 0..accounts.len() {
                                    lines.push(format!("| account: {:?}", accounts[n]));
                                }
                            },
                            None => { }
                        };                            
                        match logs {
                            Some(logs) => {
                                lines.push("+".to_string());
                                // lines.push("|".to_string());
                                lines.extend(logs.iter().map(|l|{format!("| {}", l.replace("Program log: ", ""))}));
                                lines.push("+".to_string());
                            },
                            None => { }
                        };
                        match units_consumed {
                            Some(units_consumed) => {
                                lines.push(format!("| units consumed: {}", units_consumed));
                            },
                            None => { }
                        };

                        format!("{}", lines.join("\n"))
                    },
                    _ => {
                        format!("{:#?}", client_error)
                    }
                }
            }
        }
    }
}


// #[derive(Debug)]
#[derive(Clone)]
pub struct Error {
    // pub name: String,
    // pub code: Option<u32>,
    pub message: Option<String>,
    pub source: Option<Source>,
    pub account: Option<Pubkey>,
    // pub container: Option<(u32 /* container type */, Pubkey)>,
    pub variant : Option<Variant>,
    // pub context: Option<Rc<Context<'info,'refs,'pid,'instr>>>,
}

// impl Clone for Error {
//     fn clone(&self) -> Self {
//         Error {
//             message : self.message.clone(),
//             source : self.source.clone(),
//             account: self.account.clone(),
//             variant : self.variant.clone(),
//         }
//     }
// }

impl Error {
    pub fn format(&self) -> String {
        let message = self.message.clone().unwrap_or("no message".into());
        
        let account = match self.account {
            None => "n/a".into(),
            Some(key) => { key.to_string() }
        };
        // .clone().unwrap_or("n/a".into());

        let source = match &self.source {
            None => format!("no source"),
            Some(source) => format!("{}:{}", source.filename,source.line),
        };

        match &self.variant {
            Some(variant) => {
                match variant {
                    #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
                    Variant::ClientError(_) => {
                        format!("\n+---\n{}\n+---\n", variant.info())
                    },
                    _ => {
                        format!("\n+---\n|   error: {}\n| source: {}\n| variant: {}\n| account: {}\n+---\n", 
                            message,
                            source,
                            variant.info(),
                            account
                        )
                    }
                }
            },
            None => {
                format!("\n+---\n|   error: {}\n| source: {}\n| account: {}\n+---\n", 
                    message,
                    source,
                    account
                )
            }
        }

    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.format())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.format())
    }
}



impl Error {
    pub fn new() -> Error {
        Error {
            // code : None,
            message : None,
            source : None,
            account : None,
            variant : None,
            // context : None,
        }
    }

    // pub fn new_with_program_code(code : u32) -> Error {
    //     Error {
    //         message : None,
    //         source : None,
    //         account : None,
    //         variant : None,
    //     }
    // }

    pub fn message(&self) -> String {
        match self.message {
            Some(ref message) => message.clone(),
            None => {
                match &self.variant {
                    Some(variant) => variant.info(),
                    None => format!("no message")
                }
            }
        }
    }

    // pub fn with_code(mut self, code: u32) -> Self {
    //     // self.code = Some(code);
    //     self.error = Some(Variant::)
    //     self
    // }
    
    pub fn with_variant(mut self, variant : Variant) -> Self {
        self.variant = Some(variant);
        self
    }
    
    // pub fn with_framework_code(mut self, code : ErrorCode) -> Self {
    //     self.variant = Some(Variant::FrameworkError(code));
    //     self
    // }

    
    pub fn with_code(mut self, code : ErrorCode) -> Self {
        #[cfg(target_arch = "bpf")]
        solana_program::msg!("*** ERROR: {:?} ***", code);

        self.variant = Some(Variant::ErrorCode(code));
        self
    }

    pub fn with_program_code(mut self, code : u32) -> Self {
        self.variant = Some(Variant::ProgramError(ProgramError::Custom(code)));
        self
    }

    pub fn with_program_error(mut self, program_error: ProgramError) -> Self {
        self.variant = Some(Variant::ProgramError(program_error));
        self
    }

    pub fn with_source(mut self, filename: &'static str, line: u32) -> Self {
        self.source = Some(Source { filename, line });
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        // #[cfg(not(target_arch = "bpf"))] {
            self.message = Some(message.to_string());
        // }
        self
    }

    pub fn with_account(mut self, key : &Pubkey) -> Self {
        self.account = Some(key.clone());
        self
    }

    // pub fn with_account(mut self, container_type : u32, key : Pubkey) -> Self {
    //     self.container = Some((container_type, key ));
    //     self
    // }

    // pub fn with_context(mut self, ctx : &Rc<Context>) -> Self {
    //     self.context = Some(ctx.clone());
    //     self
    // }

    // pub fn get_transaction_error(&self) -> Option<TransactionError> {
    //     match &self.variant {
    //         Some(variant) => {
    //             match variant {
    //                 Variant::ClientError(error) => {
    //                     error.get_transaction_error()
    //                 },
    //                 _ => None
    //             }
    //         },
    //         None => None
    //     }
    // }


}






// #[derive(Debug)]
// pub enum Error {
//     ErrorCode(ErrorCode),
//     BorrowError(BorrowError),
//     BorrowMutError(BorrowMutError),
//     WorkflowError(WorkflowError),
//     ProgramError(ProgramError)
// }

impl From<String> for Error {
    fn from(string: String) -> Error {
        Error::new()
            .with_code(ErrorCode::ErrorMessage)
            .with_message(&string)
        // Error::ErrorCode(error)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Error {
        Error::new()
            .with_code(ErrorCode::ErrorMessage)
            .with_message(&msg)
    }
}


impl From<ParsePubkeyError> for Error {
    fn from(error: ParsePubkeyError) -> Error {
        let code = match error {
            ParsePubkeyError::WrongSize => { ErrorCode::ParsePubkeyWrongSize },
            ParsePubkeyError::Invalid => { ErrorCode::ParsePubkeyInvalid },
        };

        Error::new()
            .with_code(code)
    }
}


impl From<ErrorCode> for Error {
    fn from(error: ErrorCode) -> Error {
        Error::new()
            .with_code(error)
        // Error::ErrorCode(error)
    }
}

impl From<ProgramError> for Error {
    fn from(error: ProgramError) -> Error {
        Error::new()
            .with_program_error(error)
        // Error::ProgramError(error)
    }
}

#[cfg(not(target_arch = "bpf"))]
impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Error {
        Error::new()
            .with_code(ErrorCode::PoisonError)
            .with_message(&format!("{:#?}", error))
            // .with_variant(Variant::PoisonError(error))
    }
}

impl From<BorrowError> for Error {
    fn from(error: BorrowError) -> Error {
        Error::new()
            .with_variant(Variant::BorrowError(error))
    }
}

// #[cfg(not(target_arch = "bpf"))]
// impl From<CacheError> for Error {
//     fn from(error: CacheError) -> Error {
//         Error::new()
//             .with_variant(Variant::CacheError(error))
//     }
// }

// #[cfg(not(target_arch = "bpf"))]
impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::new()
            .with_variant(Variant::IoError(error))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
impl From<OsString> for Error {
    fn from(os_str: OsString) -> Error {
        Error::new()
            .with_variant(Variant::OsString(os_str))
    }
}

impl From<BorrowMutError> for Error {
    fn from(error: BorrowMutError) -> Error {
        Error::new()
            .with_variant(Variant::BorrowMutError(error))
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        format!("{:?}", error)
    }
}

#[cfg(not(target_arch = "bpf"))]
impl From<Error> for wasm_bindgen::JsValue {
    fn from(error: Error) -> wasm_bindgen::JsValue {
        match error.variant {
            Some(Variant::JsValue(js_value)) => js_value,
            _ => {
                wasm_bindgen::JsValue::from(format!("{:?}", error))
            }
        }
    }
}

#[cfg(not(target_arch = "bpf"))]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(error: wasm_bindgen::JsValue) -> Error {
        Error::new()
            .with_variant(Variant::JsValue(error))
        // JsValue::from(format!("{:?}", error))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
impl From<ClientError> for Error {
    fn from(error: ClientError) -> Error {
        Error::new()
            .with_variant(Variant::ClientError(error))
        // JsValue::from(format!("{:?}", error))
    }
}


impl From<Error> for ProgramError {
    fn from(e:Error) -> ProgramError {
        #[cfg(not(target_arch = "bpf"))]
        log_trace!("Converting Error to ProgramError\n{}", e);
        match e.variant {
            None => {
                ProgramError::Custom(0)
            },
            Some(variant) => {
                match variant {
                    // Variant::PoisonError(_error) => {
                    //     // panic!("PoisonError should be converted to ProgramError");
                    //     ProgramError::Custom(ErrorCode::PoisonError as u32)
                    // },
                    // #[cfg(not(target_arch = "bpf"))]
                    // Variant::CacheError(_) => {
                    //     panic!("converting CacheError to ProgramError is not supported")
                    //     // ProgramError::Custom(0)
                    //     // ProgramError::Custom(ErrorCode::BorrowError as u32)
                    // },
                    #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
                    Variant::OsString(os_str) => {
                        log_trace!("OsString error: {:?}",os_str);
                        ProgramError::Custom(ErrorCode::OsString as u32)
                        // panic!("converting IoError to ProgramError is not supported")
                        // ProgramError::Custom(0)
                        // ProgramError::Custom(ErrorCode::BorrowError as u32)
                    },
                    // #[cfg(not(target_arch = "bpf"))]
                    Variant::IoError(error) => {
                        log_trace!("I/O error: {}",error);
                        ProgramError::Custom(ErrorCode::IoError as u32)
                        // panic!("converting IoError to ProgramError is not supported")
                        // ProgramError::Custom(0)
                        // ProgramError::Custom(ErrorCode::BorrowError as u32)
                    },
                    Variant::BorrowError(_error) => {
                        ProgramError::Custom(ErrorCode::BorrowError as u32)
                    },
                    Variant::BorrowMutError(_error) => {
                        ProgramError::Custom(ErrorCode::BorrowMutError as u32)
                    },
                    Variant::ErrorCode(error) => {
                        // #[cfg(target_arch = "bpf")]
                        // log_trace!("*** DETECTED ERROR: {:?} ***", error);
                        
                        ProgramError::Custom(error as u32)
                    },
                    Variant::ProgramError(error) => {
                        error
                    },
                    // #[cfg(target_arch = "wasm32")]
                    #[cfg(not(target_arch = "bpf"))]
                    Variant::JsValue(_error) => {
                        ProgramError::Custom(0)
                    },
                    #[cfg(not(any(target_arch = "wasm32", target_arch = "bpf")))]
                    Variant::ClientError(_error) => {
                        panic!("client error in program is not allowed");
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub filename: &'static str,
    pub line: u32,
}

// #[macro_export]
// macro_rules! source {
//     () => {
//         workflow_allocator::error::Source {
//             filename: file!(),
//             line: line!(),
//         }
//     };
// }


// #[macro_export]
// macro_rules! framework_error_code { 
//     ($code:expr) => ( 
//         Error::new()
//             .with_source(file!(),line!())
//             .with_framework_code($code)
//     )
// }

/*
pub mod client {

    use thiserror::Error;
    use wasm_bindgen::JsValue;
    // use wasm_bindgen::prelude::*;

    // #[wasm_bindgen]
    #[derive(Error, Debug)]
    pub enum Error {
        // #[error("data store disconnected")]
        // Disconnect(#[source] io::Error),
        // #[error("the data for key `{0}` is not available")]
        // Redaction(String),
        // #[error("invalid header (expected {expected:?}, found {found:?})")]
        // InvalidHeader {
        //     expected: String,
        //     found: String,
        // },
        // #[error("JsValue error")]
        // JsValue(JsValue),
        #[error("JsValue error")]
        JsValue(JsValue),
        #[error("Generic error")]
        Generic(String),
        #[error("Cache error")]
        Cache(String),
        // #[error("unknown data store error")]
        // Unknown,
    }

    pub type Result<T> = std::result::Result<T, Error>;


    impl Into<Error> for JsValue {
        fn into(self) -> Error {
            Error::JsValue(self.clone())
        }
    }

    impl Into<Error> for String {
        fn into(self) -> Error {
            Error::Generic(self.clone())
        }
    }

    impl Into<Error> for &str {
        fn into(self) -> Error {
            Error::Generic(self.to_string())
        }
    }
}
*/



#[macro_export]
macro_rules! error {
        ($($t:tt)*) => ( 
            workflow_allocator::error::Error::new()
                .with_source(file!(),line!())
                .with_message(&format_args!($($t)*).to_string()) 
        )
}
pub use error;

#[macro_export]
macro_rules! error_code {
    ($code:expr) => (
        workflow_allocator::error::Error::new()
            .with_source(file!(),line!())
            .with_code($code)
    )
}
pub use error_code;

#[macro_export]
macro_rules! program_error_code {
    ($code:expr) => ( 
        // #[cfg(target_arch = "bpf")]
        // solana_program::msg!("Error: {:?}", $code);
        workflow_allocator::error::Error::new()
            .with_source(file!(),line!())
            .with_program_code($code as u32)
            // .with_message(&format!("{:?}", $code))
    )
}
pub use program_error_code;

#[macro_export]
macro_rules! program_error {
    ($err:expr) => ( 
        workflow_allocator::error::Error::new()
            .with_source(file!(),line!())
            .with_program_error($err)
    )
}
pub use program_error;
