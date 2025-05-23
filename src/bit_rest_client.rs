// we can't use reqwest::blocking::Client since it's not allowed to work with tokio

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

// ws auth
pub const V1_WS_AUTH: &str = "/v1/ws/auth";

// SPOT
pub const V1_SPOT_INSTRUMENTS: &str = "/spot/v1/instruments";
pub const V1_SPOT_ACCOUNTS: &str = "/spot/v1/accounts";
pub const V1_SPOT_ORDERS: &str = "/spot/v1/orders";
pub const V1_SPOT_CANCEL_ORDERS: &str = "/spot/v1/cancel_orders";
pub const V1_SPOT_OPENORDERS: &str = "/spot/v1/open_orders";
pub const V1_SPOT_USER_TRADES: &str = "/spot/v1/user/trades";
pub const V1_SPOT_AMEND_ORDERS: &str = "/spot/v1/amend_orders";
pub const V1_SPOT_TRANSACTION_LOGS: &str = "/spot/v1/transactions";
pub const V1_SPOT_WS_AUTH: &str = "/spot/v1/ws/auth";
pub const V1_SPOT_BATCH_ORDERS: &str = "/spot/v1/batchorders";
pub const V1_SPOT_AMEND_BATCH_ORDERS: &str = "/spot/v1/amend_batchorders";
pub const V1_SPOT_MMP_STATE: &str = "/spot/v1/mmp_state";
pub const V1_SPOT_MMP_UPDATE_CONFIG: &str = "/spot/v1/update_mmp_config";
pub const V1_SPOT_RESET_MMP: &str = "/spot/v1/reset_mmp";
pub const V1_SPOT_ACCOUNT_CONFIGS_COD: &str = "/spot/v1/account_configs/cod";
pub const V1_SPOT_ACCOUNT_CONFIGS: &str = "/spot/v1/account_configs";
pub const V1_SPOT_AGG_TRADES: &str = "/spot/v1/aggregated/trades";

// UM
pub const V1_UM_ACCOUNT_MODE: &str = "/um/v1/account_mode";
pub const V1_UM_ACCOUNTS: &str = "/um/v1/accounts";
pub const V1_UM_TRANSACTIONS: &str = "/um/v1/transactions";

// LINEAR
pub const V1_LINEAR_POSITIONS: &str = "/linear/v1/positions";
pub const V1_LINEAR_ORDERS: &str = "/linear/v1/orders";
pub const V1_LINEAR_CANCEL_ORDERS: &str = "/linear/v1/cancel_orders";
pub const V1_LINEAR_OPENORDERS: &str = "/linear/v1/open_orders";
pub const V1_LINEAR_USER_TRADES: &str = "/linear/v1/user/trades";
pub const V1_LINEAR_AMEND_ORDERS: &str = "/linear/v1/amend_orders";
pub const V1_LINEAR_EST_MARGINS: &str = "/linear/v1/margins";
pub const V1_LINEAR_CLOSE_POS: &str = "/linear/v1/close_positions";
pub const V1_LINEAR_BATCH_ORDERS: &str = "/linear/v1/batchorders";
pub const V1_LINEAR_AMEND_BATCH_ORDERS: &str = "/linear/v1/amend_batchorders";
pub const V1_LINEAR_BLOCK_TRADES: &str = "/linear/v1/blocktrades";
pub const V1_LINEAR_USER_INFO: &str = "/linear/v1/user/info";
pub const V1_LINEAR_PLATFORM_BLOCK_TRADES: &str = "/linear/v1/platform_blocktrades";
pub const V1_LINEAR_ACCOUNT_CONFIGS: &str = "/linear/v1/account_configs";
pub const V1_LINEAR_LEVERAGE_RATIO: &str = "/linear/v1/leverage_ratio";
pub const V1_LINEAR_AGG_POSITIONS: &str = "/linear/v1/aggregated/positions";
pub const V1_LINEAR_MMP_STATE: &str = "/linear/v1/mmp_state";
pub const V1_LINEAR_MMP_UPDATE_CONFIG: &str = "/linear/v1/update_mmp_config";
pub const V1_LINEAR_RESET_MMP: &str = "/linear/v1/reset_mmp";

pub struct BitRestClient {
    access_key: String,
    secret_key: String,
    base_url: String,
    client: Client,
}

impl BitRestClient {
    pub fn new(access_key: &str, secret_key: &str, base_url: &str) -> Self {
        Self {
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    fn get_nonce(&self) -> i64 {
        Utc::now().timestamp_millis()
    }

    fn encode_object(&self, value: &Value) -> String {
        match value {
            Value::Object(map) => {
                let mut sorted_keys: Vec<_> = map.keys().collect();
                sorted_keys.sort();
                sorted_keys
                    .iter()
                    .map(|k| {
                        let v = &map[*k];
                        let val = match v {
                            Value::Bool(b) => b.to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => s.clone(),
                            Value::Array(_) | Value::Object(_) => self.encode_object(v),
                            _ => v.to_string(),
                        };
                        format!("{}={}", k, val)
                    })
                    .collect::<Vec<_>>()
                    .join("&")
            }
            Value::Array(arr) => {
                let vlist = arr
                    .iter()
                    .map(|item| self.encode_object(item))
                    .collect::<Vec<_>>();
                format!("[{}]", vlist.join("&"))
            }
            _ => value.to_string(),
        }
    }

    fn get_signature(&self, _method: &str, api_path: &str, params: &Value) -> String {
        let data = format!("{}&{}", api_path, self.encode_object(params));
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC initialization failed");
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub async fn call_private_api(
        &self,
        path: &str,
        method: Method,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        param_map["timestamp"] = Value::from(self.get_nonce());
        let signature = self.get_signature(method.as_str(), path, param_map);
        param_map["signature"] = Value::String(signature);

        let url = format!("{}{}", self.base_url, path);

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-bit-access-key"),
            HeaderValue::from_str(&self.access_key).unwrap(),
        );
        headers.insert(
            HeaderName::from_static("language-type"),
            HeaderValue::from_static("1"),
        );

        let request = self.client.request(method.clone(), &url).headers(headers);

        let response = if method == Method::GET {
            let query = param_map
                .as_object()
                .unwrap_or(&Default::default())
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or(&v.to_string()).to_string()))
                .collect::<BTreeMap<_, _>>();
            request.query(&query).send().await?
        } else {
            println!("POST with param: {:?}", param_map);
            request.json(param_map).send().await?
        };

        let text = response.text().await?;
        Ok(serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text)))
    }

    /////////////////////////////////////////
    // ws functions
    /////////////////////////////////////////
    pub async fn ws_auth(&self) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_WS_AUTH, Method::GET, &mut serde_json::json!({}))
            .await;
    }

    /////////////////////////////////////////
    // um functions
    /////////////////////////////////////////
    pub async fn get_um_account_mode(&self) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_UM_ACCOUNT_MODE, Method::GET, &mut serde_json::json!({}))
            .await;
    }

    pub async fn get_um_account(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_UM_ACCOUNTS, Method::GET, param_map)
            .await;
    }

    pub async fn get_um_txlogs(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_UM_TRANSACTIONS, Method::GET, param_map)
            .await;
    }

    /////////////////////////////////////////
    // spot functions
    /////////////////////////////////////////
    pub async fn spot_get_account_configs(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_ACCOUNT_CONFIGS, Method::GET, param_map)
            .await;
    }

    pub async fn spot_get_ws_auth(&self) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_WS_AUTH, Method::GET, &mut serde_json::json!({}))
            .await;
    }

    pub async fn spot_get_class_accounts(&self) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_ACCOUNTS, Method::GET, &mut serde_json::json!({}))
            .await;
    }

    pub async fn spot_get_class_txlogs(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_TRANSACTION_LOGS, Method::GET, param_map)
            .await;
    }

    pub async fn spot_get_orders(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_ORDERS, Method::GET, param_map)
            .await;
    }

    pub async fn spot_get_open_orders(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_OPENORDERS, Method::GET, param_map)
            .await;
    }

    pub async fn spot_get_user_trades(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_USER_TRADES, Method::GET, param_map)
            .await;
    }

    pub async fn spot_new_order(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn spot_amend_order(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_AMEND_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn spot_cancel_order(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_CANCEL_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn spot_batch_new_orders(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_BATCH_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn spot_batch_amend_orders(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_AMEND_BATCH_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn spot_enable_cod(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_ACCOUNT_CONFIGS_COD, Method::POST, param_map)
            .await;
    }

    pub async fn spot_get_mmp_state(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_MMP_STATE, Method::GET, param_map)
            .await;
    }

    pub async fn spot_update_mmp_config(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_MMP_UPDATE_CONFIG, Method::POST, param_map)
            .await;
    }

    pub async fn spot_reset_mmp(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_SPOT_RESET_MMP, Method::POST, param_map)
            .await;
    }

    /////////////////////////////////////////
    // linear functions
    /////////////////////////////////////////
    pub async fn linear_get_account_configs(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_ACCOUNT_CONFIGS, Method::GET, param_map)
            .await;
    }

    pub async fn linear_get_positions(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_POSITIONS, Method::GET, param_map)
            .await;
    }

    pub async fn linear_get_orders(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_ORDERS, Method::GET, param_map)
            .await;
    }

    pub async fn linear_get_open_orders(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_OPENORDERS, Method::GET, param_map)
            .await;
    }

    pub async fn linear_get_user_trades(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_USER_TRADES, Method::GET, param_map)
            .await;
    }

    pub async fn linear_new_order(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn linear_amend_order(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_AMEND_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn linear_cancel_order(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_CANCEL_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn linear_batch_new_orders(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_BATCH_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn linear_batch_amend_orders(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_AMEND_BATCH_ORDERS, Method::POST, param_map)
            .await;
    }

    pub async fn linear_get_mmp_state(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_MMP_STATE, Method::GET, param_map)
            .await;
    }

    pub async fn linear_update_mmp_config(
        &self,
        param_map: &mut Value,
    ) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_MMP_UPDATE_CONFIG, Method::POST, param_map)
            .await;
    }

    pub async fn linear_reset_mmp(&self, param_map: &mut Value) -> Result<Value, reqwest::Error> {
        return self
            .call_private_api(V1_LINEAR_RESET_MMP, Method::POST, param_map)
            .await;
    }
}
