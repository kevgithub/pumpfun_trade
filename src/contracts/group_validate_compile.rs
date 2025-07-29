


use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use borsh::{BorshSerialize, BorshDeserialize};
use std::str::FromStr;
use crate::configs::global::VALIDATE_COMPILE_PROGRAM_ID;
use crate::configs::global::GROUP_VALIDATOR_ACCOUNT;

const GROUP_TRADE_MAP: [u64; 5] = [0, 1, 2, 3, 4]; // 假设的映射表

// 定义指令数据结构
#[derive(BorshSerialize, BorshDeserialize)]
struct ValidateCompileData {
    group_key: u64,
    group_id: u64,
    block_number: u64,
    max_amount_out: u64,
    max_amount_out_direction: u64,
}

pub fn validate_compile(
    group: &str,
    group_id: u64,
    block_number: u64,
    max_amount_out: u64,
    max_amount_out_direction: u64,
    ata: &Pubkey,
) -> Instruction {

    let group_key = match group {
        "" => 3,
        "" => 4,
        _ => 3,
    };

    let data = ValidateCompileData {
        group_key,
        group_id,
        block_number,
        max_amount_out,
        max_amount_out_direction,
    };

    let mut instruction_data = Vec::new();
    data.serialize(&mut instruction_data).unwrap();

    // 构建指令
    Instruction {
        program_id: Pubkey::from_str(VALIDATE_COMPILE_PROGRAM_ID).unwrap(),
        accounts: vec![
            AccountMeta::new(
                Pubkey::from_str(GROUP_VALIDATOR_ACCOUNT).unwrap(),
                false,
            ),
            AccountMeta::new(*ata, false),
        ],
        data: instruction_data,
    }
}