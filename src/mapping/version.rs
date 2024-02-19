use lazy_static::lazy_static;
use regex::Regex;

struct McVersion {
    version: String,
}

async fn detect_mc_version(log: &str) -> Option<McVersion> {
    lazy_static! {
        static ref FABRIC_LOG_REGEX: Regex = Regex::new(r"Loading Minecraft (.*) with Fabric Loader").unwrap();
        static ref FABRIC_CRASH_REGEX: Regex = Regex::new(r"Minecraft Version: (.*)").unwrap();
    }
    let regexes: Vec<&Regex> = vec![&FABRIC_LOG_REGEX, &FABRIC_CRASH_REGEX];

    FABRIC_LOG_REGEX.find(log);

    None
}
