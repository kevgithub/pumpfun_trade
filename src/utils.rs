use std::sync::OnceLock;
use std::time::Duration;
use serde::Deserialize;
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use crate::configs::bribe::{
    get_node1me_random_tip_account,
    get_random_bloxroute_tip_account,
    get_random_jito_tip_account,
    get_random_nextblock_tip_account,
    get_slot0_trade_random_tip_account,
    get_temporal_random_tip_account
};
use napi_derive::napi;
use reqwest::Client;

pub const TYPE_JITO: u8 = 0;
pub const TYPE_BLOXROUTE: u8 = 1;
pub const TYPE_TEMPORAL: u8 = 2;
pub const TYPE_NEXTBLOCK: u8 = 3;
pub const TYPE_0SLOT_TRADE: u8 = 4;
pub const TYPE_NODE1_ME: u8 = 5;


#[derive(Debug, Deserialize, Clone)]
#[napi(object)]
pub struct SolAccountStruct {
pub public_key: String,
pub seed: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct SwapParam {
    pub connection: String,
    pub connection_brand : String,
    // pub connection_send: String,
    // pub connection_send_brand: String,
    // pub trade_type : String,
    pub secret_key : String,
    pub amount_in : u64,
    pub amount_out : u64,
    pub fixed_side : String,
    pub target_pool : String,
    pub token_in : String,
    pub token_out : String,
    pub decimals_in : u32,
    pub decimals_out : u32,
    pub slippage_amount : u32,
    pub compute_unit : Option<u64>,
    pub compute_price : Option<u64>,
    pub jito_compute_price: Option<u64>,
    pub bloxroute_compute_price : Option<u64>,
    pub temporal_compute_price : Option<u64>,
    pub nextblock_compute_price : Option<u64>,
    pub slot0_trade_compute_price : Option<u64>,
    pub nodeme_compute_price : Option<u64>,

    pub block_engine_locate: Option<String>,
    pub block_engine_url: Option<String>,
    pub bundle_bribe : Option<u64>,
    pub jito_bribe: Option<u64>,
    pub bloxroute_bundle_bribe: Option<u64>,
    pub temporal_bundle_bribe: Option<u64>,
    pub nextblock_bundle_bribe: Option<u64>,
    pub slot0_trade_bundle_bribe: Option<u64>,
    pub nodeme_bundle_bribe: Option<u64>,
    pub bundle_amount_out: Option<u64>,

    pub buy_once: Option<bool>,
    pub max_block_number: Option<u64>,
    pub token_balance: Option<u64>,
    pub token_mint: Option<String>,
    pub market_id: Option<String>,

    pub skip_retry: Option<bool>,
    pub group_id: Option<u64>,
    pub group: Option<String>,

    // tradeMode?: number,
    pub trade_times: Option<u8>,

    pub recent_block_hash: Option<String>,
    pub jito_recent_block_hash: Option<String>,
    pub bloxroute_recent_block_hash: Option<String>,
    pub temporal_recent_block_hash: Option<String>,
    pub nextblock_recent_block_hash: Option<String>,
    pub slot0_trade_recent_block_hash: Option<String>,
    pub nodeme_recent_block_hash: Option<String>,

    pub sol_account: Option<SolAccountStruct>,
    pub coin_account: Option<String>,

    pub max_amount_out: Option<u64>,
    pub bundle_type: Option<u8>,

    pub remain_token_balance: Option<u64>,

    pub block_number: Option<u64>,
    pub simulate_bundle_bribe: Option<u64>,
    pub second_block_bundle_bribe: Option<u64>,
    pub land_bundle_bribe: Option<u64>,

    pub calculate_amount_out: Option<bool>,

    pub track_token_account : Option<String>,
    pub track_amount_in: Option<u64>,
    pub track_slippage: Option<u64>,
    pub track_max_allow_buy: Option<u64>,
    pub track_token_balance: Option<u64>,

    pub snipe_raydium_sol_reserve: Option<u64>,

    pub send_normal_trade: Option<u8>, //0 no, 1 yes
    pub trade_manual_local_rpc: Option<u8>, //0 no, 1 yes, only for trade_manual

    pub anti_mev: Option<u8>, // third-party api prevents MEV, 0 no, 1 yes

    pub creator_vault: Option<String>,
}



#[derive(Debug, Deserialize, Clone)]
#[napi(object)]
pub struct SwapParam4Node {
    pub connection: String,
    pub connection_brand : String,
    pub secret_key : String,
    pub amount_in : String,
    pub amount_out : String,
    pub fixed_side : String,
    pub target_pool : String,
    pub token_in : String,
    pub token_out : String,
    pub decimals_in : String,
    pub decimals_out : String,
    pub slippage_amount : String,
    pub compute_unit : Option<String>,
    pub compute_price : Option<String>,
    pub jito_compute_price: Option<String>,
    pub bloxroute_compute_price : Option<String>,
    pub temporal_compute_price : Option<String>,
    pub nextblock_compute_price : Option<String>,
    pub slot0_trade_compute_price : Option<String>,
    pub nodeme_compute_price : Option<String>,

    pub block_engine_locate: Option<String>,
    pub block_engine_url: Option<String>,
    pub bundle_bribe : Option<String>,
    pub jito_bribe: Option<String>,
    pub bloxroute_bundle_bribe: Option<String>,
    pub temporal_bundle_bribe: Option<String>,
    pub nextblock_bundle_bribe: Option<String>,
    pub slot0_trade_bundle_bribe: Option<String>,
    pub nodeme_bundle_bribe: Option<String>,
    pub bundle_amount_out: Option<String>,

    pub buy_once: Option<bool>,
    pub max_block_number: Option<String>,
    pub token_balance: Option<String>,
    pub token_mint: Option<String>,
    pub market_id: Option<String>,

    pub skip_retry: Option<bool>,
    pub group_id: Option<String>,
    pub group: Option<String>,

    // tradeMode?: number,
    pub trade_times: Option<String>,

    pub recent_block_hash: Option<String>,
    pub jito_recent_block_hash: Option<String>,
    pub bloxroute_recent_block_hash: Option<String>,
    pub temporal_recent_block_hash: Option<String>,
    pub nextblock_recent_block_hash: Option<String>,
    pub slot0_trade_recent_block_hash: Option<String>,
    pub nodeme_recent_block_hash: Option<String>,

    pub sol_account: Option<SolAccountStruct>,
    pub coin_account: Option<String>,

    pub max_amount_out: Option<String>,
    pub bundle_type: Option<String>,

    pub remain_token_balance: Option<String>,

    pub block_number: Option<String>,
    pub simulate_bundle_bribe: Option<String>,
    pub second_block_bundle_bribe: Option<String>,
    pub land_bundle_bribe: Option<String>,

    pub calculate_amount_out: Option<bool>,

    pub track_token_account : Option<String>,
    pub track_amount_in: Option<String>,
    pub track_slippage: Option<String>,
    pub track_max_allow_buy: Option<String>,
    pub track_token_balance: Option<String>,

    pub snipe_raydium_sol_reserve: Option<String>,

    pub send_normal_trade: Option<String>, //0 no, 1 yes
    pub trade_manual_local_rpc: Option<String>, //0 no, 1 yes, only for trade_manual

    pub anti_mev: Option<String>, // third-party api prevents MEV, 0 no, 1 yes

    pub creator_vault: Option<String>,
}


pub fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_client::anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()
            [..8],
    );
    sighash
}


pub fn build_tip_transfer_instruction(
    swap_param: &SwapParam,
    payer: &Pubkey,
    sender_type: &u8,
) -> Instruction {
    let mut bribe: u64 = 0;
    let tip_addr: Option<Pubkey> = match sender_type {
        &TYPE_JITO => {
            bribe = swap_param.jito_bribe.unwrap_or_else(|| 0) * LAMPORTS_PER_SOL;
            Some(get_random_jito_tip_account(&bribe))
        }
        &TYPE_BLOXROUTE => {
            bribe = swap_param.bloxroute_bundle_bribe.unwrap_or_else(|| 0) * LAMPORTS_PER_SOL;
            Some(get_random_bloxroute_tip_account(&bribe))
        }
        &TYPE_TEMPORAL => {
            bribe = swap_param.temporal_bundle_bribe.unwrap_or_else(|| 0) * LAMPORTS_PER_SOL;
            Some(get_temporal_random_tip_account(&bribe))
        }
        &TYPE_NEXTBLOCK => {
            bribe = swap_param.nextblock_bundle_bribe.unwrap_or_else(|| 0) * LAMPORTS_PER_SOL;
            Some(get_random_nextblock_tip_account(&bribe))
        }
        &TYPE_0SLOT_TRADE => {
            bribe = swap_param.slot0_trade_bundle_bribe.unwrap_or_else(|| 0) * LAMPORTS_PER_SOL;
            Some(get_slot0_trade_random_tip_account(&bribe))
        }
        &TYPE_NODE1_ME => {
            bribe = swap_param.nodeme_bundle_bribe.unwrap_or_else(|| 0) * LAMPORTS_PER_SOL;
            Some(get_node1me_random_tip_account(&bribe))
        }
        _ => {
            None
        }
    };

    system_instruction::transfer(
        payer,
        &tip_addr.expect("tip_addr is none"),
        bribe,
    )
}


static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| Client::builder()
        // .default_headers(headers)
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(200)
        .tcp_keepalive(Duration::from_secs(1))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
    )
}
