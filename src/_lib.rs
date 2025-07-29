pub mod request;
pub mod contracts;
pub mod configs;
pub mod utils;

use napi::bindgen_prelude::*;
use napi_derive::napi;

// #[napi(object)]
// pub struct TransactionParams {
//     pub sender: String,
//     pub receiver: String,
//     pub amount: f64,
//     pub memo: Option<String>,
// }

#[napi]
pub fn plus_100(input: u32) -> u32 {
    input + 100
}

