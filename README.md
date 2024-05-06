# postgrest-rs
another rust client library for [PostgREST](https://postgrest.org)!

> [!WARNING]
> This library is *not* production-ready or feature-complete, it currently provides a very limited query filter and only JSON results.

## Examples
Add the library to your project in `Cargo.toml`:
```toml
[dependencies]
postgrest = { git = "https://github.com/hakusoda/postgrest-rs.git" }
```

Basic usage:
```rs
use serde::Deserialize;
use postgrest::PostgrestClient;

#[derive(Debug, Deserialize)]
struct User {
	id: String,
	username: String
}

let client = PostgrestClient::new("https://your.postgrest.endpoint.lgbt")?;
let result = client
	.from("users")
	.select::<User>("id, username")
	.await?;
println!("{result:?}");
```

Usage for Supabase projects:
```rs
use postgrest::PostgrestClient;

let client = PostgrestClient::new("https://your.postgrest.endpoint.lgbt")?
	// in a real scenario, you should use an environment variable instead of hardcoding your API key.
	.with_supabase_key("YOUR_SUPABASE_KEY")?;
/// ...your other code here!
```

## Features
### `simd-json`
This feature enables [simd-json](https://crates.io/crates/simd-json) support to utilise SIMD features of modern CPUs to deserialise responses faster, it is disabled by default.
<br/><br/>
To use this feature you must first enable the library feature in your `Cargo.toml`:
```toml
[dependencies]
postgrest = { git = "https://github.com/hakusoda/postgrest-rs.git", features = ["simd-json"] }
```

Additionally, you'll need to add this to `<project root>/.cargo/config.toml`:
```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

## Contributing
feel free to do whatever! (within acceptable bounds)