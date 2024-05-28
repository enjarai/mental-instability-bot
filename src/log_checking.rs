use std::{borrow::Cow, cmp, fs, path::Path};

use anyhow::Result;
use regex::{Captures, Regex};
use serde::{Deserialize, Deserializer};
use serenity::client::Context;

#[derive(Deserialize, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Severity {
    None,
    Medium,
    High,
}

impl Severity {
    pub fn get_color(self) -> u32 {
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
    regexes: Vec<Regex>,
    severity: Severity,
    title: String,
    response: String,
}

impl LogCheck {
    pub fn create_report(&self, regex_used: &Regex, captures: &Captures) -> (String, String) {
        let mut result = self.response.clone();
        for group in regex_used.capture_names() {
            if let Some(group) = group
                && let Some(capture) = captures.name(group)
            {
                result = result.replace(&format!("{{{group}}}"), capture.as_str());
            }
        }
        (self.title.clone(), result)
    }
}

pub struct CheckResult {
    pub severity: Severity,
    pub reports: Vec<(String, String)>,
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<Vec<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    let regexes = <Vec<Cow<str>>>::deserialize(deserializer)?;

    Ok(regexes
        .iter()
        .map(|r| Regex::new(r).expect("Incorrect regex in log check"))
        .collect())
}

pub fn load_checks() -> Vec<LogCheck> {
    let files = fs::read_dir("./log_checks/").expect("reading log checks directory");

    let mut result = vec![];

    for file in files {
        let file = file.expect("locating log check");
        let path = file.path();
        let file_name = file.file_name().into_string().expect("reading filename");

        if Path::new(&file_name)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("toml"))
        {
            let check =
                toml::from_str::<LogCheck>(&fs::read_to_string(path).expect("reading log check"))
                    .expect("parsing log check");

            result.push(check);
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
        for regex in &check.regexes {
            if let Some(captures) = regex.captures(log) {
                result.severity = cmp::max(result.severity, check.severity);
                result.reports.push(check.create_report(regex, &captures));

                break;
            }
        }
    }

    Ok(result)
}
