use std::ops::{ Deref, DerefMut };

pub struct PostgrestResult<T> {
	pub value: T,

	/// A total count of the items returned in the query.
	pub item_count: usize,

	/// An estimated item count of the queried table, this will always be `None` unless you specify a count using [FilterBuilder::count](crate::builder::FilterBuilder::count).
	pub total_item_count: Option<usize>
}

impl<T> PostgrestResult<T> {
	pub fn is_empty(&self) -> bool {
		self.item_count == 0
	}

	/// Returns `true` if a count was not specified in the query.
	pub fn is_table_empty(&self) -> bool {
		self.total_item_count.unwrap_or(0) == 0
	}
}

impl<T> Deref for PostgrestResult<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for PostgrestResult<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}