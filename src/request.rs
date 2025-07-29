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
use std::sync::OnceLock;
use std::time::Duration;
use crate::utils::get_client;

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
    client: &'static Client,
    // api_url: String,
    // auth_key: String,
}


impl ThirdPartySender {

    pub fn new(
        // api_url: String, auth_key: String
    ) -> Result<Self> {
        let client = get_client();
        Ok(ThirdPartySender {
            client,
            // api_url,
            // auth_key,
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

    // pub async fn fetch_data(&self) -> Result<String> {
    //     let response = self.client.get(&format!("{}/api/v2/rate-limit", self.api_url))
    //         .send()
    //         .await
    //         .context("Failed to send GET request")?;
    //
    //     let body = response.text()
    //         .await
    //         .context("Failed to read response body")?;
    //
    //     Ok(body)
    // }


    pub fn pre_handle_transaction(
        &self,
        transaction: &VersionedTransaction
    ) -> Result<String>{
        let serialized = bincode::serialize(&transaction)?;
        let content = general_purpose::STANDARD.encode(serialized);
        Ok(content)
    }


    pub async fn send_node1(
        &self,
        transaction: &VersionedTransaction,
        api_url: &str,
        auth_key: &str,
    ) -> Result<String> {
        let content = self.pre_handle_transaction(transaction)?;
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

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("api-key", HeaderValue::from_str(&auth_key)?);
        // headers.insert("x-sdk", HeaderValue::from_static("rust-client"));
        // headers.insert(
        //     "x-sdk-version",
        //     HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
        // );

        let response = self.client
            .post(format!("http://{}", api_url))
            .headers(headers)
            .json(&params)
            .send()
            .await
            .context("Failed to send request")?;

        let response_text = response.text().await?;
        Ok(response_text)

    }

    pub async fn send_0slot(
        &self,
        transaction: &VersionedTransaction,
        api_url: &str,
        auth_key: &str,
    ) -> Result<String> {
        let content = self.pre_handle_transaction(transaction)?;

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
            ]
        });

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));


        let response = self.client
            .post(format!("http://{}?api-key={}", api_url, auth_key))
            .headers(headers)
            .json(&params)
            .send()
            .await
            .context("Failed to send request")?;

        let response_text = response.text().await?;
        Ok(response_text)
    }


    pub async fn send_temporal(
        &self,
        transaction: &VersionedTransaction,
        api_url: &str,
        auth_key: &str,
    ) -> Result<String> {
        let content = self.pre_handle_transaction(transaction)?;

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
                ]
              });

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let response = self.client
            .post(format!("http://{}?c={}", api_url, auth_key))
            .headers(headers)
            .json(&params)
            .send()
            .await
            .context("Failed to send request")?;

        let response_text = response.text().await?;
        Ok(response_text)
    }


    pub async fn send_jito(
        &self,
        transaction: &VersionedTransaction,
        api_url: &str,
        auth_key: &str,
    ) -> Result<String> {
        let content = self.pre_handle_transaction(transaction)?;

        let params = json!({
          "id": 1,
          "jsonrpc": "2.0",
          "method": "sendBundle",
          "params": [
            [content],
            {
              "encoding": "base64"
            }
          ]
        });

        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("x-jito-auth", HeaderValue::from_str(&auth_key)?);

        let response = self.client
            // .post(format!("{}/api/v1/transactions?bundleOnly=true", api_url))
            .post(format!("https://{}/api/v1/bundles", api_url))
            .headers(headers)
            .json(&params)
            .send()
            .await
            .context("Failed to send request")?;

        let response_text = response.text().await?;
        Ok(response_text)
    }


    pub async fn send_nextblock(
        &self,
        transaction: &VersionedTransaction,
        api_url: &str,
        auth_key: &str,
    ) -> Result<String> {
        //https://
        Ok("nextbock".to_string()
        )
    }

    pub async fn send_bloxroute(
        &self,
        transaction: &VersionedTransaction,
        api_url: &str,
        auth_key: &str,
    ) -> Result<String> {
        let content = self.pre_handle_transaction(transaction)?;

        let params = json!({
            "transaction": {"content": content, "isCleanup": false },
            "skipPreFlight": true,
            "frontRunningProtection": false,
            "useStakedRPCs": true,
        });

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Authorization", HeaderValue::from_str(&auth_key)?);

        let response = self.client
            .post(format!("http://{}/api/v2/submit", api_url))
            .headers(headers)
            .json(&params)
            .send()
            .await
            .context("Failed to send request")?;

        println!("response {:?}", response);

        let response_text = response.text().await?;
        Ok(response_text)
    }

}