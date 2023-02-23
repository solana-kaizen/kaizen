//!
//! Error data structure used by the Kaizen framework (both in-program and client-side)
//!

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))] {
        use solana_client::client_error::ClientError;
        use solana_client::client_error::ClientErrorKind;
        use solana_client::rpc_request;
        use solana_client::rpc_response;
        use std::ffi::OsString;
    }
}

cfg_if! {
    if #[cfg(not(target_os = "solana"))] {
        use workflow_rpc::error::ServerError;
        use std::sync::PoisonError;
        use std::sync::Arc;
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::ParsePubkeyError;
use solana_program::pubkey::Pubkey;
use solana_program::pubkey::PubkeyError;
use std::array::TryFromSliceError;
use std::cell::{BorrowError, BorrowMutError};
use std::convert::From;
use std::io::Error as IoError;
use std::time::SystemTimeError;
use workflow_log::log_trace;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[repr(u32)]
pub enum ErrorCode {
    ModuleErrorCodeStart = 0xefff,
    FrameworkErrorCodeStart = 0xffff,

    NotImplemented,
    ErrorMessage,
    RootAccess,
    EmulatorAuthorityIsMissing,
    EmulatorInsufficientTransactionFees,
    IdentityMissing,
    IdentityMissingForAlloc,
    IdentityAccess,
    IdentityCollision,
    IdentityMissingForeignAuthority,
    IdentityCollectionMissing,
    IdentityCollectionLoadError,
    SystemProgramAccountMissing,
    PoisonError,
    BorrowError,
    BorrowMutError,
    IoError,
    OsString,
    SystemTimeError,
    ReadOnlyAccess,
    AccessDenied,
    EntryNotFound,
    AuthorityMustSign,
    InsufficientBalance,
    InsufficientAllocBalance,
    InsufficientBalanceForRent,
    InsufficientBalanceForRentSync,
    ContextAccounts,
    AccountOwnership,
    AccountIsBlank,
    AccountIsMissing,
    MaxPermittedAccountDataIncrease,
    TryFromSliceError,

    PDAAddressMatch,
    PDAAddressCreate,
    PADDataBufferSizeAvailable,
    PADDataBufferSizeDescriptor,
    PDAAccountArgumentData,
    PDAAccountArgumentMatch,
    NotEnoughAccountTemplates,
    TplAccountHasData,
    ReallocFailure,
    NonMutableAccountChange,

    AccountCollectionMissingMeta,
    AccountCollectionMetaSegmentSizeTooSmall,
    AccountCollectionCollision,
    AccountCollectionNotFound,
    AccountCollectionInvalidAddress,
    AccountCollectionInvalidContainerType,
    AccountCollectionInvalidAccount,
    AccountCollectionNotLoaded,
    AccountCollectionDataTypeNotFound,
    AccountCollectionAccountNotFound,

    AccountReferenceCollectionMissingMeta,
    AccountReferenceCollectionMetaSegmentSizeTooSmall,
    AccountReferenceCollectionCollision,
    AccountReferenceCollectionNotFound,
    AccountReferenceCollectionNotLoaded,
    AccountReferenceCollectionDataTypeNotFound,
    AccountReferenceCollectionAccountNotFound,
    AccountReferenceCollectionProxyNotFound,

    InvalidProxyContainerType,

    PubkeyCollectionMissingMeta,
    PubkeyCollectionMetaSegmentSizeTooSmall,
    PubkeyCollectionCollision,
    PubkeyCollectionNotFound,
    PubkeyCollectionNotLoaded,
    PubkeyCollectionMissing,
    PubkeyCollectionDataTypeNotFound,
    PubkeyCollectionAccountNotFound,
    PubkeyCollectionRecordNotFound,

    MappedArrayBounds,
    MappedArrayMetaNotBlank,
    MappedArrayRemoveAtError,
    SegmentStoreMagic,
    SegmentStorageSize,
    SegmentStorageBounds,
    SegmentStoreMetaNotBlank,
    SegmentNotResizable,
    SegmentSizeTooLargeForIndexUnitSize,
    AccountSizeTooSmall,
    MappedArraySegmentSizeTooSmall,
    SequenceStoreAccountDataNotBlank,
    SequenceStoreMagic,
    NotEnoughAccounts,
    UnknownContainerType,
    BPTreeUnknownContainerType,
    ContainerTypeMismatch,
    ContainerMetaVersionMismatch,
    ContainerLoadFailureAfterCreation,
    ContainerLoadFailure,
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
    MaxSeedLengthExceeded,
    InvalidSeeds,
    IllegalOwner,
    ParsePubkeyWrongSize,
    ParsePubkeyInvalid,
    // CacheError,
    LookupError,
    LookupErrorSource,
    LookupErrorDestination,

    MissingClient,
    ClientError,
    RpcError,
    ChannelSendError,
    ChannelRecvError,
    DataType,
    TransactionAlreadyCompleted,
    Web3js,
}

#[derive(Debug)]
pub enum Variant {
    ProgramError(ProgramError),
    ErrorCode(ErrorCode),
    BorrowError(BorrowError),
    BorrowMutError(BorrowMutError),
    #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
    ClientError(Arc<ClientError>),
    // IoError(Arc<IoError>),
    IoError(IoError),
    #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
    OsString(OsString),
    #[cfg(not(target_os = "solana"))]
    RpcError(Arc<workflow_rpc::client::error::Error>),
    #[cfg(not(target_os = "solana"))]
    JsValue(String),
    #[cfg(not(target_os = "solana"))]
    JsError(String),
}

impl Clone for Variant {
    fn clone(&self) -> Self {
        match self {
            Variant::ProgramError(e) => Variant::ProgramError(e.clone()),
            Variant::ErrorCode(e) => Variant::ErrorCode(e.clone()),
            Variant::BorrowError(_e) => Variant::ErrorCode(ErrorCode::BorrowError),
            Variant::BorrowMutError(_e) => Variant::ErrorCode(ErrorCode::BorrowMutError),
            #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
            Variant::ClientError(e) => Variant::ClientError(e.clone()),
            Variant::IoError(_e) => Variant::ErrorCode(ErrorCode::IoError),
            // Variant::IoError(e) => Variant::IoError(e.clone()),
            #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
            Variant::OsString(e) => Variant::OsString(e.clone()),
            #[cfg(not(target_os = "solana"))]
            Variant::RpcError(e) => Variant::RpcError(e.clone()),
            #[cfg(not(target_os = "solana"))]
            Variant::JsValue(e) => Variant::JsValue(e.clone()),
            #[cfg(not(target_os = "solana"))]
            Variant::JsError(e) => Variant::JsError(e.clone()),
        }
    }
}

impl Variant {
    pub fn info(&self) -> String {
        match self {
            Variant::ErrorCode(code) => {
                format!("code: {code:?}")
            }
            Variant::ProgramError(program_error) => {
                format!("program error: {program_error:?}")
            }

            #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
            Variant::OsString(os_str) => {
                format!("OsString error: {os_str:?}")
            }
            Variant::IoError(error) => {
                format!("I/O error: {error:?}")
            }
            Variant::BorrowError(error) => {
                format!("borrow error: {error:?}")
            }
            Variant::BorrowMutError(error) => {
                format!("borrow mut error: {error:?}")
            }
            #[cfg(not(target_os = "solana"))]
            Variant::JsValue(js_value) => js_value.to_owned(),
            #[cfg(not(target_os = "solana"))]
            Variant::JsError(js_error) => js_error.to_owned(),
            #[cfg(not(target_os = "solana"))]
            Variant::RpcError(err) => {
                format!("{err:?}")
            }
            #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
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
                        let mut lines: Vec<String> = Vec::new();
                        match err {
                            Some(err) => {
                                lines.push(format!("+ error: {err:?}"));
                            }
                            None => {}
                        };
                        match accounts {
                            Some(accounts) => {
                                for account in accounts {
                                    lines.push(format!("| account: {account:?}"));
                                }
                            }
                            None => {}
                        };
                        match logs {
                            Some(logs) => {
                                lines.push("+".to_string());
                                // lines.push("|".to_string());
                                lines.extend(
                                    logs.iter()
                                        .map(|l| format!("| {}", l.replace("Program log: ", ""))),
                                );
                                lines.push("+".to_string());
                            }
                            None => {}
                        };
                        match units_consumed {
                            Some(units_consumed) => {
                                lines.push(format!("| units consumed: {units_consumed}"));
                            }
                            None => {}
                        };

                        lines.join("\n")
                    }
                    _ => {
                        format!("{client_error:#?}")
                    }
                }
            }
        }
    }
}

// #[derive(Debug)]
#[derive(Clone)]
pub struct Error {
    pub message: Option<String>,
    pub source: Option<Source>,
    pub account: Option<Pubkey>,
    pub variant: Option<Variant>,
}

impl Error {
    pub fn format(&self) -> String {
        let message = self.message.clone().unwrap_or_else(|| "no message".into());

        let account = match self.account {
            None => "n/a".to_string(),
            Some(key) => key.to_string(),
        };

        let source = match &self.source {
            None => "no source".to_string(),
            Some(source) => format!("{}:{}", source.filename, source.line),
        };

        match &self.variant {
            Some(variant) => match variant {
                #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
                Variant::ClientError(_) => {
                    format!("\n+---\n{}\n+---\n", variant.info())
                }
                _ => {
                    format!("\n+---\n|   error: {message}\n|  source: {source}\n| variant: {}\n| account: {account}\n+---\n", 
                            variant.info(),
                        )
                }
            },
            None => {
                format!(
                    "\n+---\n|   error: {message}\n|  source: {source}\n| account: {account}\n+---\n"
                )
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl Default for Error {
    fn default() -> Self {
        Self::new()
    }
}

impl Error {
    pub fn new() -> Error {
        Error {
            message: None,
            source: None,
            account: None,
            variant: None,
        }
    }

    pub fn message(&self) -> String {
        match self.message {
            Some(ref message) => message.clone(),
            None => match &self.variant {
                Some(variant) => variant.info(),
                None => "no message".to_string(),
            },
        }
    }

    pub fn with_variant(mut self, variant: Variant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn with_code(mut self, code: ErrorCode) -> Self {
        #[cfg(target_os = "solana")]
        solana_program::msg!("*** ERROR: {:?} ***", code);

        self.variant = Some(Variant::ErrorCode(code));
        self
    }

    pub fn with_program_code(mut self, code: u32) -> Self {
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
        self.message = Some(message.to_string());
        self
    }

    pub fn with_account(mut self, key: &Pubkey) -> Self {
        self.account = Some(*key);
        self
    }
}

#[cfg(target_arch = "wasm32")]
pub fn parse_js_error(e: wasm_bindgen::JsValue, msg: Option<&str>) -> Error {
    let mut err = match workflow_wasm::utils::try_get_string_from_prop(&e, "message") {
        Ok(msg) => Error::new().with_message(&msg),
        Err(e) => {
            if let Some(msg) = msg {
                Error::new().with_message(&format!("{}, Error:{:?}", msg, e))
            } else {
                Error::new().with_message(&format!("Error:{:?}", e))
            }
        }
    };
    match js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from("error")) {
        Ok(error_obj) => {
            match js_sys::Reflect::get(&error_obj, &wasm_bindgen::JsValue::from("code")) {
                Ok(code) => {
                    err = err.with_variant(Variant::JsValue(format!("{:?}", code)));
                }
                Err(_e) => {
                    //skip code search error
                    //log_trace!("error code not found: {:?}, error:{:?}", _e, e);
                }
            }
        }
        Err(_e) => {
            //skip code search error
        }
    }

    err
}

#[cfg(not(target_os = "solana"))]
impl From<Error> for ServerError {
    fn from(err: Error) -> Self {
        ServerError::Text(err.to_string())
    }
}

impl From<String> for Error {
    fn from(string: String) -> Error {
        Error::new()
            .with_code(ErrorCode::ErrorMessage)
            .with_message(&string)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Error {
        Error::new()
            .with_code(ErrorCode::ErrorMessage)
            .with_message(msg)
    }
}

impl From<ParsePubkeyError> for Error {
    fn from(error: ParsePubkeyError) -> Error {
        let code = match error {
            ParsePubkeyError::WrongSize => ErrorCode::ParsePubkeyWrongSize,
            ParsePubkeyError::Invalid => ErrorCode::ParsePubkeyInvalid,
        };

        Error::new().with_code(code)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(_: TryFromSliceError) -> Error {
        Error::new().with_code(ErrorCode::TryFromSliceError)
    }
}

impl From<PubkeyError> for Error {
    fn from(error: PubkeyError) -> Error {
        let code = match error {
            PubkeyError::MaxSeedLengthExceeded => ErrorCode::MaxSeedLengthExceeded,
            PubkeyError::InvalidSeeds => ErrorCode::InvalidSeeds,
            PubkeyError::IllegalOwner => ErrorCode::IllegalOwner,
        };

        Error::new().with_code(code)
    }
}

impl From<ErrorCode> for Error {
    fn from(error: ErrorCode) -> Error {
        Error::new().with_code(error)
    }
}

impl From<ProgramError> for Error {
    fn from(error: ProgramError) -> Error {
        Error::new().with_program_error(error)
    }
}

#[cfg(not(target_os = "solana"))]
impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Error {
        Error::new()
            .with_code(ErrorCode::PoisonError)
            .with_message(&format!("{error:#?}"))
    }
}

impl From<BorrowError> for Error {
    fn from(error: BorrowError) -> Error {
        Error::new().with_variant(Variant::BorrowError(error))
    }
}

impl From<SystemTimeError> for Error {
    fn from(_error: SystemTimeError) -> Error {
        Error::new().with_code(ErrorCode::SystemTimeError)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::new().with_variant(Variant::IoError(error))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
impl From<OsString> for Error {
    fn from(os_str: OsString) -> Error {
        Error::new().with_variant(Variant::OsString(os_str))
    }
}

impl From<BorrowMutError> for Error {
    fn from(error: BorrowMutError) -> Error {
        Error::new().with_variant(Variant::BorrowMutError(error))
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        format!("{error:?}")
    }
}

#[cfg(not(target_os = "solana"))]
impl From<Error> for wasm_bindgen::JsValue {
    fn from(error: Error) -> wasm_bindgen::JsValue {
        match error.variant {
            Some(Variant::JsValue(js_value)) => wasm_bindgen::JsValue::from_str(&js_value),
            Some(Variant::JsError(js_error)) => wasm_bindgen::JsValue::from_str(&js_error),
            _ => wasm_bindgen::JsValue::from(format!("xxx {error:?}")),
        }
    }
}

#[cfg(not(target_os = "solana"))]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(js_value: wasm_bindgen::JsValue) -> Error {
        Error::new().with_variant(Variant::JsValue(
            js_value
                .as_string()
                .unwrap_or_else(|| format!("{js_value:?}")),
        ))
    }
}

#[cfg(not(target_os = "solana"))]
impl From<Error> for wasm_bindgen::JsError {
    fn from(error: Error) -> wasm_bindgen::JsError {
        match error.variant {
            Some(Variant::JsError(js_error)) => wasm_bindgen::JsError::new(&js_error),
            Some(Variant::JsValue(js_value)) => wasm_bindgen::JsError::new(&js_value),
            _ => wasm_bindgen::JsError::new(&format!("yyy {error:?}")),
            // _ => wasm_bindgen::JsError::new(&error.to_string()),
        }
    }
}

#[cfg(not(target_os = "solana"))]
impl From<wasm_bindgen::JsError> for Error {
    fn from(error: wasm_bindgen::JsError) -> Error {
        let js_value: wasm_bindgen::JsValue = error.into();
        Error::new().with_variant(Variant::JsError(
            js_value
                .as_string()
                .unwrap_or_else(|| format!("{js_value:?}")),
        ))
    }
}

#[cfg(not(target_os = "solana"))]
impl From<workflow_rpc::client::error::Error> for Error {
    fn from(error: workflow_rpc::client::error::Error) -> Error {
        Error::new().with_variant(Variant::RpcError(Arc::new(error)))
    }
}

#[cfg(not(target_os = "solana"))]
impl From<async_std::channel::RecvError> for Error {
    fn from(error: async_std::channel::RecvError) -> Error {
        Error::new()
            .with_code(ErrorCode::ChannelRecvError)
            .with_message(&format!("{error}"))
    }
}

#[cfg(not(target_os = "solana"))]
impl<T> From<async_std::channel::SendError<T>> for Error {
    fn from(error: async_std::channel::SendError<T>) -> Error {
        Error::new()
            .with_code(ErrorCode::ChannelSendError)
            .with_message(&format!("{error}"))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
impl From<ClientError> for Error {
    fn from(error: ClientError) -> Error {
        Error::new().with_variant(Variant::ClientError(Arc::new(error)))
    }
}

#[cfg(not(target_os = "solana"))]
impl From<solana_web3_sys::error::Error> for Error {
    fn from(error: solana_web3_sys::error::Error) -> Error {
        Error::new()
            .with_code(ErrorCode::Web3js)
            .with_message(&error.to_string())
    }
}

impl From<Error> for ProgramError {
    fn from(e: Error) -> ProgramError {
        #[cfg(not(target_os = "solana"))]
        log_trace!("Converting Error to ProgramError\n{}", e);
        match e.variant {
            None => ProgramError::Custom(0),
            Some(variant) => match variant {
                #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
                Variant::OsString(os_str) => {
                    log_trace!("OsString error: {:?}", os_str);
                    ProgramError::Custom(ErrorCode::OsString as u32)
                }
                Variant::IoError(error) => {
                    log_trace!("I/O error: {}", error);
                    ProgramError::Custom(ErrorCode::IoError as u32)
                }
                Variant::BorrowError(_error) => ProgramError::Custom(ErrorCode::BorrowError as u32),
                Variant::BorrowMutError(_error) => {
                    ProgramError::Custom(ErrorCode::BorrowMutError as u32)
                }
                Variant::ErrorCode(error) => ProgramError::Custom(error as u32),
                Variant::ProgramError(error) => error,
                #[cfg(not(target_os = "solana"))]
                Variant::JsValue(_error) => ProgramError::Custom(0),
                #[cfg(not(target_os = "solana"))]
                Variant::JsError(_error) => ProgramError::Custom(0),
                #[cfg(not(target_os = "solana"))]
                Variant::RpcError(_error) => ProgramError::Custom(ErrorCode::RpcError as u32),
                #[cfg(not(any(target_arch = "wasm32", target_os = "solana")))]
                Variant::ClientError(_error) => {
                    panic!("client error in program is not allowed");
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub filename: &'static str,
    pub line: u32,
}

#[macro_export]
macro_rules! error {
        ($($t:tt)*) => (
            kaizen::error::Error::new()
                .with_source(file!(),line!())
                .with_message(&format_args!($($t)*).to_string())
        )
}
pub use error;

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! js_error {
    ($e:expr, $msg:literal) => {
        parse_js_error($e, Some($msg)).with_source(file!(), line!())
    };
}
#[cfg(target_arch = "wasm32")]
pub use js_error;

#[macro_export]
macro_rules! error_code {
    ($code:expr) => {
        kaizen::error::Error::new()
            .with_source(file!(), line!())
            .with_code($code)
    };
}
pub use error_code;

#[macro_export]
macro_rules! program_error_code {
    ($code:expr) => {
        // Into::<solana_program::program_error::ProgramError>::into(
        kaizen::error::Error::new()
            .with_source(file!(), line!())
            .with_program_code($code as u32)
            .into()
    };
}
pub use program_error_code;

#[macro_export]
macro_rules! program_error {
    ($err:expr) => {
        kaizen::error::Error::new()
            .with_source(file!(), line!())
            .with_program_error($err)
    };
}
pub use program_error;
