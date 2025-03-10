use std::future::Ready;
use std::sync::Arc;

use bitcoin::{Address, Amount, Txid};
use jsonrpc_core::{Error as RpcError, IoHandler, Params, Value};
use serde_json::json;
use tokio::task::{self, LocalSet};

use super::error::rpc_error_from_wallet_error;
use crate::wallet::{CreateWalletOptions, WalletInterface};

pub fn register_wallet_methods(io: &mut IoHandler, wallet_interface: Arc<WalletInterface>) {
    register_createwallet(io, wallet_interface.clone());
    register_loadwallet(io, wallet_interface.clone());
    register_getwalletinfo(io, wallet_interface.clone());
    register_getnewaddress(io, wallet_interface.clone());
    register_getbalance(io, wallet_interface.clone());
    register_listunspent(io, wallet_interface.clone());
    register_listtransactions(io, wallet_interface.clone());
    register_gettransaction(io, wallet_interface.clone());
    register_sendtoaddress(io, wallet_interface.clone());
    register_rescanblockchain(io, wallet_interface.clone());
}

fn register_createwallet(io: &mut IoHandler, wallet_interface: Arc<WalletInterface>) {
    io.add_sync_method("createwallet", move |params: Params| {
        let wallet_interface = wallet_interface.clone();
        log::info!("=========================");
        log::info!("Creating wallet...");

        let options = match parse_create_wallet_options(params) {
            Ok(options) => options,
            Err(e) => return Err(e),
        };

        let wallet_name = options.wallet_name.clone();
        match wallet_interface.create_wallet(options) {
            Ok(_) => {
                let result = json!({
                    "name": wallet_name,
                    "warning": ""
                });
                Ok(result)
            }
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_loadwallet(io: &mut IoHandler, wallet_interface: Arc<WalletInterface>) {
    io.add_sync_method("loadwallet", move |params: Params| {
        log::info!("=========================");
        log::info!("Loading wallet...");
        let wallet_interface = wallet_interface.clone();
        let wallet_name = match params {
            Params::Array(arr) => arr
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("Missing wallet name"))?
                .to_string(),
            _ => return Err(RpcError::invalid_params("Invalid parameters")),
        };

        // Use spawn_blocking to run the async operation in a separate thread
        match task::block_in_place(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            let local = LocalSet::new();
            rt.block_on(async {
                local.run_until(async {
                    log::debug!("Inside async block in thread {:?}", std::thread::current().id());
                    wallet_interface.load_wallet(&wallet_name).await
                }).await
            })
        }) {
            Ok(_) => Ok(json!({
                "name": wallet_name,
                "warning": ""
            })),
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_getwalletinfo(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("getwalletinfo", move |_params| {
        log::info!("=========================");
        log::info!("Getting wallet info…");
        match wallet.get_balance() {
            Ok(balance) => {
                // Get transaction count
                let tx_count = match wallet.list_transactions() {
                    Ok(txs) => txs.len(),
                    Err(_) => 0,
                };
                
                let result = json!({
                    "walletname": "default",
                    "walletversion": 169900,
                    "format": "bdk",
                    "balance": balance.confirmed.to_btc(),
                    "unconfirmed_balance": balance.unconfirmed.to_btc(),
                    "immature_balance": balance.immature.to_btc(),
                    "txcount": tx_count,
                    "keypoololdest": 0, 
                    "keypoolsize": 1000,
                    "keypoolsize_hd_internal": 1000,
                    "paytxfee": 0,
                    "private_keys_enabled": true,
                    "avoid_reuse": false,
                    "scanning": false,
                    "descriptors": true,
                });
                Ok(result)
            }
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_getnewaddress(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("getnewaddress", move |params: Params| {
        log::info!("=========================");
        log::info!("Getting new address");
        let (label, address_type) = match params {
            Params::Array(arr) => {
                let label = arr.get(0).and_then(|v| v.as_str()).map(String::from);
                let address_type = arr.get(1).and_then(|v| v.as_str()).map(String::from);
                (label, address_type)
            }
            Params::Map(map) => {
                let label = map.get("label").and_then(|v| v.as_str()).map(String::from);
                let address_type = map.get("address_type").and_then(|v| v.as_str()).map(String::from);
                (label, address_type)
            }
            _ => (None, None),
        };

        if let Some(atype) = &address_type {
            if !["legacy", "p2sh-segwit", "bech32"].contains(&atype.as_str()) {
                return Err(RpcError::invalid_params("Invalid address type"));
            }

            if atype != "bech32" {
                log::warn!("Ignoring address_type={}, always returning bech32", atype);
            }
        }

        match wallet.get_new_address(label.as_deref()) {
            Ok(address) => Ok(Value::String(address.to_string())),
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_getbalance(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("getbalance", move |params: Params| {
        let wallet = wallet.clone();
        log::info!("=========================");
        log::info!("💰 Getting balance");
        match wallet.get_balance() {
            Ok(balance) => {
                let amt = balance.confirmed.to_btc();
                Ok(Value::Number(serde_json::Number::from_f64(amt).unwrap()))
            }
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_listunspent(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("listunspent", move |_params: Params| {
        log::info!("💰 Listing unspent");
        match wallet.list_unspent() {
            Ok(unspent) => {
                let result = json!({
                    "result": unspent
                });
                Ok(result)
            }
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_listtransactions(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("listtransactions", move |params: Params| {
        log::info!("=========================");
        log::info!("Listing transactions…");

        match wallet.list_transactions() {
            Ok(transactions) => {
                let result = json!({
                    "result": transactions
                });
                Ok(result)
            }
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_rescanblockchain(io: &mut IoHandler, wallet_interface: Arc<WalletInterface>) {
    io.add_sync_method("rescanblockchain", move |params: Params| {
        log::info!("=========================");
        log::info!("Rescanning blockchain...");
        
        // Parse optional start_height and stop_height parameters
        let (start_height, stop_height) = match params {
            Params::Array(arr) => {
                let start = arr.get(0).and_then(|v| v.as_i64()).unwrap_or(0);
                let stop = arr.get(1).and_then(|v| v.as_i64());
                (start, stop)
            }
            Params::Map(map) => {
                let start = map.get("start_height").and_then(|v| v.as_i64()).unwrap_or(0);
                let stop = map.get("stop_height").and_then(|v| v.as_i64());
                (start, stop)
            }
            _ => (0, None),
        };
        
        // Validate parameters
        if start_height < 0 {
            return Err(RpcError::invalid_params("Start height cannot be negative"));
        }
        
        if let Some(stop) = stop_height {
            if stop < start_height {
                return Err(RpcError::invalid_params("Stop height must be greater than or equal to start height"));
            }
        }
        
        // Use our dedicated rescan_blockchain method
        match task::block_in_place(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            let local = LocalSet::new();
            rt.block_on(async {
                local.run_until(async {
                    log::debug!("Starting blockchain rescan from height {}", start_height);
                    wallet_interface.rescan_blockchain(start_height as i32, stop_height.map(|h| h as i32)).await
                }).await
            })
        }) {
            Ok((actual_start, actual_stop)) => {
                Ok(json!({
                    "start_height": actual_start,
                    "stop_height": actual_stop
                }))
            },
            Err(e) => Err(rpc_error_from_wallet_error(e)),
        }
    });
}

fn register_gettransaction(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("gettransaction", move |params: Params| {
        log::info!("Getting transaction…");
        // Parse parameters
        // let (txid_str, include_watchonly) = match params {
        //     Params::Array(arr) => {
        //         let txid = arr.get(0).and_then(|v| v.as_str()).ok_or_else(|| {
        //             RpcError::invalid_params("Missing txid parameter")
        //         })?;
        //         let include_watchonly = arr.get(1).and_then(|v| v.as_bool()).unwrap_or(false);
        //         (txid, include_watchonly)
        //     }
        //     Params::Map(map) => {
        //         let txid = map.get("txid").and_then(|v| v.as_str()).ok_or_else(|| {
        //             RpcError::invalid_params("Missing txid parameter")
        //         })?;
        //         let include_watchonly = map.get("include_watchonly").and_then(|v| v.as_bool()).unwrap_or(false);
        //         (txid, include_watchonly)
        //     }
        //     _ => return Err(RpcError::invalid_params("Invalid parameters")),
        // };

        // Parse txid
        // let txid = match Txid::from_str(txid_str) {
        //     Ok(txid) => txid,
        //     Err(_) => return Err(RpcError::invalid_params("Invalid txid")),
        // };

        // Get transaction
        // Note: This would need to be handled by a background task in a real implementation
        // match tokio::runtime::Handle::current().block_on(wallet.get_transaction(&txid)) {
        //     Ok(tx) => {
        //         let result = json!({
        //             "amount": tx.amount.to_btc(),
        //             "confirmations": tx.confirmations,
        //             "blockhash": tx.blockhash.map(|h| h.to_string()).unwrap_or_default(),
        //             "blockindex": 0, // Would need actual block index
        //             "blocktime": tx.timestamp,
        //             "txid": tx.txid.to_string(),
        //             "time": tx.timestamp,
        //             "timereceived": tx.timestamp,
        //             "comment": tx.comment,
        //             "details": [
        //                 {
        //                     "address": "", // Would need tx details
        //                     "category": if tx.amount.is_negative() { "send" } else { "receive" },
        //                     "amount": tx.amount.to_btc(),
        //                     "label": tx.label,
        //                     "vout": 0, // Would need tx details
        //                 }
        //             ],
        //             "hex": "", // Would need serialized tx
        //         });
        //         Ok(result)
        //     }
        //     Err(e) => Err(rpc_error_from_wallet_error(e)),
        // }
        Ok(Value::String("txid".to_string()))
    });
}

fn register_sendtoaddress(io: &mut IoHandler, wallet: Arc<WalletInterface>) {
    io.add_sync_method("sendtoaddress", move |params: Params| {
        log::info!("Sending to address…");
        // Parse parameters
        // let (address_str, amount, comment, comment_to, subtract_fee, avoid_reuse, fee_rate) = match params {
        //     Params::Array(arr) => {
        //         let address = arr.get(0).and_then(|v| v.as_str()).ok_or_else(|| {
        //             RpcError::invalid_params("Missing address parameter")
        //         })?;
        //         let amount = arr.get(1).and_then(|v| v.as_f64()).ok_or_else(|| {
        //             RpcError::invalid_params("Missing amount parameter")
        //         })?;
        //         let comment = arr.get(2).and_then(|v| v.as_str()).unwrap_or("");
        //         let comment_to = arr.get(3).and_then(|v| v.as_str()).unwrap_or("");
        //         let subtract_fee = arr.get(4).and_then(|v| v.as_bool()).unwrap_or(false);
        //         let avoid_reuse = arr.get(5).and_then(|v| v.as_bool()).unwrap_or(false);
        //         let fee_rate = arr.get(6).and_then(|v| v.as_f64());
        //         (address, amount, comment, comment_to, subtract_fee, avoid_reuse, fee_rate)
        //     }
        //     Params::Map(map) => {
        //         let address = map.get("address").and_then(|v| v.as_str()).ok_or_else(|| {
        //             RpcError::invalid_params("Missing address parameter")
        //         })?;
        //         let amount = map.get("amount").and_then(|v| v.as_f64()).ok_or_else(|| {
        //             RpcError::invalid_params("Missing amount parameter")
        //         })?;
        //         let comment = map.get("comment").and_then(|v| v.as_str()).unwrap_or("");
        //         let comment_to = map.get("comment_to").and_then(|v| v.as_str()).unwrap_or("");
        //         let subtract_fee = map.get("subtract_fee_from_amount").and_then(|v| v.as_bool()).unwrap_or(false);
        //         let avoid_reuse = map.get("avoid_reuse").and_then(|v| v.as_bool()).unwrap_or(false);
        //         let fee_rate = map.get("fee_rate").and_then(|v| v.as_f64());
        //         (address, amount, comment, comment_to, subtract_fee, avoid_reuse, fee_rate)
        //     }
        //     _ => return Err(RpcError::invalid_params("Invalid parameters")),
        // };

        // Parse address
        // let address = match Address::from_str(address_str) {
        //     Ok(addr) => addr,
        //     Err(_) => return Err(RpcError::invalid_params("Invalid address")),
        // };

        // // Create amount in satoshis
        // let btc_amount = Amount::from_btc(amount).map_err(|_| {
        //     RpcError::invalid_params("Invalid amount")
        // })?;

        // // Create transaction
        // let recipient = TxRecipient {
        //     script: address.script_pubkey(),
        //     amount: btc_amount,
        // };

        // let subtract_indices = if subtract_fee { Some(vec![0]) } else { None };

        // match wallet.create_transaction(&[recipient], fee_rate, subtract_indices) {
        //     Ok(tx_details) => {
        //         // Sign transaction
        //         let mut tx = tx_details.transaction.clone();
        //         if let Err(e) = wallet.sign_transaction(&mut tx) {
        //             return Err(rpc_error_from_wallet_error(e));
        //         }
        //         // Send transaction
        //         // Note: This would need to be handled by a background task in a real implementation
        //         match tokio::runtime::Handle::current().block_on(wallet.send_transaction(&tx)) {
        //             Ok(txid) => Ok(Value::String(txid.to_string())),
        //             Err(e) => Err(rpc_error_from_wallet_error(e)),
        //         }
        //     }
        //     Err(e) => Err(rpc_error_from_wallet_error(e)),
        // }
        Ok(Value::String("txid".to_string()))
    });
}

fn parse_create_wallet_options(params: Params) -> Result<CreateWalletOptions, RpcError> {
    let mut options = CreateWalletOptions::default();

    match params {
        Params::Array(arr) => {
            options.wallet_name = arr
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("Missing wallet name parameter"))?
                .to_string();

            // Optional parameters
            if let Some(v) = arr.get(1).and_then(|v| v.as_bool()) {
                options.disable_private_keys = v;
            }
            if let Some(v) = arr.get(2).and_then(|v| v.as_bool()) {
                options.blank = v;
            }
            if let Some(v) = arr.get(3).and_then(|v| v.as_str()) {
                options.passphrase = Some(v.to_string());
            }
            if let Some(v) = arr.get(4).and_then(|v| v.as_bool()) {
                options.avoid_reuse = v;
            }
            if let Some(v) = arr.get(5).and_then(|v| v.as_bool()) {
                options.descriptors = v;
            }
            if let Some(v) = arr.get(6).and_then(|v| v.as_bool()) {
                options.load_on_startup = v;
            }
        }
        Params::Map(map) => {
            options.wallet_name = map
                .get("wallet_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("Missing wallet name parameter"))?
                .to_string();

            // Optional parameters
            if let Some(v) = map.get("disable_private_keys").and_then(|v| v.as_bool()) {
                options.disable_private_keys = v;
            }
            if let Some(v) = map.get("blank").and_then(|v| v.as_bool()) {
                options.blank = v;
            }
            if let Some(v) = map.get("passphrase").and_then(|v| v.as_str()) {
                options.passphrase = Some(v.to_string());
            }
            if let Some(v) = map.get("avoid_reuse").and_then(|v| v.as_bool()) {
                options.avoid_reuse = v;
            }
            if let Some(v) = map.get("descriptors").and_then(|v| v.as_bool()) {
                options.descriptors = v;
            }
            if let Some(v) = map.get("load_on_startup").and_then(|v| v.as_bool()) {
                options.load_on_startup = v;
            }
        }
        _ => return Err(RpcError::invalid_params("Invalid parameters")),
    };

    Ok(options)
}