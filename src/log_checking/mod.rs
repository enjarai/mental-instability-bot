use crate::log_upload;

use self::{
    checks::{check_checks, Severity},
    environment::get_environment_info,
};
use serenity::all::CreateEmbed;
use tokio::time::Instant;

pub mod checks;
pub mod environment;

pub fn check_logs(log: &str, name: &str, t: &log_upload::LogType) -> CreateEmbed {
    let start = Instant::now();
    let ctx = get_environment_info(log);
    let checks = check_checks(log, &ctx);
    let severity = checks
        .iter()
        .map(|r| r.severity)
        .max()
        .unwrap_or(Severity::None);
    let took = Instant::now() - start;

    let mut embed = CreateEmbed::new()
        .title(t.title_format(name, &took))
        .color(severity.get_color())
        .description(format!(
            "{ctx}{}",
            if checks.is_empty() {
                ""
            } else if matches!(severity, Severity::None) {
                "\n**More Information:**\n"
            } else {
                "\n**Potential Issues Detected:**\n"
            }
        ));

    for ele in &checks {
        embed = embed.field(format!("- {}", &ele.title), &ele.description, false);
    }

    embed
}
