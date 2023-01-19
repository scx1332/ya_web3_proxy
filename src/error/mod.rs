mod bag;
mod custom;
mod wrapped;

pub use allowance::AllowanceRequest;
pub use bag::ErrorBag;
pub use custom::{CustomError, TransactionFailedError};
pub use wrapped::Web3ProxyError;

mod allowance;
/// Export macros for creating errors
mod macros;
