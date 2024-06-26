use crate::log_upload;

use self::{
    checks::{check_checks, Severity},
    environment::get_environment_info,
};
use serenity::all::CreateEmbed;
use tokio::time::Instant;

pub mod checks;
pub mod environment;

pub fn check_logs(log: &str, name: &str, t: &log_upload::LogType, map_status: &log_upload::MapStatus) -> CreateEmbed {
    let start = Instant::now();
    let ctx = get_environment_info(log, map_status);
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
        embed = embed.field(
            format!("{} {}", ele.severity.get_emoji(), &ele.title),
            &ele.description,
            false,
        );
    }

    embed
}
