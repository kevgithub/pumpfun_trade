use solana_program::native_token::LAMPORTS_PER_SOL;
// src/conversion.rs
use crate::utils::*;

// impl From<SolAccountStructJs> for SolAccountStruct {
//     fn from(js: SolAccountStructJs) -> Self {
//         SolAccountStruct {
//             public_key: js.public_key,
//             seed: js.seed,
//         }
//     }
// }

impl From<SwapParam4Node> for SwapParam {
    fn from(node: SwapParam4Node) -> Self {
        SwapParam {
            connection: node.connection,
            connection_brand : node.connection_brand,
            secret_key: node.secret_key,
            amount_in: node.amount_in.parse().unwrap(),
            amount_out: node.amount_out.parse().unwrap(),
            fixed_side: node.fixed_side,
            target_pool: node.target_pool,
            token_in: node.token_in,
            token_out: node.token_out,
            decimals_in: node.decimals_in.parse().unwrap(),
            decimals_out: node.decimals_out.parse().unwrap(),
            slippage_amount: node.slippage_amount.parse().unwrap(),
            compute_unit: node.compute_unit.map(|s| s.parse().unwrap()),
            compute_price: node.compute_price.map(|s| s.parse().unwrap()),
            jito_compute_price: node.jito_compute_price.map(|s| s.parse().unwrap()),
            bloxroute_compute_price: node.bloxroute_compute_price.map(|s| s.parse().unwrap()),
            temporal_compute_price: node.temporal_compute_price.map(|s| s.parse().unwrap()),
            nextblock_compute_price: node.nextblock_compute_price.map(|s| s.parse().unwrap()),
            slot0_trade_compute_price: node.slot0_trade_compute_price.map(|s| s.parse().unwrap()),
            nodeme_compute_price: node.nodeme_compute_price.map(|s| s.parse().unwrap()),

            block_engine_locate: node.block_engine_locate,
            block_engine_url: node.block_engine_url,
            bundle_bribe: node.bundle_bribe.map(|s| s.parse().unwrap()),
            jito_bribe: node.jito_bribe.map(|s| s.parse().unwrap()),
            bloxroute_bundle_bribe: node.bloxroute_bundle_bribe.map(|s| s.parse().unwrap()),
            temporal_bundle_bribe: node.temporal_bundle_bribe.map(|s| s.parse().unwrap()),
            nextblock_bundle_bribe: node.nextblock_bundle_bribe.map(|s| s.parse().unwrap()),
            slot0_trade_bundle_bribe: node.slot0_trade_bundle_bribe.map(|s| s.parse().unwrap()),
            nodeme_bundle_bribe: node.nodeme_bundle_bribe.map(|s| s.parse().unwrap()),
            bundle_amount_out: node.bundle_amount_out.map(|s| s.parse().unwrap()),

            buy_once: node.buy_once,
            max_block_number: node.max_block_number.map(|s| s.parse().unwrap()),
            token_balance: node.token_balance.map(|s| s.parse().unwrap()),
            token_mint: node.token_mint,
            market_id: node.market_id,

            skip_retry: node.skip_retry,
            group_id: node.group_id.map(|s| s.parse().unwrap()),
            group: node.group,

            // tradeMode?: number,
            trade_times: node.trade_times.map(|s| s.parse().unwrap()),

            recent_block_hash: node.recent_block_hash,
            jito_recent_block_hash: node.jito_recent_block_hash,
            bloxroute_recent_block_hash: node.bloxroute_recent_block_hash,
            temporal_recent_block_hash: node.temporal_recent_block_hash,
            nextblock_recent_block_hash: node.nextblock_recent_block_hash,
            slot0_trade_recent_block_hash: node.slot0_trade_recent_block_hash,
            nodeme_recent_block_hash: node.nodeme_recent_block_hash,

            sol_account: node.sol_account,
            coin_account: node.coin_account,

            max_amount_out: node.max_amount_out.map(|s| s.parse().unwrap()),
            bundle_type: node.bundle_type.map(|s| s.parse().unwrap()),

            remain_token_balance: node.remain_token_balance.map(|s| s.parse().unwrap()),

            block_number: node.block_number.map(|s| s.parse().unwrap()),
            simulate_bundle_bribe: node.simulate_bundle_bribe.map(|s| s.parse().unwrap()),
            second_block_bundle_bribe: node.second_block_bundle_bribe.map(|s| s.parse().unwrap()),
            land_bundle_bribe: node.land_bundle_bribe.map(|s| s.parse().unwrap()),

            calculate_amount_out: node.calculate_amount_out,

            track_token_account: node.track_token_account,
            track_amount_in: node.track_amount_in.map(|s| s.parse().unwrap()),
            track_slippage: node.track_slippage.map(|s| s.parse().unwrap()),
            track_max_allow_buy: node.track_max_allow_buy.map(|s| s.parse().unwrap()),
            track_token_balance: node.track_token_balance.map(|s| s.parse().unwrap()),

            snipe_raydium_sol_reserve: node.snipe_raydium_sol_reserve.map(|s| s.parse().unwrap()),

            send_normal_trade: node.send_normal_trade.map(|s| s.parse().unwrap()), //0 no, 1 yes
            trade_manual_local_rpc: node.trade_manual_local_rpc.map(|s| s.parse().unwrap()), //0 no, 1 yes, only for trade_manual

            anti_mev: node.anti_mev.map(|s| s.parse().unwrap()), // third-party api prevents MEV, 0 no, 1 yes

            creator_vault: node.creator_vault,

        }
    }
}