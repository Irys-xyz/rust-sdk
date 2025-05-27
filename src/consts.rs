pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_BUNDLER_URL: &str = "https://uploader.irys.xyz/";
pub const CHUNK_SIZE: u64 = 256u64 * 1024;
/// Multiplier applied to the buffer argument from the cli to determine the maximum number
/// of simultaneous request to the `chunk/ endpoint`.
pub const CHUNKS_BUFFER_FACTOR: usize = 20;

/// Number of times to retry posting chunks if not successful.
pub const CHUNKS_RETRIES: u16 = 10;

/// Number of seconds to wait between retying to post a failed chunk.
pub const CHUNKS_RETRY_SLEEP: u64 = 1;

/// Number of seconds to wait between retying to post a failed chunk.
pub const RETRY_SLEEP: u64 = 10;

/// Number of confirmations needed to consider a transaction funded
pub const CONFIRMATIONS_NEEDED: u64 = 5;

pub const USE_JS_SDK: &str = "Currently unsupported, please use the js-sdk (https://github.com/Irys-xyz/js-sdk) to perform this operation (PRs welcome!)";

pub const LIST_AS_BUFFER: &[u8] = "list".as_bytes();
pub const BLOB_AS_BUFFER: &[u8] = "blob".as_bytes();
pub const DATAITEM_AS_BUFFER: &[u8] = "dataitem".as_bytes();
pub const ONE_AS_BUFFER: &[u8] = "1".as_bytes();
