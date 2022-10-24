pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const BUNDLR_DEFAULT_URL: &str = "https://node1.bundlr.network/";
pub const CHUNK_SIZE: u64 = 256u64 * 1024;
/// Multiplier applied to the buffer argument from the cli to determine the maximum number
/// of simultaneous request to the `chunk/ endpoint`.
pub const CHUNKS_BUFFER_FACTOR: usize = 20;

/// Number of times to retry posting chunks if not successful.
pub const CHUNKS_RETRIES: u16 = 10;

/// Number of seconds to wait between retying to post a failed chunk.
pub const CHUNKS_RETRY_SLEEP: u64 = 1;
