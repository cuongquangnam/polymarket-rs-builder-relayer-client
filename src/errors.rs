use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelayerClientException {
    #[error("{0}")]
    Message(String),
}

impl RelayerClientException {
    pub fn new(msg: impl Into<String>) -> Self {
        RelayerClientException::Message(msg.into())
    }
}

#[derive(Error, Debug)]
pub enum RelayerApiException {
    #[error("API error: status_code={status_code:?}, error_message={error_msg}")]
    ApiError {
        status_code: Option<u16>,
        error_msg: String,
    },
    #[error("Request exception: {0}")]
    RequestException(String),
}

impl RelayerApiException {
    pub fn new(status_code: Option<u16>, error_msg: String) -> Self {
        RelayerApiException::ApiError {
            status_code,
            error_msg,
        }
    }

    pub fn from_request_error(msg: String) -> Self {
        RelayerApiException::RequestException(msg)
    }
}

