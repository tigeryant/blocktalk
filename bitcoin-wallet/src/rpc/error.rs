use crate::error::WalletError;
use jsonrpc_core::{Error as RpcError, ErrorCode};

/// Convert wallet errors to RPC errors
pub fn rpc_error_from_wallet_error(e: WalletError) -> RpcError {
    RpcError::new(ErrorCode::InternalError)
}
