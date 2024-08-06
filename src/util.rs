use number_prefix::NumberPrefix;
use reqwest::Client;

pub fn format_bytes(bytes: u32) -> String {
    match NumberPrefix::decimal(bytes as f32) {
        NumberPrefix::Standalone(amount) => {
            format!("{amount} bytes")
        }
        NumberPrefix::Prefixed(prefix, amount) => {
            format!("{amount:.1} {prefix}B")
        }
    }
}

pub fn create_http() -> reqwest::Result<reqwest::Client> {
    Client::builder()
        .user_agent("enjarai/mental-instability-bot (enjarai@protonmail.com)")
        .build()
}