use solana_program::pubkey::Pubkey;
use solana_program::instruction::{AccountMeta, Instruction};
use std::str::FromStr;
use crate::configs::global::REMAIN_BALANCE_CHECK_PROGRAM_ID;

const METHOD_CHECK: u64 = 19565707223;

pub fn remain_balance_check(
    coin_ata: Pubkey,
    remain_coin_balance: u64,
) -> Instruction {

    let accounts = vec![
        AccountMeta::new(coin_ata, false),
    ];

    let remain_coin_balance_program_id = Pubkey::from_str(REMAIN_BALANCE_CHECK_PROGRAM_ID).unwrap();

    let build_swap_instruction = Instruction::new_with_bincode(
        remain_coin_balance_program_id,
        &(METHOD_CHECK, remain_coin_balance),
        accounts,
    );

    build_swap_instruction
}