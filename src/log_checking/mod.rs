use self::{
    checks::{check_checks, Severity},
    environment::get_environment_info,
};
use serenity::all::CreateEmbed;

pub mod checks;
pub mod environment;

pub fn check_logs(embed: CreateEmbed, log: &str) -> CreateEmbed {
    let ctx = get_environment_info(log);
    let checks = check_checks(log, &ctx);
    let severity = checks
        .iter()
        .map(|r| r.severity)
        .max()
        .unwrap_or(Severity::None);

    let mut embed = embed.color(severity.get_color()).description(format!(
        "{ctx}{}",
        if checks.is_empty() {
            ""
        } else {
            "\n**Potential Issues Detected:**\n"
        }
    ));

    for ele in &checks {
        embed = embed.field(format!("- {}", &ele.title), &ele.description, true);
    }

    embed
}
