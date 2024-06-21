use number_prefix::NumberPrefix;

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