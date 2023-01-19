use super::{CustomError, TransactionFailedError};
use crate::error::allowance::AllowanceRequest;
use rustc_hex::FromHexError;
use std::fmt::Display;
use std::num::ParseIntError;
use web3::ethabi::ethereum_types::FromDecStrErr;

/// Enum containing all possible errors used in the library
/// Probably you can use thiserror crate to simplify this process
#[derive(Debug)]
pub enum ErrorBag {
    ParseError(ParseIntError),
    IoError(std::io::Error),
    CustomError(CustomError),
    TransactionFailedError(TransactionFailedError),
    EthAbiError(web3::ethabi::Error),
    Web3Error(web3::Error),
    FromHexError(FromHexError),
    NoAllowanceFound(AllowanceRequest),
    FromDecStrErr(FromDecStrErr),
}

impl Display for ErrorBag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorBag::ParseError(parse_int_error) => write!(f, "{}", parse_int_error),
            ErrorBag::IoError(io_error) => write!(f, "{}", io_error),
            ErrorBag::CustomError(custom_error) => write!(f, "{}", custom_error),
            ErrorBag::TransactionFailedError(transaction_failed_error) => {
                write!(f, "{}", transaction_failed_error)
            }
            ErrorBag::EthAbiError(eth_abi_error) => write!(f, "{:?}", eth_abi_error),
            ErrorBag::Web3Error(web3_error) => write!(f, "{:?}", web3_error),
            ErrorBag::FromHexError(from_hex_error) => write!(f, "{:?}", from_hex_error),
            ErrorBag::NoAllowanceFound(allowance_request) => write!(f, "{:?}", allowance_request),
            ErrorBag::FromDecStrErr(from_dec_str_err) => write!(f, "{:?}", from_dec_str_err),
        }
    }
}

impl std::error::Error for ErrorBag {}

impl From<ParseIntError> for ErrorBag {
    fn from(err: ParseIntError) -> Self {
        ErrorBag::ParseError(err)
    }
}

impl From<std::io::Error> for ErrorBag {
    fn from(err: std::io::Error) -> Self {
        ErrorBag::IoError(err)
    }
}

impl From<CustomError> for ErrorBag {
    fn from(err: CustomError) -> Self {
        ErrorBag::CustomError(err)
    }
}

impl From<TransactionFailedError> for ErrorBag {
    fn from(err: TransactionFailedError) -> Self {
        ErrorBag::TransactionFailedError(err)
    }
}

impl From<web3::ethabi::Error> for ErrorBag {
    fn from(err: web3::ethabi::Error) -> Self {
        ErrorBag::EthAbiError(err)
    }
}

impl From<web3::Error> for ErrorBag {
    fn from(err: web3::Error) -> Self {
        ErrorBag::Web3Error(err)
    }
}

impl From<FromHexError> for ErrorBag {
    fn from(err: FromHexError) -> Self {
        ErrorBag::FromHexError(err)
    }
}

impl From<AllowanceRequest> for ErrorBag {
    fn from(err: AllowanceRequest) -> Self {
        ErrorBag::NoAllowanceFound(err)
    }
}

impl From<FromDecStrErr> for ErrorBag {
    fn from(err: FromDecStrErr) -> Self {
        ErrorBag::FromDecStrErr(err)
    }
}
