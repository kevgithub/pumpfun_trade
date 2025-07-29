// src/request.rs
use anyhow::{Result};
use reqwest::{
    header::{HeaderMap, HeaderValue},
};
use serde_json::json;
use std::time::Duration;
use crate::utils::get_client;

pub async fn send_bloxroute(
    api_key: &str,
    api_url: &str,
){
    let mut headers = HeaderMap::new();
    if let Ok(header_value) = HeaderValue::from_str(api_key) {
        headers.insert("Authorization", header_value);
    }

    let client = get_client();
    let _ = client
        .get(format!("http://{}/api/v2/rate-limit", api_url))
        .headers(headers)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // println!("bloxroute {:?}", response);

}

pub async fn send_nodeme(
    api_url: &str,
) {
    let client = get_client();
    let _ = client
        .get(format!("http://{}/ping", api_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // println!("nodeme {:?}", response);
}

pub async fn send_temporal(
    api_url: &str,
) {
    let client = get_client();
    let _ = client
        .get(format!("http://{}/ping", api_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // println!("temporal {:?}", response);
}

pub async fn send_0slot(
    api_url: &str,
    auth_key: &str,
) {

    let params = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getHealth"
        });

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let client = get_client();
    let _ = client
        .post(format!("http://{}?api-key={}", api_url, auth_key))
        .headers(headers)
        .json(&params)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // println!("0slot {:?}", response);

}

pub async fn send_nextblock(
    api_url: &str,
    auth_key: &str,
) {

    // let mut headers = HeaderMap::new();
    // headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    // headers.insert("Authorization", HeaderValue::from_str(&auth_key)?);
    //
    // let client = get_client();
    // let response = client
    //     .get(format!("http://{}/api/v2/rate-limit", api_url))
    //     .headers(headers)
    //     .timeout(Duration::from_secs(5))
    //     .send()
    //     .await;
    //
    // println!("{:?}", response);

}