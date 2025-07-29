
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use solana_program::native_token::LAMPORTS_PER_SOL;

pub const SENDER_JITO :u8 = 0;
pub const SENDER_BLOXROUTE :u8 = 1;
pub const SENDER_TEMPORAL :u8 = 2;
pub const SENDER_NEXT_BLOCK :u8 = 3;
pub const SENDER_SLOT0_TRADE :u8 = 4;
pub const SENDER_NODE1_ME :u8 = 5;


pub fn get_random_tip_account(tip_accounts: &[&str], bribe: &u64) -> Pubkey {

    let value = ((*bribe as f64).sin().abs() * 1e5).floor();
    let index = (value as usize) % tip_accounts.len();

    match Pubkey::from_str(&tip_accounts[index]) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            //fetch the first one
            tip_accounts.iter()
                .find_map(|s| Pubkey::from_str(s).ok())
                .unwrap_or_else(|| panic!("tip_accounts 中没有有效的地址"))
        }
    }
}

//node1.me
const NODE1ME_TIP_ACCOUNTS: &[&str] = &[
    "node1PqAa3BWWzUnTHVbw8NJHC874zn9ngAkXjgWEej",
    "node1UzzTxAAeBTpfZkQPJXBAqixsbdth11ba1NXLBG",
    "node1Qm1V4fwYnCurP8otJ9s5yrkPq7SPZ5uhj3Tsv",
    "node1PUber6SFmSQgvf2ECmXsHP5o3boRSGhvJyPMX1",
    "node1AyMbeqiVN6eoQzEAwCA6Pk826hrdqdAHR7cdJ3",
    "node1YtWCoTwwVYTFLfS19zquRQzYX332hs1HEuRBjC",
];

pub fn get_node1me_random_tip_account(bribe: &u64) -> Pubkey {
    get_random_tip_account(NODE1ME_TIP_ACCOUNTS, bribe)
}

//0slot.trade
const SLOT0_TRADE_TIP_ACCOUNTS: &[&str] = &[
    "6fQaVhYZA4w3MBSXjJ81Vf6W1EDYeUPXpgVQ6UQyU1Av",
    "4HiwLEP2Bzqj3hM2ENxJuzhcPCdsafwiet3oGkMkuQY4",
    "7toBU3inhmrARGngC7z6SjyP85HgGMmCTEwGNRAcYnEK",
    "8mR3wB1nh4D6J9RUCugxUpc6ya8w38LPxZ3ZjcBhgzws",
    "6SiVU5WEwqfFapRuYCndomztEwDjvS5xgtEof3PLEGm9",
    "TpdxgNJBWZRL8UXF5mrEsyWxDWx9HQexA9P1eTWQ42p",
    "D8f3WkQu6dCF33cZxuAsrKHrGsqGP2yvAHf8mX6RXnwf",
    "GQPFicsy3P3NXxB5piJohoxACqTvWE9fKpLgdsMduoHE",
    "Ey2JEr8hDkgN8qKJGrLf2yFjRhW7rab99HVxwi5rcvJE",
    "4iUgjMT8q2hNZnLuhpqZ1QtiV8deFPy2ajvvjEpKKgsS",
    "3Rz8uD83QsU8wKvZbgWAPvCNDU6Fy8TSZTMcPm3RB6zt",
    "DiTmWENJsHQdawVUUKnUXkconcpW4Jv52TnMWhkncF6t",
    "HRyRhQ86t3H4aAtgvHVpUJmw64BDrb61gRiKcdKUXs5c",
    "7y4whZmw388w1ggjToDLSBLv47drw5SUXcLk6jtmwixd",
    "J9BMEWFbCBEjtQ1fG5Lo9kouX1HfrKQxeUxetwXrifBw",
    "8U1JPQh3mVQ4F5jwRdFTBzvNRQaYFQppHQYoH38DJGSQ",
    "Eb2KpSC8uMt9GmzyAEm5Eb1AAAgTjRaXWFjKyFXHZxF3",
    "FCjUJZ1qozm1e8romw216qyfQMaaWKxWsuySnumVCCNe",
    "ENxTEjSQ1YabmUpXAdCgevnHQ9MHdLv8tzFiuiYJqa13",
    "6rYLG55Q9RpsPGvqdPNJs4z5WTxJVatMB8zV3WJhs5EK",
    "Cix2bHfqPcKcM233mzxbLk14kSggUUiz2A87fJtGivXr",
];

pub fn  get_slot0_trade_random_tip_account(bribe: &u64) -> Pubkey {
    get_random_tip_account(SLOT0_TRADE_TIP_ACCOUNTS, bribe)
}


//temporal

const TEMPORAL_TIP_ACCOUNTS: &[&str] = &[
    "TEMPaMeCRFAS9EKF53Jd6KpHxgL47uWLcpFArU1Fanq",
    "noz3jAjPiHuBPqiSPkkugaJDkJscPuRhYnSpbi8UvC4",
    "noz3str9KXfpKknefHji8L1mPgimezaiUyCHYMDv1GE",
    "noz6uoYCDijhu1V7cutCpwxNiSovEwLdRHPwmgCGDNo",
    "noz9EPNcT7WH6Sou3sr3GGjHQYVkN3DNirpbvDkv9YJ",
    "nozc5yT15LazbLTFVZzoNZCwjh3yUtW86LoUyqsBu4L",
    "nozFrhfnNGoyqwVuwPAW4aaGqempx4PU6g6D9CJMv7Z",
    "nozievPk7HyK1Rqy1MPJwVQ7qQg2QoJGyP71oeDwbsu",
    "noznbgwYnBLDHu8wcQVCEw6kDrXkPdKkydGJGNXGvL7",
    "nozNVWs5N8mgzuD3qigrCG2UoKxZttxzZ85pvAQVrbP",
    "nozpEGbwx4BcGp6pvEdAh1JoC2CQGZdU6HbNP1v2p6P",
    "nozrhjhkCr3zXT3BiT4WCodYCUFeQvcdUkM7MqhKqge",
    "nozrwQtWhEdrA6W8dkbt9gnUaMs52PdAv5byipnadq3",
    "nozUacTVWub3cL4mJmGCYjKZTnE9RbdY5AP46iQgbPJ",
    "nozWCyTPppJjRuw2fpzDhhWbW355fzosWSzrrMYB1Qk",
    "nozWNju6dY353eMkMqURqwQEoM3SFgEKC6psLCSfUne",
    "nozxNBgWohjR75vdspfxR5H9ceC7XXH99xpxhVGt3Bb",
];

pub fn get_temporal_random_tip_account(bribe: &u64) -> Pubkey {
    get_random_tip_account(TEMPORAL_TIP_ACCOUNTS, bribe)
}

//bloxroute

const BLOXROUTE_TIP_ACCOUNTS: &[&str] = &[
    "HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY",
    "95cfoy472fcQHaw4tPGBTKpn6ZQnfEPfBgDQx6gcRmRg",
    "3UQUKjhMKaY2S6bjcQD6yHB7utcZt5bfarRCmctpRtUd",
    "FogxVNs6Mm2w9rnGL1vkARSwJxvLE8mujTv3LK8RnUhF",
];

pub fn get_random_bloxroute_tip_account(bribe: &u64) -> Pubkey {
    get_random_tip_account(BLOXROUTE_TIP_ACCOUNTS, bribe)
}

//nextblock

const NEXTBLOCK_TIP_ACCOUNTS: &[&str] = &[
    "NextbLoCkVtMGcV47JzewQdvBpLqT9TxQFozQkN98pE",
    "NexTbLoCkWykbLuB1NkjXgFWkX9oAtcoagQegygXXA2",
    "NeXTBLoCKs9F1y5PJS9CKrFNNLU1keHW71rfh7KgA1X",
    "NexTBLockJYZ7QD7p2byrUa6df8ndV2WSd8GkbWqfbb",
    "neXtBLock1LeC67jYd1QdAa32kbVeubsfPNTJC1V5At",
    "nEXTBLockYgngeRmRrjDV31mGSekVPqZoMGhQEZtPVG",
    "NEXTbLoCkB51HpLBLojQfpyVAMorm3zzKg7w9NFdqid",
    "nextBLoCkPMgmG8ZgJtABeScP35qLa2AMCNKntAP7Xc",
];

pub fn get_random_nextblock_tip_account(bribe: &u64) -> Pubkey {
    get_random_tip_account(NEXTBLOCK_TIP_ACCOUNTS, bribe)
}



//jito

const JITO_TIP_ACCOUNTS: &[&str] = &[
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

pub fn get_random_jito_tip_account(bribe: &u64) -> Pubkey {
    get_random_tip_account(JITO_TIP_ACCOUNTS, bribe)
}


pub fn send_normal_or_not(send_normal_trade: bool) -> bool{
    send_normal_trade
}

pub fn send_jito_or_not(
    jito_bribe: Option<u64>,
    block_number: Option<u64>,
    first_block_bundle_bribe: Option<u64>,
    second_block_bundle_bribe: Option<u64>,
    other_block_bundle_bribe: Option<u64>,
    block_engine_url: &Option<String>,
) -> bool {
    if let Some(bl) = block_engine_url {
        if let Some(bribe) = jito_bribe { true }
        else {
            if block_number.is_some() &&
                first_block_bundle_bribe.is_some() &&
                second_block_bundle_bribe.is_some() &&
                other_block_bundle_bribe.is_some() { true } else { false }
        }
    }
    else { false }
}


pub fn send_slot0_trade_or_not(
    slot0_trade_bribe: Option<u64>,
    block_engine_url: &Option<String>,
) -> bool {
    const MIN_BRIBE: u64 = 100_000; // 0.0001
    if let Some(block_engine) = block_engine_url {
        if let Some(bribe) = slot0_trade_bribe {
            if bribe >= MIN_BRIBE { true } else { false }
        }
        else { false }
    }
    else { false }
}

pub fn send_bloxroute_or_not(
    bloxroute_bribe: Option<u64>,
    block_engine_url: &Option<String>,
) -> bool {
    const MIN_BRIBE: u64 = 2_000_000; // 0.002
    if let Some(block_engine) = block_engine_url {
        if let Some(bribe) = bloxroute_bribe {
            if bribe >= MIN_BRIBE { true } else { false }
        }
        else { false }
    }
    else { false }
}

pub fn send_temporal_or_not(
    temporal_bribe: Option<u64>,
    block_engine_url: &Option<String>,
) -> bool {
    const MIN_BRIBE: u64 = 1_000_000; // 0.001
    if let Some(block_engine) = block_engine_url {
        if let Some(bribe) = temporal_bribe {
            if bribe >= MIN_BRIBE { true } else { false }
        }
        else { false }
    }
    else { false }
}

pub fn send_nextblock_or_not(
    nextblock_bribe: Option<u64>,
    block_engine_url: &Option<String>,
) -> bool {
    const MIN_BRIBE: u64 = 1_000_000; // 0.001
    if let Some(block_engine) = block_engine_url {
        if let Some(bribe) = nextblock_bribe {
            if bribe >= MIN_BRIBE { true } else { false }
        }
        else { false }
    }
    else { false }
}

pub fn send_node1me_or_not(
    node1me_bribe: Option<u64>,
    block_engine_url: &Option<String>,
) -> bool {

    const MIN_BRIBE: u64 = 2_000_000; // 0.002
    if let Some(block_engine) = block_engine_url {
        if let Some(bribe) = node1me_bribe {
            if bribe >= MIN_BRIBE { true } else { false }
        }
        else { false }
    }
    else { false }
}








