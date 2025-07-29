import {swap} from "./index.js"

const start = new Date().getTime();

console.log(
  await swap({
          connection: "https://api.mainnet-beta.solana.com",
          connectionBrand : "mainnet",
          // pub connection_send: String,
          // pub connection_send_brand: String,
          // pub trade_type : String,
          secretKey: "",
          amountIn: 2 * 10 ** 9 + '',
          amountOut: 1_000 * 10 ** 6 + '',
          fixedSide: "in",
          targetPool: "",
          tokenIn: "So11111111111111111111111111111111111111111",
          tokenOut: "FPh9d5pzRo3AK1Fonxnn1z3WWXcwEnmMEQXdbemby3DS",
          decimalsIn: 9 + '',
          decimalsOut: 6 + '',
          slippageAmount: 100 + '',
          computeUnit: 150_000 + '',
          computePrice: 5_000 + '',
          // jitoComputePrice: None,
          // bloxrouteComputePrice: None,
          // temporalComputePrice: None,
          // nextblockComputePrice: None,
          // slot0TradeComputePrice: None,
          // nodemeComputePrice: None,
          //
          // blockEngineLocate: None,
          // blockEngineUrl: None,
          // bundleBribe: None,
          // jitoBribe: 2 + 10 ** 9 + '',
          bloxrouteBundleBribe: 2 * 10 ** 9 + '',
          // temporalBundleBribe: 2 * 10 ** 9 + '',
          // nextblockBundleBribe: 2 * 10 ** 9 + '',
          // slot0TradeBundleBribe: 2 * 10 ** 9 + '',
          // nodemeBundleBribe: 2 * 10 ** 9 + '',
          // bundleAmountOut: None,
          //
          buyOnce: true,
          // maxBlockNumber: None,
          // tokenBalance: None,
          // tokenMint: None,
          // marketId: None,
          //
          // skipRetry: None,
          // groupId: None,
          // // group: Some("7uQxuvVFzY3uApna9PFQEMQEUKrrwFYG4D4272aPHT1d".to_string()),
          // group: None,
          //
          // // tradeMode?: number,
          // tradeTimes: None,
          //
          // recentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
          // jitoRecentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
          // bloxrouteRecentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
          // temporalRecentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
          // nextblockRecentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
          // slot0TradeRecentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
          // nodemeRecentBlockHash: "6tnfawsUrS5vAwQtysT5pjiDrCFhQxpagVTJ3NnKHZCh",
    //
          // solAccount: None,
          // coinAccount: None,
          //
          // maxAmountOut: None,
          // bundleType: None,
          //
          // // remainTokenBalance: Some(5555),
          // remainTokenBalance: None,
          //
          // // blockNumber: Some(499),
          // // simulateBundleBribe: Some(1_000_000),
          // // secondBlockBundleBribe: Some(2_000_000),
          // // landBundleBribe: Some(3_000_000),
          // blockNumber: None,
          // simulateBundleBribe: None,
          // secondBlockBundleBribe: None,
          // landBundleBribe: None,
          //
          // calculateAmountOut: None,
          //
          // trackTokenAccount: None,
          // trackAmountIn: None,
          // trackSlippage: None,
          // trackMaxAllowBuy: None,
          // trackTokenBalance: None,
          //
          // snipeRaydiumSolReserve: None,
          //
          // sendNormalTrade: None, //0 no, 1 yes
          // tradeManualLocalRpc: None, //0 no, 1 yes, only for trade_manual
          //
          // antiMev: None, // third-party api prevents MEV, 0 no, 1 yes
          //
          // creatorVault: None,
  })
);
console.log('code cost', new Date().getTime() - start);