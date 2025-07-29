mod request;
mod contracts;
mod configs;
mod utils;

use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    native_token::LAMPORTS_PER_SOL,
    message::v0::{Message, MessageAddressTableLookup},
    transaction::VersionedTransaction,
    signature::Signature
};

use solana_program::{
    instruction::{AccountMeta, Instruction},
    hash::Hash,
};

use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
};
use anyhow::{Error, Result, Context, bail};

use solana_sdk::message::VersionedMessage;
use thiserror::Error;

use std::time::Instant;
use std::str::FromStr;

use crate::request::ThirdPartySender;
use crate::request::SignedTransaction;
use crate::contracts::group_validate_compile::validate_compile;
use crate::contracts::pumpfun_proxy::PumpfunProxy;
use crate::contracts::jito_trick::jito_trick_trade;
use crate::contracts::remain_balance_check::remain_balance_check;
use crate::configs::global::*;
use crate::configs::bribe::*;
use base64::{Engine as _, engine::general_purpose};
use base64::{engine::general_purpose::STANDARD, Engine};
use anyhow::{anyhow};

use serde::Deserialize;
use crate::utils::{SwapParam, build_tip_transfer_instruction, SolAccountStruct};

use anchor_lang::Discriminator;
use anchor_spl::token::accessor::mint;
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use futures::join;


#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Signing error")]
    SigningError,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid private key")]
    InvalidPrivateKey,
}

pub struct TransactionBuilder {
    rpc_url: String,
    rpc_client: RpcClient,
    fee_payer: Keypair,
}

impl TransactionBuilder {
    pub fn new(rpc_url: String, fee_payer: Keypair) -> Self {
        TransactionBuilder {
            rpc_url: rpc_url.clone(),
            rpc_client: RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed()),
            fee_payer,
        }
    }

    pub fn check_balance(&self) -> Result<u64> {
        self.rpc_client
            .get_balance(&self.fee_payer.pubkey())
            .map_err(|e| TransactionError::RpcError(e.to_string()).into())
    }

    pub async fn build_custom_instructions(
        &self,
        param: &SwapParam,
        is_buy: bool,
        token_coin: &Pubkey,
        token_ata_account_addr: &Pubkey,
        buy_once: bool,
    ) -> Result<Vec<Instruction>> {

        let mut instructions = vec![];

        //pumpfun_proxy
        let client_test_only = RpcClient::new_with_commitment(
            self.rpc_url.clone(),
            CommitmentConfig::processed()
        );
        let pumpfun_builder = PumpfunProxy::new(
            client_test_only,
            Pubkey::from_str(PUMPFUN_PROGRAM_ID)?,
            Pubkey::from_str(PUMPFUN_PROXY_PROGRAM_ID)?,
        );
        // let creator_vault: Option<&Pubkey> = match param.creator_vault {
        //     Some(value) => Some(&Pubkey::from_str(&value)?),
        //     None => None
        // };
        let creator_vault_pubkey;
        let creator_vault: Option<&Pubkey> = match &param.creator_vault {
            Some(value) => {
                creator_vault_pubkey = Pubkey::from_str(&value)?;
                Some(&creator_vault_pubkey)
            },
            None => None
        };

        if is_buy {
            let calculate_amount_out = param.calculate_amount_out.unwrap_or_else(|| false);
            // let calculate_amount_out = match param.calculate_amount_out {
            //     Some(value) => true,
            //     None => false
            // };
            let build_buy_instruction = pumpfun_builder.get_buy_instruction(
                &param.amount_in,
                &param.amount_out,
                &calculate_amount_out,
                &token_coin,
                &token_ata_account_addr,
                &self.fee_payer.pubkey(),
                creator_vault
            ).await.unwrap();
            instructions.push(build_buy_instruction);
        }
        else{
            let build_sell_instruction = pumpfun_builder.get_sell_instruction(
                &param.amount_in,
                &param.amount_out,
                &token_coin,
                &token_ata_account_addr,
                &self.fee_payer.pubkey(),
                creator_vault
            ).await.unwrap();
            instructions.push(build_sell_instruction);
        }

        Ok(instructions)
    }

    pub async fn build_pub_instructions(
        &self,
        swap_param: &SwapParam,
        is_buy: bool,
        token_coin: &Pubkey,
        token_ata_account_addr: &Pubkey,
        buy_once: bool,
    ) -> Result<Vec<Instruction>> {

        let mut param = swap_param.clone();
        // let is_buy: bool = if param.token_in.to_string() == NATIVE_MINT { true } else { false };
        // let token_coin: &Pubkey = if is_buy { &Pubkey::from_str(&param.token_out)? } else { &Pubkey::from_str(&param.token_in)? };
        // let buy_once = param.buy_once.unwrap_or_else(|| false);
        let native_mint = Pubkey::from_str(NATIVE_MINT)?;
        let mut instructions = vec![];

        // priority fee limit
        let compute_unit = param.compute_unit.unwrap_or(0);
        let compute_price = param.compute_price.unwrap_or(0);

        // const MAX_FEE: u128 = 50_000_000; //0.05
        // const FEE_DECIMALS: u128 = 1_000_000_000_000_000; // 10^9 * 10^6
        const MAX_FEE_SCALED: u128 = 50_000_000 * 1_000_000_000_000_000;

        if (compute_unit as u128) * (compute_price as u128) > MAX_FEE_SCALED {
            param.compute_price = Some((MAX_FEE_SCALED / compute_unit as u128) as u64);
        }

        //bribe limit
        const MAX_TIP: u64 = 500_000_000; //0.5
        if let Some(bundle_bribe) = param.bundle_bribe {
            if bundle_bribe > MAX_TIP {
                param.bundle_bribe = Some(MAX_TIP);
            }
        }
        if let Some(land_bundle_bribe) = param.land_bundle_bribe {
            if land_bundle_bribe > MAX_TIP {
                param.land_bundle_bribe = Some(MAX_TIP);
            }
        }

        if let Some(compute_unit) = param.compute_unit {
            let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(compute_unit as u32);
            let jito_mev_pubkey = Pubkey::from_str("jitodontfront111111111111111111111111111111")?;
            let mut new_accounts = compute_ix.accounts.clone();
            new_accounts.push(AccountMeta {
                pubkey: jito_mev_pubkey,
                is_signer: false,
                is_writable: false,
            });

            instructions.push(
                Instruction {
                    program_id: compute_ix.program_id,
                    accounts: new_accounts,
                    data: compute_ix.data.clone(),
                }
            );
        }

        if let Some(compute_price) = param.compute_price {
            instructions.push(
                ComputeBudgetInstruction::set_compute_unit_price(compute_price),
            );
        }


        //coin_ata
        // let token_ata_account_addr = get_associated_token_address(
        //     &self.fee_payer.pubkey(),
        //     &token_coin,
        // );

        if is_buy {
            let token_ata_instruction = create_associated_token_account(
                &self.fee_payer.pubkey(),
                &self.fee_payer.pubkey(),
                &token_coin,
                &Pubkey::from_str(TOKEN_PROGRAM_ID)?
            );

            if buy_once {
                instructions.push(token_ata_instruction);
            }
            else{
                match self.rpc_client.get_account(
                    &token_ata_account_addr
                ) {
                    Ok(account_info) => {
                        Some(1)
                    },
                    Err(e) => {
                        instructions.push(token_ata_instruction);
                        None
                    }
                };
            }
        }

        //validate_compile
        let token_account_out = if is_buy { token_ata_account_addr } else { &native_mint };
        if (param.group.is_some() && param.group_id.is_some()) ||
            param.max_block_number.is_some() ||
            param.max_amount_out.is_some() {

            let group = param.group.unwrap_or_else(|| "trade_group".to_string());
            let group_id = param.group_id.unwrap_or_else(|| 0);
            let max_block_number = param.max_block_number.unwrap_or_else(|| 0);
            let max_amount_out = param.max_amount_out.unwrap_or_else(|| 0);

            let validate_compile_instruction = validate_compile(
                &group,
                group_id,
                max_block_number,
                max_amount_out,
                2,
                &token_account_out
            );

            instructions.push(validate_compile_instruction);
        }

        //jito trick
        if param.block_number.is_some() && param.simulate_bundle_bribe.is_some() &&
            param.second_block_bundle_bribe.is_some() && param.land_bundle_bribe.is_some() {
            let first_block_tip_amount = param.simulate_bundle_bribe.unwrap_or_else(|| 0);
            let second_block_tip_amount = param.second_block_bundle_bribe.unwrap_or_else(|| 0);
            let other_block_tip_amount = param.land_bundle_bribe.unwrap_or_else(|| 0);
            let block_number = param.block_number.unwrap_or_else(|| 0);
            let jito_trick_instruction = jito_trick_trade(
                self.fee_payer.pubkey(),
                &first_block_tip_amount,
                &second_block_tip_amount,
                &other_block_tip_amount,
                block_number
            );
            instructions.push(jito_trick_instruction);
        }

        //classic transfer
        if param.token_balance.is_some() && param.token_mint.is_some() {
            let token_balance = param.token_balance.unwrap();
            let token_mint = Pubkey::from_str(&param.token_mint.unwrap())?;
            let token_account_addr = get_associated_token_address(
                &self.fee_payer.pubkey(),
                &token_mint,
            );
            let transfer_instruction = system_instruction::transfer(
                &token_account_addr,
                &token_account_addr,
                token_balance,
            );

            instructions.push(transfer_instruction);
        }

        //remain_balance_check
        if param.remain_token_balance.is_some() {
            let remain_token_balance = param.remain_token_balance.unwrap();
            let remain_check_instruction = remain_balance_check(
                *token_ata_account_addr,
                remain_token_balance
            );
            instructions.push(remain_check_instruction);
        }

        Ok(instructions)

    }


    pub async fn send_tx(
        &self,
        swap_param: &SwapParam,
        mut instructions: Vec<Instruction>,
        bundle_type: Option<u8>,
        recent_blockhash: &Hash,
        sender: &ThirdPartySender,
    ) -> Result<String> {

        if let Some(sender_type) = bundle_type {
            let tip_transfer_instruction = build_tip_transfer_instruction(
                swap_param,
                &self.fee_payer.pubkey(),
                &sender_type,
            );
            // println!("tip_transfer_instruction {:?}", tip_transfer_instruction);
            instructions.extend(vec![tip_transfer_instruction]);
            // instructions.push(tip_transfer_instruction);
        }

        let start = Instant::now();

        //versioned tx
        let message = Message::try_compile(
            &self.fee_payer.pubkey(),
            &instructions,
            &[],
            *recent_blockhash,
        )?;
        let transaction = VersionedTransaction::try_new(
            VersionedMessage::V0(message),
            &[&self.fee_payer],
        )?;


        // println!("Instructions: {:?} {:?}", instructions, recent_blockhash);


        let duration = start.elapsed();
        println!("Message::new 执行时间: {:?}", duration);

        //start sending
        let response = if Some(0) == bundle_type {
            sender.send_jito(
                &transaction,
                "https://amsterdam.mainnet.block-engine.jito.wtf",
                JITO_WALLET_KEY,
            ).await
        }
        else if Some(1) == bundle_type {
            sender.send_bloxroute(
                &transaction,
                "http://ams1.nozomi.temporal.xyz",
                TEMPORAL_KEY,
            ).await
        }
        else if Some(2) == bundle_type {
            sender.send_temporal(
                &transaction,
                "http://ams1.nozomi.temporal.xyz",
                TEMPORAL_KEY,
            ).await
        }
        else if Some(3) == bundle_type {
            sender.send_nextblock(
                &transaction,
                "http://de1.0slot.trade",
                SLOTE0_TRADE_KEY,
            ).await
        }
        else if Some(4) == bundle_type {
            sender.send_0slot(
                &transaction,
                "http://de1.0slot.trade",
                SLOTE0_TRADE_KEY,
            ).await
        }
        else if Some(5) == bundle_type {
            sender.send_node1(
                &transaction,
                "http://fra.node1.me",
                NODE1_ME_KEY,
            ).await
        }
        else {
            bail!("Invalid bundle type");
        };

        response

    }


    pub async fn trade(
        &self,
        swap_param: &SwapParam,
    ) -> Result<String> {

        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .map_err(|e| TransactionError::RpcError(e.to_string()))?;

        let start = Instant::now();

        let mut param = swap_param.clone();
        let is_buy: bool = if param.token_in.to_string() == NATIVE_MINT { true } else { false };
        let token_coin: Pubkey = if is_buy { Pubkey::from_str(&param.token_out)? } else { Pubkey::from_str(&param.token_in)? };
        let buy_once = param.buy_once.unwrap_or_else(|| false);

        let token_ata_account_addr = get_associated_token_address(
            &self.fee_payer.pubkey(),
            &token_coin,
        );

        let pub_instructions = self.build_pub_instructions(
            &param,
            is_buy,
            &token_coin,
            &token_ata_account_addr,
            buy_once,
        ).await.unwrap();

        let custom_instructions = self.build_custom_instructions(
            &param,
            is_buy,
            &token_coin,
            &token_ata_account_addr,
            buy_once,
        ).await.unwrap();

        // println!("pub_instructions {:?}", pub_instructions);
        // println!("custom_instructions {:?}", custom_instructions);

        let mut instructions = vec![];
        instructions.extend(pub_instructions);
        instructions.extend(custom_instructions);

        // let recent_blockhash = self.rpc_client
        //     .get_latest_blockhash()
        //     .map_err(|e| TransactionError::RpcError(e.to_string()))?;

        let sender = match ThirdPartySender::new(){
            Ok(sender) => sender,
            Err(e) => {
                eprintln!("Failed to create sender: {}", e);
                return Err(e);
            }
        };

        let (
        signature_jito,
            signature_bloxroute,
            signature_temporal,
            signature_nextblock,
            signature_0slot,
            signature_node1
        ) = join!(
            self.send_tx(
                &param,
                instructions.clone(),
                Some(0),
                &recent_blockhash,
                &sender
            ),
            self.send_tx(
                &param,
                instructions.clone(),
                Some(1),
                &recent_blockhash,
                &sender
            ),
            self.send_tx(
                &param,
                instructions.clone(),
                Some(2),
                &recent_blockhash,
                &sender
            ),
            self.send_tx(
                &param,
                instructions.clone(),
                Some(3),
                &recent_blockhash,
                &sender
            ),
            self.send_tx(
                &param,
                instructions.clone(),
                Some(4),
                &recent_blockhash,
                &sender
            ),
            self.send_tx(
                &param,
                instructions.clone(),
                Some(5),
                &recent_blockhash,
                &sender
            )
        );

        let duration = start.elapsed();
        println!("trade 执行时间: {:?}", duration);

        println!("signature {:?} {:?}", signature_0slot, signature_node1);

        Ok(signature_node1?)

    }
}


#[tokio::main]
async fn main() -> Result<()> {


    let rpc_url = "https://api.mainnet-beta.solana.com".to_string();

    let private_key = "";

    let fee_payer = Keypair::from_base58_string(private_key);

    let amount = 100 * LAMPORTS_PER_SOL; // 1 SOL = 1,000,000,000 lamports
    // let amount = (0.002 * LAMPORTS_PER_SOL as f64) as u64; // 1 SOL = 1,000,000,000 lamports

    println!("初始化交易构建器...");
    let mut builder = TransactionBuilder::new(rpc_url, fee_payer);

    let param = SwapParam {
        connection: "".to_string(),
        connection_brand : "".to_string(),
        secret_key : private_key.to_string(),
        amount_in : amount,
        amount_out : 1000,
        fixed_side : "in".to_string(),
        target_pool : "".to_string(),
        token_in : NATIVE_MINT.to_string(),
        token_out : "FPh9d5pzRo3AK1Fonxnn1z3WWXcwEnmMEQXdbemby3DS".to_string(),
        decimals_in : 9,
        decimals_out : 6,
        slippage_amount : 100,
        compute_unit : Some(150000),
        compute_price : Some(5000),
        jito_compute_price: None,
        bloxroute_compute_price : None,
        temporal_compute_price : None,
        nextblock_compute_price : None,
        slot0_trade_compute_price : None,
        nodeme_compute_price : None,

        block_engine_locate: None,
        block_engine_url: None,
        bundle_bribe : None,
        jito_bribe: Some(1),
        bloxroute_bundle_bribe: Some(1),
        temporal_bundle_bribe: Some(1),
        nextblock_bundle_bribe: Some(1),
        slot0_trade_bundle_bribe: Some(1),
        nodeme_bundle_bribe: Some(2),
        bundle_amount_out: None,

        buy_once: None,
        // max_block_number: Some(9999999999),
        max_block_number: None,
        token_balance: None,
        token_mint: None,
        market_id: None,

        skip_retry: None,
        group_id: None,
        // group: Some("7uQxuvVFzY3uApna9PFQEMQEUKrrwFYG4D4272aPHT1d".to_string()),
        group: None,

        // tradeMode?: number,
        trade_times: None,

        recent_block_hash: None,
        jito_recent_block_hash: None,
        bloxroute_recent_block_hash: None,
        temporal_recent_block_hash: None,
        nextblock_recent_block_hash: None,
        slot0_trade_recent_block_hash: None,
        nodeme_recent_block_hash: None,

        sol_account: None,
        coin_account: None,

        max_amount_out: None,
        bundle_type: None,

        // remain_token_balance: Some(5555),
        remain_token_balance: None,

        // block_number: Some(499),
        // simulate_bundle_bribe: Some(1_000_000),
        // second_block_bundle_bribe: Some(2_000_000),
        // land_bundle_bribe: Some(3_000_000),
        block_number: None,
        simulate_bundle_bribe: None,
        second_block_bundle_bribe: None,
        land_bundle_bribe: None,

        calculate_amount_out: None,

        track_token_account : None,
        track_amount_in: None,
        track_slippage: None,
        track_max_allow_buy: None,
        track_token_balance: None,

        snipe_raydium_sol_reserve: None,

        send_normal_trade: None, //0 no, 1 yes
        trade_manual_local_rpc: None, //0 no, 1 yes, only for trade_manual

        anti_mev: None, // third-party api prevents MEV, 0 no, 1 yes

        creator_vault: None,
    };

    let response = builder.trade(&param).await;

    match response {
        Ok(signature) => {
            println!("交易哈希: {}", signature);
            println!("查看交易: https://explorer.solana.com/tx/{}?cluster=devnet", signature);

        },
        Err(e) => {
            eprintln!("❌ 自转账失败: {}", e);
        }
    }

    Ok(())
}