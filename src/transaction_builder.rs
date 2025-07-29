
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
use crate::utils::{SwapParam, build_tip_transfer_instruction, SolAccountStruct, TYPE_JITO, TYPE_NEXTBLOCK, TYPE_TEMPORAL, TYPE_BLOXROUTE, TYPE_0SLOT_TRADE, TYPE_NODE1_ME};

use anchor_lang::Discriminator;
use anchor_spl::token::accessor::mint;
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use futures::future::join_all;
use std::env;
use solana_program::hash::hash;
use tokio::runtime::Handle;
use dotenv::dotenv;

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

    pub fn check_balance(&self) -> anyhow::Result<u64> {
        self.rpc_client
            .get_balance(&self.fee_payer.pubkey())
            .map_err(|e| TransactionError::RpcError(e.to_string()).into())
    }

    pub async fn build_proxy_contract_instructions(
        &self,
        param: &SwapParam,
        is_buy: bool,
        token_coin: &Pubkey,
        token_ata_account_addr: &Pubkey,
        buy_once: bool,
    ) -> anyhow::Result<Vec<Instruction>> {

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
        // let compute_unit = param.compute_unit.unwrap_or(0);
        // let compute_price = param.compute_price.unwrap_or(0);

        // const MAX_FEE: u128 = 50_000_000; //0.05
        // const FEE_DECIMALS: u128 = 1_000_000_000_000_000; // 10^9 * 10^6
        // const MAX_FEE_SCALED: u128 = 50_000_000 * 1_000_000_000_000_000;
        //
        // if (compute_unit as u128) * (compute_price as u128) > MAX_FEE_SCALED {
        //     param.compute_price = Some((MAX_FEE_SCALED / compute_unit as u128) as u64);
        // }

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
            let jito_mev_pubkey = Pubkey::from_str(JITO_MEV_PREVENT_ADDR)?;
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

        // if let Some(compute_price) = param.compute_price {
        //     instructions.push(
        //         ComputeBudgetInstruction::set_compute_unit_price(compute_price),
        //     );
        // }


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

    pub fn build_custom_instruction(
        &self,
        bundle_type: Option<u8>,
        swap_param: &SwapParam,
    ) -> Vec<Instruction> {
        let mut compute_price: u64 = 0;
        if let Some(bundle_id) = bundle_type {
            if TYPE_JITO == bundle_id {
                compute_price = swap_param.jito_compute_price.unwrap_or(0);
            }
            else if TYPE_BLOXROUTE == bundle_id {
                compute_price = swap_param.bloxroute_compute_price.unwrap_or(0);
            }
            else if TYPE_TEMPORAL == bundle_id {
                compute_price = swap_param.temporal_compute_price.unwrap_or(0);
            }
            else if TYPE_NEXTBLOCK == bundle_id {
                compute_price = swap_param.nextblock_compute_price.unwrap_or(0);
            }
            else if TYPE_0SLOT_TRADE == bundle_id {
                compute_price = swap_param.slot0_trade_compute_price.unwrap_or(0);
            }
            else if TYPE_NODE1_ME == bundle_id {
                compute_price = swap_param.nodeme_compute_price.unwrap_or(0);
            }
            else {
                compute_price = swap_param.compute_price.unwrap_or(0);
            }
        }
        else {
            //normal trade
            compute_price = swap_param.compute_price.unwrap_or(0);
        }

        // priority fee limit
        let compute_unit = swap_param.compute_unit.unwrap_or(0);

        // const MAX_FEE: u128 = 50_000_000; //0.05
        // const FEE_DECIMALS: u128 = 1_000_000_000_000_000; // 10^9 * 10^6
        const MAX_FEE_SCALED: u128 = 50_000_000 * 1_000_000_000_000_000;

        if (compute_unit as u128) * (compute_price as u128) > MAX_FEE_SCALED {
            compute_price = (MAX_FEE_SCALED / compute_unit as u128) as u64;
        }

        vec![
            ComputeBudgetInstruction::set_compute_unit_price(compute_price)
        ]
    }

    pub fn rpc_recent_block_hash(
        &self,
    ) -> Hash {
        self.rpc_client
            .get_latest_blockhash()
            .expect("rpc_recent_block_hash failed")
    }

    pub fn get_recent_block_hash(
        &self,
        bundle_type: Option<u8>,
        swap_param: &SwapParam,
    ) -> Hash {
        if let Some(bundle_id) = bundle_type {
            if TYPE_JITO == bundle_id {
                if let Some(block_hash) = &swap_param.jito_recent_block_hash {
                    Hash::from_str(block_hash).unwrap()
                }
                else { self.rpc_recent_block_hash() }
            }
            else if TYPE_BLOXROUTE == bundle_id {
                if let Some(block_hash) = &swap_param.bloxroute_recent_block_hash {
                    Hash::from_str(block_hash).unwrap()
                }
                else { self.rpc_recent_block_hash() }
            }
            else if TYPE_TEMPORAL == bundle_id {
                if let Some(block_hash) = &swap_param.temporal_recent_block_hash {
                    Hash::from_str(block_hash).unwrap()
                }
                else { self.rpc_recent_block_hash() }
            }
            else if TYPE_NEXTBLOCK == bundle_id {
                if let Some(block_hash) = &swap_param.nextblock_recent_block_hash {
                    Hash::from_str(block_hash).unwrap()
                }
                else { self.rpc_recent_block_hash() }
            }
            else if TYPE_0SLOT_TRADE == bundle_id {
                if let Some(block_hash) = &swap_param.slot0_trade_recent_block_hash {
                    Hash::from_str(block_hash).unwrap()
                }
                else { self.rpc_recent_block_hash() }
            }
            else if TYPE_NODE1_ME == bundle_id {
                if let Some(block_hash) = &swap_param.nodeme_recent_block_hash {
                    Hash::from_str(block_hash).unwrap()
                }
                else { self.rpc_recent_block_hash() }
            }
            else {
                println!("11111111");
                self.rpc_recent_block_hash()
            }
        }
        else {
            //normal trade
            println!("else 88888");
            self.rpc_recent_block_hash()
        }
    }


    pub async fn send_tx(
        &self,
        swap_param: &SwapParam,
        mut instructions: Vec<Instruction>,
        bundle_type: Option<u8>,
        block_engine_url: String,
        sender: &ThirdPartySender,
    ) -> Result<Vec<String>> {

        let recent_blockhash = self.get_recent_block_hash(bundle_type, swap_param);

        println!("recent_blockhash {:?}", recent_blockhash);

        if let Some(sender_type) = bundle_type {
            let tip_transfer_instruction = build_tip_transfer_instruction(
                swap_param,
                &self.fee_payer.pubkey(),
                &sender_type,
            );
            instructions.extend(vec![tip_transfer_instruction]);
        }
        let custom_instructions = self.build_custom_instruction(bundle_type, &swap_param);
        instructions.extend(custom_instructions);

        //versioned tx
        let message = Message::try_compile(
            &self.fee_payer.pubkey(),
            &instructions,
            &[],
            recent_blockhash,
        )?;
        let transaction = VersionedTransaction::try_new(
            VersionedMessage::V0(message),
            &[&self.fee_payer],
        )?;


        // println!("Instructions: {:?} {:?}", instructions, recent_blockhash);

        let http_time = Instant::now();

        //start sending
        let response = if Some(TYPE_JITO) == bundle_type {
            sender.send_jito(
                &transaction,
                &block_engine_url,
                JITO_WALLET_KEY,
            ).await
        }
        else if Some(TYPE_BLOXROUTE) == bundle_type {
            sender.send_bloxroute(
                &transaction,
                &block_engine_url,
                BLOXROUTE_KEY,
            ).await
        }
        else if Some(TYPE_TEMPORAL) == bundle_type {
            sender.send_temporal(
                &transaction,
                &block_engine_url,
                TEMPORAL_KEY,
            ).await
        }
        else if Some(TYPE_NEXTBLOCK) == bundle_type {
            sender.send_nextblock(
                &transaction,
                &block_engine_url,
                SLOTE0_TRADE_KEY,
            ).await
        }
        else if Some(TYPE_0SLOT_TRADE) == bundle_type {
            sender.send_0slot(
                &transaction,
                &block_engine_url,
                SLOTE0_TRADE_KEY,
            ).await
        }
        else if Some(TYPE_NODE1_ME) == bundle_type {
            sender.send_node1(
                &transaction,
                &block_engine_url,
                NODE1_ME_KEY,
            ).await
        }
        else {
            bail!("Invalid bundle type");
        };

        println!("http_time {:?}", http_time.elapsed());

        let result = vec![block_engine_url, response?];

        Ok(result)

        // response

    }


    pub async fn trade(
        &self,
        swap_param: &SwapParam,
    ) -> Result<Vec<String>> {

        let start = Instant::now();

        dotenv().ok();
        let mut param = swap_param.clone();

        // let recent_blockhash = self.rpc_client
        //     .get_latest_blockhash()
        //     .map_err(|e| TransactionError::RpcError(e.to_string()))?;


        let is_buy: bool = if param.token_in.to_string() == NATIVE_MINT { true } else { false };
        let token_coin: Pubkey = if is_buy { Pubkey::from_str(&param.token_out)? } else { Pubkey::from_str(&param.token_in)? };
        let buy_once = param.buy_once.unwrap_or_else(|| false);

        let token_ata_account_addr = get_associated_token_address(
            &self.fee_payer.pubkey(),
            &token_coin,
        );

        println!("trade before {:?}", start.elapsed());

        let pub_instructions = self.build_pub_instructions(
            &param,
            is_buy,
            &token_coin,
            &token_ata_account_addr,
            buy_once,
        ).await.unwrap();

        println!("trade pub instruction {:?}", start.elapsed());

        let custom_instructions = self.build_proxy_contract_instructions(
            &param,
            is_buy,
            &token_coin,
            &token_ata_account_addr,
            buy_once,
        ).await.unwrap();

        println!("trade custom instruction {:?}", start.elapsed());

        // println!("pub_instructions {:?}", pub_instructions);
        // println!("custom_instructions {:?}", custom_instructions);

        let mut instructions = vec![];
        instructions.extend(pub_instructions);
        instructions.extend(custom_instructions);

        println!("trade fuck 1 {:?}", start.elapsed());

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

        println!("trade fuck 2 {:?}", start.elapsed());
        // let duration = start.elapsed();
        // println!("trade cost: {:?}", duration);

        let mut execute_list = vec![];

        let bl_jito: Option<String> = env::var("STRATEGY_BIG_BLOCKENGINE_RUST").ok();
        let bl_nextblock: Option<String> = env::var("NEXTBLOCK_BLOCKENGINE_RUST").ok();
        let bl_temporal: Option<String> = env::var("TEMPORAL_BLOCKENGINE_RUST").ok();
        let bl_bloxroute: Option<String> = env::var("BLOXROUTE_BLOCKENGINE_RUST").ok();
        let bl_0slot: Option<String> = env::var("SLOT0_TRADE_BLOCKENGINE_RUST").ok();
        let bl_node1me: Option<String> = env::var("NODE1_ME_BLOCKENGINE_RUST").ok();

        println!("trade send before {:?}", start.elapsed());

        //node1me
        if send_node1me_or_not(
            param.nodeme_bundle_bribe,
            &bl_node1me,
        ) {
            let bl = bl_node1me.as_ref().unwrap();
            execute_list.push(
                self.send_tx(
                    &param,
                    instructions.clone(),
                    Some(TYPE_NODE1_ME),
                    bl.clone(),
                    &sender
                )
            );
        }

        //0slot.trade
        if send_slot0_trade_or_not(
            param.slot0_trade_bundle_bribe,
            &bl_0slot,
        ) {
            let bl = bl_0slot.as_ref().unwrap();
            execute_list.push(
                self.send_tx(
                    &param,
                    instructions.clone(),
                    Some(TYPE_0SLOT_TRADE),
                    bl.clone(),
                    &sender
                )
            );
        }

        //bloxroute
        if send_bloxroute_or_not(
            param.bloxroute_bundle_bribe,
            &bl_bloxroute,
        ) {
            let bl = bl_bloxroute.as_ref().unwrap();
            // let mut bloxroute_ins = instructions.clone();
            //
            // let memo_ins = Instruction {
            //     program_id: Pubkey::from_str(BLOXROUTE_TRADER_API_MEMO_PROGRAM)?,
            //     accounts: Vec::new(),
            //     data: BLOXROUTE_MEMO_MARKER_MSG.as_bytes().to_vec(),
            // };
            // bloxroute_ins.push(memo_ins);

            execute_list.push(
                self.send_tx(
                    &param,
                    instructions.clone(),
                    Some(TYPE_BLOXROUTE),
                    bl.clone(),
                    &sender
                )
            );
        }

        // temporal
        if send_temporal_or_not(
            param.temporal_bundle_bribe,
            &bl_temporal,
        ) {
            let bl = bl_temporal.as_ref().unwrap();
            execute_list.push(
                self.send_tx(
                    &param,
                    instructions.clone(),
                    Some(TYPE_TEMPORAL),
                    bl.clone(),
                    &sender
                )
            );
        }

        //jito
        if send_jito_or_not(
            param.jito_bribe,
            param.block_number,
            param.simulate_bundle_bribe,
            param.second_block_bundle_bribe,
            param.land_bundle_bribe,
            &bl_jito,
        ) {
            let bl = bl_jito.as_ref().unwrap();
            execute_list.push(
                self.send_tx(
                    &param,
                    instructions.clone(),
                    Some(TYPE_JITO),
                    bl.clone(),
                    &sender
                )
            );
        }

        //nextblock
        if send_nextblock_or_not(
            param.nextblock_bundle_bribe,
            &bl_nextblock
        ) {
            let bl = bl_nextblock.as_ref().unwrap();
            execute_list.push(
                self.send_tx(
                    &param,
                    instructions.clone(),
                    Some(TYPE_NEXTBLOCK),
                    bl.clone(),
                    &sender
                ),
            );
        }

        println!("trade send done {:?}", start.elapsed());

        let response = join_all(execute_list).await;

        println!("response {:?}", response);

        let results: Vec<String> = response
            .into_iter()
            .flat_map(|result| {
                result.unwrap_or_else(|e| vec![format!("HTTP请求错误: {}", e)])
            })
            .collect();

        Ok(results)

        // for result in response {
        //     match result {
        //         Ok(signature) => {
        //             // signatures = signature.clone();
        //             println!("signature: {:?}", signature);
        //         }
        //         Err(e) => {
        //             eprintln!("{:?}", e);
        //         }
        //     }
        // }
        //
        // println!("trade rest {:?}", start.elapsed());
        //
        //
        // Ok(signatures)

    }
}
