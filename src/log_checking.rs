use std::{borrow::Cow, cmp, fs};

use serde::{Deserialize, Deserializer, de::Error};
use regex::RegexSet;
use serenity::client::Context;
use anyhow::Result;

#[derive(Deserialize, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Severity {
    None,
    Medium,
    High,
}

impl Severity {
    pub fn get_color(&self) -> u32 {
        match self {
            Severity::None => 0x219ebc,
            Severity::Medium => 0xf77f00,
            Severity::High => 0xd62828,
        }
    }
}

#[derive(Deserialize)]
pub struct LogCheck {
    #[serde(deserialize_with = "deserialize_regex")]
    regexes: RegexSet,
    severity: Severity,
    response: String,
}

pub struct CheckResult {
    pub severity: Severity,
    pub reports: Vec<String>,
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<RegexSet, D::Error> where D: Deserializer<'de> {
    let regexes = <Vec<Cow<str>>>::deserialize(deserializer)?;

    match RegexSet::new(regexes) {
        Ok(regexset) => Ok(regexset),
        Err(err) => Err(D::Error::custom(err)),
    }
}

pub fn load_checks() -> Vec<LogCheck> {
    let files = fs::read_dir("./log_checks/").expect("reading log checks directory");

    let mut result = vec![];

    for file in files {
        let file = file.expect("locating log check");
        let path = file.path();
        let file_name = file.file_name().into_string().expect("reading filename");

        if file_name.ends_with(".toml") {
            let check = toml::from_str::<LogCheck>(&fs::read_to_string(path).expect("reading log check"))
                .expect("parsing log check");

            result.push(check)
        }
    }

    result
}

pub async fn check_checks(ctx: &Context, log: &str) -> Result<CheckResult> {
    let data = ctx.data.read().await;
    let checks = data.get::<crate::LogChecksData>().expect("no log checks?");

    let mut result = CheckResult {
        severity: Severity::None,
        reports: vec![],
    };

    for check in checks {
        if check.regexes.is_match(log) {
            result.severity = cmp::max(result.severity, check.severity);
            result.reports.push(check.response.clone());
        }
    }

    Ok(result)
}