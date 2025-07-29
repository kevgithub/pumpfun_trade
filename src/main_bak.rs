mod request;
mod contracts;
mod configs;

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
};

use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
};
use anyhow::{Error, Result, Context};

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
use solana_transaction_builder::utils::SwapParam;

use anchor_lang::Discriminator;
use anchor_spl::token::accessor::mint;
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};


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
    priority_fee: u64, 
    compute_unit_limit: u32,
}

impl TransactionBuilder {
    pub fn new(rpc_url: String, fee_payer: Keypair) -> Self {
        TransactionBuilder {
            rpc_url: rpc_url.clone(),
            rpc_client: RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed()),
            fee_payer,
            priority_fee: 1_000,
            compute_unit_limit: 200_000,
        }
    }

    pub fn set_priority_fee(&mut self, micro_lamports: u64) {
        self.priority_fee = micro_lamports;
    }

    pub fn set_compute_unit_limit(&mut self, units: u32) {
        self.compute_unit_limit = units;
    }

    pub fn check_balance(&self) -> Result<u64> {
        self.rpc_client
            .get_balance(&self.fee_payer.pubkey())
            .map_err(|e| TransactionError::RpcError(e.to_string()).into())
    }

    pub async fn build_instructions(
        &self,
        swap_param: SwapParam,
    ) -> Result<Vec<Instruction>> {

        let mut param = swap_param.clone();
        let is_buy: bool = if param.token_in.to_string() == NATIVE_MINT { true } else { false };
        let token_mint: &Pubkey = if is_buy { &Pubkey::from_str(&param.token_out)? } else { &Pubkey::from_str(&param.token_in)? };
        let buy_once = param.buy_once.unwrap_or_else(|| false);
        let mut instructions = vec![];

        // priority fee limit
        let compute_unit = param.compute_unit.unwrap_or(0);
        let compute_price = param.compute_price.unwrap_or(0);
        const MAX_FEE: u64 = 50_000_000; //0.05
        const FEE_DECIMALS: u64 = 1_000_000_000_000_000; // 10^9 * 10^6

        if compute_unit * compute_price > MAX_FEE * FEE_DECIMALS {
            param.compute_price = Some(MAX_FEE * FEE_DECIMALS / compute_unit);
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
        let token_ata_account_addr = get_associated_token_address(
            &self.fee_payer.pubkey(),
            &token_mint,
        );

        if is_buy {
            let token_ata_instruction = create_associated_token_account(
                &self.fee_payer.pubkey(),
                &self.fee_payer.pubkey(),
                &token_mint,
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
      
        let creator_vault_pubkey;
        let creator_vault: Option<&Pubkey> = match param.creator_vault {
            Some(value) => {
                creator_vault_pubkey = Pubkey::from_str(&value)?;
                Some(&creator_vault_pubkey)
            },
            None => None
        };

        if is_buy {
            let calculate_amount_out = match param.calculate_amount_out {
                Some(value) => true,
                None => false
            };
            let build_buy_instruction = pumpfun_builder.get_buy_instruction(
                &param.amount_in,
                &param.amount_out,
                &calculate_amount_out,
                &token_mint,
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
                &token_mint,
                &token_ata_account_addr,
                &self.fee_payer.pubkey(),
                creator_vault
            ).await.unwrap();
            instructions.push(build_sell_instruction);
        }

        //validate_compile
        let token_account_out = if is_buy { token_ata_account_addr } else { Pubkey::from_str(NATIVE_MINT)? };
        if (param.group.is_some() && param.group_id.is_some()) ||
            param.max_block_number.is_some() ||
            param.max_amount_out.is_some() {

            let group = param.group.unwrap_or_else(|| "trade_group".to_string());
            let group_id = param.group_id.unwrap_or_else(|| 0);
            let max_block_number = param.max_block_number.unwrap_or_else(|| 0);
            let max_amount_out = param.max_amount_out.unwrap_or_else(|| 0);

            let validate_compile_instruction = validate_compile(
                &group,
                &group_id,
                &max_block_number,
                &max_amount_out,
                &2,
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
                first_block_tip_amount,
                second_block_tip_amount,
                other_block_tip_amount,
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
                token_ata_account_addr,
                remain_token_balance
            );
            instructions.push(remain_check_instruction);
        }

        Ok(instructions)

    }


    pub async fn send_tx(
        &self,
        swap_param: SwapParam,
    ) -> Result<String> {

        let instructions = self.build_instructions(swap_param).await?;

        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .map_err(|e| TransactionError::RpcError(e.to_string()))?;

        let start = Instant::now();

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

        // 结束计时
        let duration = start.elapsed();
        println!("Message::new 执行时间: {:?}", duration);

        //start sending
        let sender = match ThirdPartySender::new(){
            Ok(sender) => sender,
            Err(e) => {
                eprintln!("Failed to create sender: {}", e);
                return Err(e);
            }
        };

        let signature_str = sender.send_node1(
            &transaction,
            "http://fra.node1.me",
            NODE1_ME_KEY,
        ).await?;
       

        Ok(signature_str)

    }

    pub async fn build_and_send_transfer(
        &self,
        to_pubkey: Pubkey,
        amount: u64,
    ) -> Result<String> {

        let compute_budget_instructions = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(self.compute_unit_limit),
            ComputeBudgetInstruction::set_compute_unit_price(self.priority_fee),
        ];

        let transfer_instruction = system_instruction::transfer(
            &self.fee_payer.pubkey(),
            &to_pubkey,
            amount,
        );

        let native_mint = Pubkey::from_str("So11111111111111111111111111111111111111111")
            .expect("Invalid pubkey string");

        let token_mint = Pubkey::from_str("Fm2FRiU2uJgj9k8AEdyvNN2g6beSVzkFAVmMATDysE41")?;
        
        let client_test_only = RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed());
        let pumpfun_builder = PumpfunProxy::new(
            client_test_only,
            Pubkey::from_str(PUMPFUN_PROGRAM_ID)?,
            Pubkey::from_str(PUMPFUN_PROXY_PROGRAM_ID)?,
        );


        let token_ata_account_addr = get_associated_token_address(
            &self.fee_payer.pubkey(),
            &token_mint,
        );

        println!("token_ata_account_addr {:?}", token_ata_account_addr);

        let token_ata_account_instruction: Option<Instruction> = match self.rpc_client.get_account(
            &token_ata_account_addr
        ) {
            Ok(account_info) => {
                None
            },
            Err(e) => {
                let token_ata_instruction = create_associated_token_account(
                    &self.fee_payer.pubkey(),
                    &self.fee_payer.pubkey(),
                    &token_mint,
                    &Pubkey::from_str(TOKEN_PROGRAM_ID)?
                );
                Some(token_ata_instruction)
            }
        };

        let build_swap_instruction = pumpfun_builder.get_buy_instruction(
            &1000u64,
            &100000u64,
            &false,
            &Pubkey::from_str("")?,
            &token_ata_account_addr,
            &self.fee_payer.pubkey(),
            None
        ).await.unwrap();
        

        println!("tx_instruction {:?}", build_swap_instruction);


        let mut instructions = vec![];
        if let Some(create_token_ata_needed) = token_ata_account_instruction {
            instructions.push(create_token_ata_needed);
        }
        instructions.push(build_swap_instruction);
        instructions.push(transfer_instruction);


        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .map_err(|e| TransactionError::RpcError(e.to_string()))?;

        let start = Instant::now();
        

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

        let duration = start.elapsed();
        println!("Message::new 执行时间: {:?}", duration);


        let sender = match ThirdPartySender::new(
         ){
            Ok(sender) => sender,
            Err(e) => {
                eprintln!("Failed to create sender: {}", e);
                return Err(e);
            }
        };


        let signature_str = sender.send_node1(
            &transaction,
            "http://fra.node1.me",
            NODE1_ME_KEY,
        ).await?;
       

        Ok(signature_str)


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
    builder.set_compute_unit_limit(50_000); // 设置更高的计算单元限制
    builder.set_priority_fee(1_000); // 设置1 micro-lamport的优先费
   
    let receiver_address = get_node1me_random_tip_account(amount);

    println!("钱包地址x: {}", receiver_address);

    match builder.check_balance() {
        Ok(balance) => {
            let sol_balance = balance as f64 / LAMPORTS_PER_SOL as f64;
            println!("当前余额: {} lamports ({} SOL)", balance, sol_balance);
        },
        Err(e) => eprintln!("无法获取余额: {}", e),
    }
   
    match builder.build_and_send_transfer(receiver_address, amount).await {
        Ok(signature) => {
            println!("✅ 自转账成功!");
            println!("交易哈希: {}", signature);
            println!("查看交易: https://explorer.solana.com/tx/{}?cluster=devnet", signature);

            // 检查新余额
            if let Ok(new_balance) = builder.check_balance() {
                let new_sol_balance = new_balance as f64 / LAMPORTS_PER_SOL as f64;
                println!("新余额: {} SOL", new_sol_balance);
            }
        },
        Err(e) => {
            eprintln!("❌ 自转账失败: {}", e);
            if let Some(TransactionError::InsufficientFunds) = e.downcast_ref() {
                eprintln!("请先获取测试币: https://faucet.solana.com/");
            }
        }
    }

    Ok(())
}