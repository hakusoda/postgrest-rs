use std::{
	pin::Pin,
	task::{ Poll, Context },
	future::Future,
	marker::PhantomData
};
use bytes::Bytes;
use serde::{
	de::DeserializeOwned,
	Deserialize, Deserializer
};
use reqwest::{
	header::{ HeaderMap, HeaderName },
	Method
};

use super::QueryBuilder;
use crate::{
	json,
	error::Error,
	result::PostgrestResult,
	Count, Result, PostgrestClient
};

pub struct FilterBuilder<'a, T: DeserializeOwned, F: DeserializeOwned> {
	url: Option<String>,
	#[allow(clippy::type_complexity)]
	fut: Option<Pin<Box<dyn Future<Output = Result<PostgrestResult<F>>> + Send + 'a>>>,
	body: Option<serde_json::Value>,
	count: Option<Count>,
	query: Option<Vec<(&'static str, String)>>,
	method: Option<Method>,
	client: Option<&'a PostgrestClient>,
	schema: Option<String>,
	headers: Option<HeaderMap>,
	phantom: PhantomData<(T, F)>,
	is_maybe_single: bool
}

impl<'a, T: DeserializeOwned, F: DeserializeOwned> FilterBuilder<'a, T, F> {
	pub fn new(query: QueryBuilder<'a>, method: Method, body: Option<serde_json::Value>) -> Self {
		Self {
			fut: None,
			url: Some(query.url),
			body,
			count: None,
			query: Some(query.query),
			method: Some(method),
			client: Some(query.client),
			schema: Some(query.schema),
			headers: Some(query.headers),
			phantom: Default::default(),
			is_maybe_single: false
		}
	}

	/// Match only rows where `column` is equal to `value`.
	///
	/// To check if the value of `column` is NULL, you should use `.is()` instead.
	pub fn eq(mut self, column: &'static str, value: impl ToString) -> Self {
		self.query.as_mut().unwrap().push((column, format!("eq.{}", value.to_string())));
		self
	}

	pub fn head(self) -> FilterBuilder<'a, (), ()> {
		FilterBuilder {
			url: self.url,
			fut: None,
			body: self.body,
			count: self.count,
			query: self.query,
			client: self.client,
			method: Some(Method::HEAD),
			schema: self.schema,
			headers: self.headers,
			phantom: PhantomData,
			is_maybe_single: false
		}
	}

	pub fn count(mut self, count: Count) -> Self {
		self = self.header("prefer", format!("count={}", count.to_string()));
		self.count.replace(count);
		self
	}

	pub fn header(mut self, key: impl ToString, value: impl ToString) -> Self {
		self.headers.as_mut().unwrap().insert(HeaderName::try_from(key.to_string()).unwrap(), value.to_string().parse().unwrap());
		self
	}

	/// Limit the query result by `count`.
	pub fn limit(mut self, count: usize) -> Self {
		self.query.as_mut().unwrap().push(("limit", count.to_string()));
		self
	}

	/// Return `data` as a single object instead of an array of objects.
	pub fn single(mut self) -> FilterBuilder<'a, T, T> {
		self.headers.as_mut().unwrap().insert("accept", "application/vnd.pgrst.object+json".parse().unwrap());
		FilterBuilder {
			url: self.url,
			fut: None,
			body: self.body,
			count: self.count,
			query: self.query,
			method: self.method,
			client: self.client,
			schema: self.schema,
			headers: self.headers,
			phantom: PhantomData,
			is_maybe_single: false
		}
	}
	
    /// Return `data` as a single object instead of an array of objects.
	///
    /// Query result must be zero or one row (e.g. using `.limit(1)`), otherwise this returns an error.
	pub fn maybe_single(mut self) -> FilterBuilder<'a, T, Option<T>> {
		self.headers.as_mut().unwrap().insert("accept", match self.method.as_ref().unwrap() {
			&Method::GET => "application/json",
			_ => "application/vnd.pgrst.object+json"
		}.parse().unwrap());
		FilterBuilder {
			url: self.url,
			fut: None,
			body: self.body,
			count: self.count,
			query: self.query,
			method: self.method,
			client: self.client,
			schema: self.schema,
			headers: self.headers,
			phantom: PhantomData,
			is_maybe_single: true
		}
	}
}

impl<'a, T: DeserializeOwned + Unpin, F: DeserializeOwned + Unpin> Future for FilterBuilder<'a, T, F> {
	type Output = Result<PostgrestResult<F>>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.fut.is_none() {
			let client = self.client.take().unwrap();
			let mut headers = client.default_headers.clone();
			for (key, value) in self.headers.take().unwrap().iter() {
				headers.insert(key, value.clone());
			}

			let method = self.method.take().unwrap();
			let is_head = matches!(method, Method::HEAD);
			let is_select = is_head || matches!(method, Method::GET);
			let mut builder = client.http
				.request(method, self.url.take().unwrap())
				.query(&self.query.take().unwrap())
				.headers(headers)
				.header(if is_select { "accept-profile" } else { "content-profile" }, self.schema.take().unwrap());
			if let Some(body) = self.body.take() {
				builder = builder
					.body(json::to_string(&body)?)
					.header("content-type", "application/json");
			}

			let response = builder.send();

			let count = self.count.take();
			let is_maybe_single = self.is_maybe_single;
			self.fut = Some(Box::pin(async move {
				let response = response.await?;
				let (mut item_count, total_item_count) = {
					if let Some(range) = response.headers().get("content-range") {
						let mut split = range
							.to_str()
							.unwrap()
							.split('/');
						let item_count = if let Some(mut range) = split.next().map(|x| x.split('-')) {
							let from = range.next().and_then(|x| x.parse::<isize>().ok()).unwrap_or(0);
							let to = range.next().and_then(|x| x.parse::<isize>().ok()).unwrap_or(-1);
							(to - from + 1) as usize
						} else { 0 };
						(item_count, count.map(|_| split.next().unwrap().parse::<usize>().unwrap()))
					} else { (0, None) }
				};

				let is_success = response.status().is_success();
				let mut bytes = if is_head || !is_select { Bytes::from("null") } else { response.bytes().await? };
				if bytes.is_empty() {
					bytes = Bytes::from("[]");
				}
				
				println!("{}", std::str::from_utf8(&bytes).unwrap());
				match is_success {
					true => Ok(PostgrestResult {
						value: match is_maybe_single {
							true => {
								let (item, is_some) = json::from_bytes::<MaybeSingleWrapper<F>>(&bytes)?.0;
								if is_some {
									item_count = 1;
								}
								
								item
							},
							false => json::from_bytes(&bytes)?
						},
						item_count,
						total_item_count
					}),
					false => Err(Error::PostgrestError(
						json::from_bytes(&bytes)?
					))
				}
			}));
		}

		self.fut.as_mut().unwrap().as_mut().poll(cx)
	}
}


// honestly kind of silly, but it's a classic katsumi workaround, so blehhhhhh!!!
#[derive(Deserialize)]
struct MaybeSingleWrapper<T: DeserializeOwned>(#[serde(deserialize_with = "deserialize_maybe_single_wrapper")] (T, bool));

fn deserialize_maybe_single_wrapper<'de, D: Deserializer<'de>, T: DeserializeOwned>(deserializer: D) -> core::result::Result<(T, bool), D::Error> {
	let value = serde_json::Value::deserialize(deserializer)?;

	let item = value.as_array().unwrap().first();
	let is_some = item.is_some();
	Ok((serde_json::from_value(match item {
		Some(x) => x.clone(),
		None => serde_json::Value::Null
	}).unwrap(), is_some))
}