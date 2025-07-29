// src/request.rs
use anyhow::{anyhow, Result, Context};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION},
};
use solana_sdk::{
    transaction::VersionedTransaction,
    signature::Signature
};
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct SignedTransaction {
    pub content: String,
    pub is_cleanup: bool,
}

#[derive(Debug, Serialize)]
struct TransactionContent {
    content: String,
    is_cleanup: bool,
}

#[derive(Debug, Serialize)]
struct SubmitParams {
    transaction: TransactionContent,
    skip_pre_flight: bool,
    #[serde(rename = "frontRunningProtection")]
    front_running_protection: bool,
    #[serde(rename = "fastBestEffort")]
    fast_best_effort: bool,
}

#[derive(Debug, Deserialize)]
struct SubmitResponse {
    signature: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

pub struct ThirdPartySender {
    client: Client,
    api_url: String,
    auth_key: String,
}

impl ThirdPartySender {

    pub fn new(api_url: String, auth_key: String) -> Result<Self> {
        Ok(ThirdPartySender {
            client: Client::new(),
            api_url,
            auth_key,
        })
        // let headers = Self::build_headers(&auth_key)?;
        //
        // let client = Client::builder()
        //     .default_headers(headers)
        //     // .pool_idle_timeout(None)
        //     // .pool_max_idle_per_host(200)
        //     // .tcp_keepalive(Some(std::time::Duration::from_secs(15)))
        //     .build()
        //     .context("Failed to create HTTP client")?;
        //
        // Ok(ThirdPartySender {
        //     client,
        //     api_url,
        //     auth_key,
        // })
    }

    fn build_headers(auth_key: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        headers.insert(
            "api-key",
            HeaderValue::from_str(auth_key)
                .map_err(|e| anyhow!("Invalid auth header: {}", e))?,
        );
        println!("api-key {:?}", auth_key);
        // headers.insert("x-sdk", HeaderValue::from_static("rust-client"));
        // headers.insert(
        //     "x-sdk-version",
        //     HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
        // );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/json"),
        );
        

        Ok(headers)
    }

    pub async fn fetch_data(&self) -> Result<String> {
        let response = self.client.get(&format!("{}/api/v2/rate-limit", self.api_url))
            .send()
            .await
            .context("Failed to send GET request")?;

        let body = response.text()
            .await
            .context("Failed to read response body")?;

        Ok(body)
    }

    pub async fn send_node1(
        &self,
        transaction: &VersionedTransaction,
        skip_pre_flight: bool,
        front_running_protection: bool,
        fast_best_effort: bool,
    ) -> Result<String> {
        let serialized = bincode::serialize(&transaction)?;
        let content = general_purpose::STANDARD.encode(serialized);
        

        println!("transaction {:?}", transaction);
        
        let params = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "sendTransaction",
                "params": [
                  content,
                  {
                    "encoding": "base64",
                    "skipPreflight": true,
                    "preflightCommitment": "processed"
                  }
                ],
              });

        println!("params {:?}", params);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("api-key", HeaderValue::from_str(&self.auth_key)?);
      

        // 发送请求
        let response = self.client
            .post(format!("{}", self.api_url))
            .headers(headers)
            .json(&params)
            .send()
            .await
            .context("Failed to send request")?;

        println!("response {:?}", response);

        let response_text = response.text().await?;
        println!("response_text: {:?}", response_text);
        Ok(response_text)


        // let response_data = response
        //     .json()
        //     .await
        //     .context("Failed to parse response")?;
        //
        // print!("response_data {:?}", response_data);

        // match response_data {
        //     SubmitResponse { signature: Some(sig), .. } => Ok(sig),
        //     SubmitResponse { error: Some(err), .. } => {
        //         Err(anyhow::anyhow!("API error: {}", err))
        //     },
        //     _ => Err(anyhow::anyhow!("Unexpected response format")),
        // }

    }
}