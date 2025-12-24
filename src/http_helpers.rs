use crate::{TransactionRequest, errors::RelayerApiException};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(30);

pub enum RequestData {
    Value(Value),
    TransactionRequest(TransactionRequest),
}

pub fn request(
    endpoint: &str,
    method: &str,
    headers: Option<reqwest::header::HeaderMap>,
    data: Option<&RequestData>,
) -> Result<Value, RelayerApiException> {
    let client = Client::builder().timeout(TIMEOUT).build().map_err(|e| {
        RelayerApiException::from_request_error(format!("Failed to create client: {}", e))
    })?;

    let mut req = match method {
        "GET" => client.get(endpoint),
        "POST" => client.post(endpoint),
        "DELETE" => client.delete(endpoint),
        "PUT" => client.put(endpoint),
        _ => {
            return Err(RelayerApiException::from_request_error(format!(
                "Unsupported method: {}",
                method
            )));
        }
    };

    if let Some(h) = headers {
        req = req.headers(h);
    }

    if let Some(d) = data {
        match d {
            RequestData::Value(v) => req = req.json(v),
            RequestData::TransactionRequest(tr) => {
                req = req.body(reqwest::blocking::Body::from(
                    serde_json::to_string(tr)
                        .unwrap()
                        .replace(":\"", ": \"")
                        .replace(":{", ": {")
                        .replace(",\"", ", \""),
                ));
            }
        }
    }

    let resp = req
        .send()
        .map_err(|e| RelayerApiException::from_request_error(format!("Request failed: {}", e)))?;

    let status = resp.status();
    if !status.is_success() {
        let error_msg = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
        return Err(RelayerApiException::new(Some(status.as_u16()), error_msg));
    }

    resp.json::<Value>().map_err(|e| {
        RelayerApiException::from_request_error(format!("Failed to parse JSON: {}", e))
    })
}

pub fn post(
    endpoint: &str,
    headers: Option<reqwest::header::HeaderMap>,
    data: Option<&RequestData>,
) -> Result<Value, RelayerApiException> {
    request(endpoint, "POST", headers, data)
}

pub fn get(
    endpoint: &str,
    headers: Option<reqwest::header::HeaderMap>,
    data: Option<&RequestData>,
) -> Result<Value, RelayerApiException> {
    request(endpoint, "GET", headers, data)
}
