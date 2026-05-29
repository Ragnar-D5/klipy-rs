use std::result::Result as StdResult;

/// Alias for results returned by this crate.
pub type Result<T> = StdResult<T, Error>;

/// Errors for `klipy` crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The underlying HTTP transport failed.
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// The API responded with a non-success HTTP status.
    #[error("klipy returned status {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },

    /// The request succeeded but the API reported `result: false`.
    #[error("klipy reported failure")]
    Unsuccessful,
}
