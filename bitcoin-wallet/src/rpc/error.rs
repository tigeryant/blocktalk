use jsonrpc_core::{Error as RpcError, ErrorCode};
use crate::error::WalletError;

/// Convert wallet errors to RPC errors
pub fn rpc_error_from_wallet_error(e: WalletError) -> RpcError {
    RpcError::new(ErrorCode::InternalError)
}