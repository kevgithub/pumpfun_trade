
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use crate::configs::bribe::get_random_jito_tip_account;
use crate::configs::global::{JITO_TRICK_PROGRAM_ID};
use solana_program::system_program::ID as SYSTEM_PROGRAM_ID;
use std::str::FromStr;
const METHOD_TRADE: u64 = 9425677158898;

pub fn jito_trick_trade(
    wallet: Pubkey,
    first_block_tip_amount: &u64,
    second_block_tip_amount: &u64,
    other_block_tip_amount: &u64,
    block_number: u64,
) -> Instruction {

    let tip_account = get_random_jito_tip_account(first_block_tip_amount);
    let jito_trick_program_id = Pubkey::from_str(JITO_TRICK_PROGRAM_ID).unwrap();
    let block_number_encrypt = block_number;

    let accounts = vec![
        AccountMeta::new(wallet, false),
        AccountMeta::new(tip_account, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
    ];

    let build_swap_instruction = Instruction::new_with_bincode(
        jito_trick_program_id,
        &(METHOD_TRADE, first_block_tip_amount, second_block_tip_amount, other_block_tip_amount, block_number_encrypt),
        accounts,
    );

    build_swap_instruction
}