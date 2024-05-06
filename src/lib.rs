pub mod error;
pub mod client;
pub mod result;
pub mod builder;

mod json;

pub use error::Error;
pub use client::PostgrestClient;
pub use result::PostgrestResult;
pub(crate) use error::Result;

/// Use this with [FilterBuilder::count](builder::FilterBuilder::count) to receive an item count with a query result.
pub enum Count {
	/// Calculates the exact count. Note that the larger the table, the slower this query runs in the database.
	/// [Learn more](https://postgrest.org/en/v12/references/api/pagination_count.html#exact-count)
	Exact,

	/// Calculates an approximated count. This is fast, but the accuracy depends on how up-to-date the PostgreSQL statistics tables are.
	/// [Learn more](https://postgrest.org/en/v12/references/api/pagination_count.html#planned-count)
	Planned,

	/// Automatically uses [Exact](Count::Exact) for small tables, and [Planned](Count::Planned) for large tables.
	/// [Learn more](https://postgrest.org/en/v12/references/api/pagination_count.html#estimated-count)
	Estimated,
	
	/// Specify a custom counter, generally you shouldn't use this unless you've modified PostgREST or this library is out-of-date.
	Custom(String)
}

impl ToString for Count {
	fn to_string(&self) -> String {
		match self {
			Count::Exact => "exact",
			Count::Planned => "planned",
			Count::Estimated => "estimated",
			Count::Custom(x) => x
		}.to_string()
	}
}