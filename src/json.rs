use serde::de::DeserializeOwned;

#[cfg(feature = "simd-json")]
pub use simd_json::{ Error, Result, to_string, from_slice };

#[cfg(not(feature = "simd-json"))]
pub use serde_json::{ Error, Result, to_string, from_slice };

pub fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>  {
	#[cfg(feature = "simd-json")] {
		from_slice(&mut bytes.to_vec())
	}

	#[cfg(not(feature = "simd-json"))] {
		from_slice(bytes)
	}
}