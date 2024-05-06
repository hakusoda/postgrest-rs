use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Json Error: {0}")]
	JsonError(#[from] crate::json::Error),

	#[error("Reqwest Error: {0}")]
	ReqwestError(#[from] reqwest::Error),

	#[error("PostgREST Error: {0}")]
	PostgrestError(PostgrestError),

	#[error("Invalid Header Value Error: {0}")]
	InvalidHeaderValueError(#[from] reqwest::header::InvalidHeaderValue),

	#[error("Unknown")]
	Unknown
}

#[derive(Debug, Deserialize)]
pub struct PostgrestError {
	pub code: String,
	pub details: String,
	pub message: String
}

impl std::fmt::Display for PostgrestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}

pub type Result<T> = core::result::Result<T, Error>;