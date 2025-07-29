#![deny(clippy::all)]
pub mod request;
pub mod keep_alive;
pub mod contracts;
pub mod configs;
pub mod utils;
pub mod transaction_builder;
pub mod node_to_rust;

use napi_derive::napi;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use crate::transaction_builder::TransactionBuilder;
use crate::utils::{get_client, SwapParam, SwapParam4Node};
use crate::configs::global::{NATIVE_MINT};
use crate::node_to_rust::*;
use crate::keep_alive::*;
use napi::Error as NapiError;
use std::time::Instant;
use anchor_spl::token::accessor::authority;

#[napi]
pub async fn swap(
    param: SwapParam4Node
) -> Result<Vec<String>, NapiError>  {
    
    let start = Instant::now();

    let rust_param: SwapParam = param.into();
    let rpc_url = rust_param.connection.to_string();

    let fee_payer = Keypair::from_base58_string(&rust_param.secret_key);

    // let amount = 100 * LAMPORTS_PER_SOL; // 1 SOL = 1,000,000,000 lamports
    // let amount = (0.002 * LAMPORTS_PER_SOL as f64) as u64; // 1 SOL = 1,000,000,000 lamports

    println!("start...");
    let mut builder = TransactionBuilder::new(rpc_url, fee_payer);

    let response = builder.trade(&rust_param).await;

    println!("rust cost {:?}", start.elapsed());

    response.map_err(|e| NapiError::from_reason(e.to_string()))

}

#[napi]
pub async fn keepalive_bloxroute(
    api_key: String,
    api_url: String,
){
    let _ = send_bloxroute(&api_key, &api_url).await;
}

#[napi]
pub async fn keepalive_nodeme(
    api_url: String,
){
    let _ = send_nodeme(&api_url).await;
}

#[napi]
pub async fn keepalive_temporal(
    api_url: String,
){
    let _ = send_temporal(&api_url).await;
}

#[napi]
pub async fn keepalive_slot(
    api_url: String,
    auth_key: String,
){
    let _ = send_0slot(&api_url, &auth_key).await;
}