use super::environment::{EnvironmentContext, Launcher};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Severity {
    None,
    Medium,
    High,
}

impl Severity {
    pub fn get_color(&self) -> u32 {
        match self {
            Severity::None => 0x0021_9ebc,
            Severity::Medium => 0x00f7_7f00,
            Severity::High => 0x00d6_2828,
        }
    }
}

pub struct CheckReport {
    pub title: String,
    pub description: String,
    pub severity: Severity,
}

pub fn check_checks(log: &str, ctx: &EnvironmentContext) -> Vec<CheckReport> {
    vec![
        polymc,
    ]
    .iter()
    .filter_map(|check| check(log, ctx))
    .collect()
}

pub fn polymc(_log: &str, ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(Launcher::PolyMC) = &ctx.launcher {
        return Some(CheckReport {
            title: "PolyMC Detected".to_string(),
            description: "PolyMC is an outdated launcher maintained by a queerphobic team. Consider switching to [Prism Launcher](https://prismlauncher.org/), a fork with more features and better support.".to_string(),
            severity: Severity::Medium,
        });
    }
    None
}