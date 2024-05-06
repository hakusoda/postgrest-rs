use url::{ Url, ParseError };
use reqwest::{
	header::{ HeaderMap, InvalidHeaderValue },
	Client
};

use crate::builder::QueryBuilder;

pub struct PostgrestClient {
	pub(crate) http: Client,
	schema: String,
	base_url: Url,
	pub(crate) default_headers: HeaderMap
}

impl PostgrestClient {
	pub fn new(base_url: &str) -> Result<Self, ParseError> {
		Ok(Self {
			http: Client::new(),
			schema: "public".into(),
			base_url: Url::parse(base_url)?,
			default_headers: Default::default()
		})
	}
	
	pub fn default_headers(mut self, headers: HeaderMap) -> Self {
		for (key, value) in headers.iter() {
			self.default_headers.insert(key, value.clone());
		}
		self
	}

	pub fn with_supabase_key(self, api_key: &str) -> Result<Self, InvalidHeaderValue> {
		let mut headers = HeaderMap::new();
		headers.insert("apikey", api_key.parse()?);
		headers.insert("authorization", format!("Bearer {api_key}").parse()?);

		Ok(self.default_headers(headers))
	}

	/// Perform a query on a table or a view.
	pub fn from(&self, relation: &str) -> QueryBuilder {
		QueryBuilder::new(self, format!("{}/{relation}", self.base_url), self.schema.clone())
	}
}

#[cfg(test)]
mod tests {
	#![allow(dead_code)]
	use serde::Deserialize;
	use crate::{ Count, PostgrestClient };

	fn client() -> PostgrestClient {
		PostgrestClient::new("https://hakumi.supabase.co/rest/v1")
			.unwrap()
			.with_supabase_key(env!("SUPABASE_API_KEY"))
			.unwrap()
	}

	#[derive(Debug, Deserialize)]
	struct MinimalUser {
		id: String,
		username: String
	}

	#[tokio::test]
	async fn complex_user_select() {
		let result = client()
			.from("users")
			.select::<MinimalUser>("id, username")
			.count(Count::Estimated)
			.limit(10)
			.await
			.unwrap();

		assert!(result.len() == 10);
		assert!(result.total_item_count.unwrap() >= 10);
	}

	#[tokio::test]
	async fn basic_user_select_with_limit() {
		let result = client()
			.from("users")
			.select::<MinimalUser>("id, username")
			.limit(10)
			.await
			.unwrap();

		assert!(result.len() <= 10);
		assert_eq!(result.total_item_count, None);
	}

	#[tokio::test]
	async fn single_user_select() {
		client()
			.from("users")
			.select::<MinimalUser>("id, username")
			.limit(1)
			.single()
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn maybe_single_user_select() {
		let client = client();
		let result = client
			.from("users")
			.select::<MinimalUser>("id, username")
			.limit(1)
			.maybe_single()
			.await
			.unwrap();
		assert!(result.is_some());
		assert_eq!(result.item_count, 1);

		let result = client
			.from("users")
			.select::<MinimalUser>("id, username")
			.eq("username", "superlongusernamethatnoonecouldeverobtainbynormalmeans")
			.limit(1)
			.maybe_single()
			.await
			.unwrap();
		assert!(result.is_none());
		assert_eq!(result.item_count, 0);
	}

	#[tokio::test]
	async fn user_head() {
		let client = client();
		let result = client
			.from("users")
			.select::<()>("*")
			.head()
			.limit(1)
			.await
			.unwrap();
		assert_eq!(result.item_count, 1);

		let result = client
			.from("users")
			.select::<()>("*")
			.head()
			.eq("username", "katsumi")
			.limit(1)
			.await
			.unwrap();
		assert_eq!(result.item_count, 1);

		let result = client
			.from("users")
			.select::<()>("*")
			.head()
			.eq("username", "superlongusernamethatnoonecouldeverobtainbynormalmeans")
			.limit(1)
			.await
			.unwrap();
		assert_eq!(result.item_count, 0);
	}
}