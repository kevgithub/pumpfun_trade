
use anyhow::{anyhow, Result, Context};
use reqwest::Client;
use solana_program::{
    system_program,
    pubkey::Pubkey,
    program_pack::Pack,
    sysvar::rent::Rent,
};
use spl_associated_token_account::get_associated_token_address;
use borsh::{BorshDeserialize, BorshSerialize, from_slice};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::{AccountMeta, Instruction};
use crate::configs::global::*;

use solana_program::system_program::ID as SYSTEM_PROGRAM_ID;
use std::str::FromStr;
use std::time::Instant;

const BONDING_CURVE_SEED: &str = "bonding-curve";
const METHOD_PROXY_BUY: u64 = 04974560345395; //buy
const METHOD_PROXY_SELL: u64 = 15265570577868; //sell

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct BondingCurveInfo {
    pub discriminator: u64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
}

pub struct PumpfunProxy {
    connection: RpcClient,
    pumpfun_program_id: Pubkey,
    pumpfun_proxy_program_id: Pubkey,
}

impl PumpfunProxy {

    pub fn new(
        connection: RpcClient,
        pumpfun_program_id: Pubkey,
        pumpfun_proxy_program_id: Pubkey,
    ) -> Self  {
        PumpfunProxy {
            connection,
            pumpfun_program_id,
            pumpfun_proxy_program_id
        }
    }

    pub fn get_bonding_curve_pda(
        &self,
        mint: &Pubkey,
    ) -> Pubkey {
        let (bonding_curve, _bump) = Pubkey::find_program_address(
            &[
                BONDING_CURVE_SEED.as_bytes(),
                mint.as_ref(),
            ],
            &self.pumpfun_program_id,
        );
        bonding_curve
    }

    pub async fn get_bonding_curve_info(
        &self,
        bonding_curve: &Pubkey,
    ) -> Result<BondingCurveInfo, Box<dyn std::error::Error>> {

        let account_data = self.connection.get_account_data(bonding_curve)
            .map_err(|_| "Bonding curve account not found or has no data")?;

        let bonding_curve_account = from_slice::<BondingCurveInfo>(&account_data[..81])
            .map_err(|e| {
                anyhow!(
                "Failed to deserialize bonding curve account: {}",
                e.to_string()
            )
            })?;

        Ok(bonding_curve_account)

    }

    pub async fn get_creator_vault(
        &self,
        bonding_curve: &Pubkey,
    ) -> Pubkey {
        let account_info = self.get_bonding_curve_info(bonding_curve).await.unwrap();
        let creator = account_info.creator;

        let (creator_vault, _bump) = Pubkey::find_program_address(
            &[
                b"creator-vault",
                creator.as_ref()
            ],
            &self.pumpfun_program_id
        );
        creator_vault
    }

    pub async fn get_accounts(
        &self,
        mint: &Pubkey,
        coin_ata: &Pubkey,
        payer: &Pubkey,
        creator_vault: Option<&Pubkey>,
    ) -> Result<Vec<AccountMeta>, Box<dyn std::error::Error>> {
        let bonding_curve = self.get_bonding_curve_pda(mint);
        let associated_bonding_curve = get_associated_token_address(&bonding_curve, mint);

        let creator_vault_get = match creator_vault {
            Some(v) => *v,
            None => self.get_creator_vault(&bonding_curve).await,
        };

        Ok(
            vec![
                AccountMeta::new(Pubkey::from_str("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf")?, false),
                AccountMeta::new(Pubkey::from_str(PUMPFUN_FEE_RECIPIENT)?, false),
                AccountMeta::new(mint.clone(), false),
                AccountMeta::new(bonding_curve, false),
                AccountMeta::new(associated_bonding_curve, false),
                AccountMeta::new(coin_ata.clone(), false),
                AccountMeta::new(payer.clone(), false),
                AccountMeta::new(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new(Pubkey::from_str(TOKEN_PROGRAM_ID)?, false),
                AccountMeta::new(creator_vault_get, false),
                AccountMeta::new(Pubkey::from_str("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1")?, false),
                AccountMeta::new(self.pumpfun_program_id, false)
            ]
        )
    }

    pub async fn get_buy_instruction(
        &self,
        amount: &u64,
        max_sol_cost: &u64,
        cal_amount_out: &bool,
        mint: &Pubkey,
        coin_ata: &Pubkey,
        payer: &Pubkey,
        creator_vault: Option<&Pubkey>,
    ) -> Result<Instruction> {
        let accounts = self.get_accounts(mint, coin_ata, payer, creator_vault).await.unwrap();
        let cal_amount_out_u64 = if cal_amount_out.clone() { 1 } else { 0 };
        let build_swap_instruction = Instruction::new_with_bincode(
            self.pumpfun_proxy_program_id,
            &(METHOD_PROXY_BUY, amount, max_sol_cost, cal_amount_out_u64),
            accounts,
        );
        Ok(build_swap_instruction)
    }

    pub async fn get_sell_instruction(
        &self,
        amount: &u64,
        min_sol_output: &u64,
        mint: &Pubkey,
        coin_ata: &Pubkey,
        payer: &Pubkey,
        creator_vault: Option<&Pubkey>,
    ) -> Result<Instruction> {
        let accounts = self.get_accounts(mint, coin_ata, payer, creator_vault).await.unwrap();
        let build_swap_instruction = Instruction::new_with_bincode(
            self.pumpfun_proxy_program_id,
            &(METHOD_PROXY_SELL, amount, min_sol_output),
            accounts,
        );
        Ok(build_swap_instruction)
    }

}