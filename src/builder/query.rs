use serde::{ de::DeserializeOwned, Serialize };
use reqwest::{ header::HeaderMap, Method };

use crate::{ json, PostgrestClient };
use super::FilterBuilder;

pub struct QueryBuilder<'a> {
	pub(super) url: String,
	pub(super) query: Vec<(&'static str, String)>,
	pub(super) client: &'a PostgrestClient,
	pub(super) schema: String,
	pub(super) headers: HeaderMap
}

impl<'a> QueryBuilder<'a> {
	pub fn new(client: &'a PostgrestClient, url: String, schema: String) -> Self {
		Self {
			url,
			query: vec![],
			client,
			schema,
			headers: Default::default()
		}
	}

	/// Perform a SELECT query on the table or view.
	pub fn select<T: DeserializeOwned + Unpin>(mut self, columns: impl Into<String>) -> FilterBuilder<'a, T, Vec<T>> {
		let columns: String = columns.into();

		// https://github.com/supabase/postgrest-js/blob/03a811da4e16ff76baa2eb96891770c5b8ee5ac9/src/PostgrestQueryBuilder.ts#L72
		let mut quoted = false;
		let cleaned_columns: String = columns
			.chars()
			.filter(|x| {
				if x.is_whitespace() && !quoted {
					return false;
				} else if *x == '"' {
					quoted = !quoted;
				}
				true
			})
			.collect();
		self.query.push(("select", cleaned_columns));

		FilterBuilder::new(self, Method::GET, None)
	}

	/// Perform an INSERT into the table or view.
	pub fn insert<T: Serialize>(mut self, values: T) -> json::Result<FilterBuilder<'a, (), ()>> {
		let value = json::to_value(values)?;

		// https://github.com/supabase/postgrest-js/blob/03a811da4e16ff76baa2eb96891770c5b8ee5ac9/src/PostgrestQueryBuilder.ts#L164
		if let Some(array) = value.as_array() {
			// this is untested
			let mut columns = array.iter().fold(vec![], |mut acc, value| {
				if let Some(map) = value.as_object() {
					acc.extend(map.keys());
				}
				acc
			});
			if !columns.is_empty() {
				columns.sort();
				columns.dedup();
				self.query.push(("columns", columns.into_iter().map(|x| format!("\"{x}\"")).collect::<Vec<String>>().join(",")))
			}
		}

		Ok(FilterBuilder::new(self, Method::POST, Some(value)))
	}

	/// Perform an UPDATE on the table or view.
	pub fn update<T: Serialize>(mut self, values: T) -> json::Result<FilterBuilder<'a, (), ()>> {
		let value = json::to_value(values)?;

		// https://github.com/supabase/postgrest-js/blob/03a811da4e16ff76baa2eb96891770c5b8ee5ac9/src/PostgrestQueryBuilder.ts#L164
		if let Some(array) = value.as_array() {
			// this is untested
			let mut columns = array.iter().fold(vec![], |mut acc, value| {
				if let Some(map) = value.as_object() {
					acc.extend(map.keys());
				}
				acc
			});
			if !columns.is_empty() {
				columns.sort();
				columns.dedup();
				self.query.push(("columns", columns.into_iter().map(|x| format!("\"{x}\"")).collect::<Vec<String>>().join(",")))
			}
		}

		Ok(FilterBuilder::new(self, Method::PATCH, Some(value)))
	}
}