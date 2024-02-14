use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub quotes_channel: Option<u64>,
    pub log_extensions: Option<Vec<String>>,
}
