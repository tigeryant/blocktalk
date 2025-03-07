use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfoResponse {
    pub walletname: String,
    pub walletversion: u32,
    pub balance: f64,
    pub unconfirmed_balance: f64,
    pub immature_balance: f64,
    pub txcount: u32,
    pub keypoololdest: u64,
    pub keypoolsize: u32,
    pub keypoolsize_hd_internal: u32,
    pub paytxfee: f64,
    pub private_keys_enabled: bool,
    pub avoid_reuse: bool,
    pub scanning: bool,
    pub descriptors: bool,
}