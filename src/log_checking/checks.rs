use crate::{grab, grab_all};

use super::environment::{EnvironmentContext, Launcher, ModLoader};
use regex::Regex;
use std::{collections::HashSet, fmt::Write};

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

    pub fn get_emoji(&self) -> &'static str {
        match self {
            Severity::None => "<:severity_none:1246879605399228449>",
            Severity::Medium => "<:severity_medium:1246879606993190972>",
            Severity::High => "<:severity_high:1246879607869935678>",
        }
    }
}

pub struct CheckReport {
    pub title: String,
    pub description: String,
    pub severity: Severity,
}

pub fn check_checks(log: &str, ctx: &EnvironmentContext) -> Vec<CheckReport> {
    [
        crash_report_analysis,
        dependency_generic,
        crash_generic,
        mixin_conflicts,
        class_missing_generic,
        java,
        missing_field,
        quilt,
        polymc,
        optifabric,
        bclib,
        feather,
        indium,
    ]
    .iter()
    .filter_map(|check| check(log, ctx))
    .collect()
}

pub fn crash_report_analysis(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(captures) = grab_all!(
        log,
        r"---- Minecraft Crash Report ----\n// .+\n\nTime: .+\nDescription: (.+)\n\n(.+)\n"
    ) {
        let description = captures.get(1).expect("Regex err").as_str();
        let error = captures.get(2).expect("Regex err 2").as_str();
        return Some(CheckReport {
            title: "Crash report analysis".to_string(),
            description: format!("Context: `{description}`\n```\n{error}\n```"),
            severity: Severity::High,
        });
    }
    None
}

pub fn dependency_generic(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(captures) = grab_all!(
        log,
        r"Mod '(.+)' \(\S+\) \S+ requires any version between \S+ and \S+ of (.+), which is missing!",
        r"Mod '(.+)' \(\S+\) \S+ requires version \S+ or later of (.+), which is missing!",
        r"Mod '(.+)' \(\S+\) \S+ requires any version of (.+), which is missing!"
    ) {
        let dependent = captures.get(1).expect("Regex err").as_str();
        let dependency = captures.get(2).expect("Regex err 2").as_str();
        return Some(CheckReport {
            title: "Missing dependency".to_string(),
            description: format!(
                "The `{dependent}` mod needs `{dependency}` to be installed, but it is missing."
            ),
            severity: Severity::High,
        });
    }
    None
}

pub fn crash_generic(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(Some(mod_id)) = grab!(
        log,
        r"RuntimeException: Error creating Mixin config \S+\.json for mod (\S+)"
    ) {
        return Some(CheckReport {
            title: "Invalid mixin config".to_string(),
            description: format!("The mod `{mod_id}` is providing an invalid mixin config and cannot load in its current state, consider removing or updating it."),
            severity: Severity::High,
        });
    }

    if let Some(captures) = grab_all!(
        log,
        r"InvalidInjectionException: Critical injection failure: @Inject annotation on \S+ could not find any targets matching '.+' in \S+\. Using refmap \S+ \[PREINJECT Applicator Phase \-> \S+:(\w+) from mod (\w+)",
        r"InvalidAccessorException: No candidates were found matching \S+ in \S+ for \S+:(\w+) from mod (\w+)"
    ) {
        let mixin = captures.get(1).expect("Regex err").as_str();
        let mod_id = captures.get(2).expect("Regex err 2").as_str();
        return Some(CheckReport {
            title: "Mixin inject failed".to_string(),
            description: format!("Mixin `{mixin}` from mod `{mod_id}` has failed to apply. It is possible that `{mod_id}` is not compatible with this Minecraft version, consider double-checking its version."),
            severity: Severity::High,
        });
    }

    if let Some(captures) = grab_all!(
        log,
        r"InvalidInjectionException: \S+ on \S+ with priority \w+ cannot inject into \S+ merged by (\S+)\.\w+\.\w+ with priority \w+ \[PREINJECT Applicator Phase \-> \S+\.json:\S+ from mod (\S+) \->"
    ) {
        let mod1 = captures.get(1).expect("Regex err").as_str();
        let mod2 = captures.get(2).expect("Regex err 2").as_str();
        return Some(CheckReport {
            title: "Mixin conflict".to_string(),
            description: format!("A mixin from the mod `{mod2}` collided with one from `{mod1}`, these mods may be incompatible."),
            severity: Severity::High,
        });
    }

    if let Some(Some(mod_id)) = grab!(
        log,
        r"MixinApplyError: Mixin \[\S+\.mixins\.json:\S+ from mod (\S+)\] from phase \[\S+\] in config \[\S+\.mixins\.json\] FAILED during \S+",
        r"InvalidInjectionException: .+ from mod ([\w\(\)-]+)\s?\->.+"
    ) {
        return Some(CheckReport {
            title: "Mixin error".to_string(),
            description: format!("The mod `{mod_id}` has encountered a mixin error, this may be caused by a mismatch in Minecraft version or a mod incompatibility. Further investigation is required."),
            severity: Severity::High,
        });
    }

    if let Some(Some(mod_id)) = grab!(
        log,
        r"RuntimeException: Could not execute entrypoint stage '\S+' due to errors, provided by '(\S+)'!"
    ) {
        return Some(CheckReport {
            title: "Entrypoint error".to_string(),
            description: format!("The mod `{mod_id}` has encountered an error in it's entrypoint, though it may not have caused it. Further investigation is required."),
            severity: Severity::High,
        });
    }
    None
}

pub fn mixin_conflicts(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    let regex_redirect = Regex::new(r"@Redirect conflict\. Skipping (?:#redirector:)?\S+\.json:\S+ from mod (\S+)\->@Redirect::\S+ with priority \w+, already redirected by \S+\.json:\S+ from mod (\S+)->@Redirect::\S+ with priority \w+").expect("Regex err");
    let conflicts = regex_redirect
        .captures_iter(log)
        .map(|c| {
            (
                c.get(1).expect("Regex err").as_str(),
                c.get(2).expect("Regex err 2").as_str(),
            )
        })
        .collect::<HashSet<(&str, &str)>>();
    if !conflicts.is_empty() {
        let mut description = "Mixins from the mods below are conflicting, this may cause unintentional behaviour or broken features.\n".to_string();
        for ele in conflicts {
            let _ = write!(description, "- `{}` <-> `{}`\n", ele.0, ele.1);
        }

        return Some(CheckReport {
            title: "Mixin conflicts".to_string(),
            description,
            severity: Severity::Medium,
        });
    }
    None
}

pub fn class_missing_generic(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    let packages = Regex::new(r"java\.lang\.ClassNotFoundException: (\S+)\.\w+")
        .expect("Regex err")
        .captures_iter(log)
        .map(|cap| cap.get(1).expect("Reger err").as_str())
        .collect::<HashSet<&str>>();
    if !packages.is_empty() {
        let mut description = "Classes from the packages below failed to load, this may be an indicator of missing dependencies or outdated mods.\n".to_string();
        for ele in packages {
            let _ = write!(description, "- `{}`\n", ele);
        }

        return Some(CheckReport {
            title: "Missing classes".to_string(),
            description,
            severity: Severity::Medium,
        });
    }
    None
}

pub fn match_java_classfile_version(classfile_version: &str) -> Option<&'static str> {
    match classfile_version {
        "49.0" => Some("5"),
        "50.0" => Some("6"),
        "51.0" => Some("7"),
        "52.0" => Some("8"),
        "53.0" => Some("9"),
        "54.0" => Some("10"),
        "55.0" => Some("11"),
        "56.0" => Some("12"),
        "57.0" => Some("13"),
        "58.0" => Some("14"),
        "59.0" => Some("15"),
        "60.0" => Some("16"),
        "61.0" => Some("17"),
        "62.0" => Some("18"),
        "63.0" => Some("19"),
        "64.0" => Some("20"),
        "65.0" => Some("21"),
        _ => None,
    }
}

pub fn java(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(captures) = grab_all!(
        log,
        r"- Replace '.+' \(java\) ([0-9]+) with version ([0-9]+) or later\."
    ) {
        let has = captures.get(1).expect("Regex err").as_str();
        let need = captures.get(2).expect("Regex err 2").as_str();
        return Some(CheckReport {
            title: "Incorrect Java version".to_string(),
            description: format!(
                "A mod or Minecraft itself requires Java {need} to be used, but an older version, Java {has} is being used instead. You may have to [download](https://adoptium.net/temurin/releases/?version={need}) a newer Java version and/or select it in your launcher."
            ),
            severity: Severity::High,
        });
    }
    if let Some(captures) = grab_all!(
        log,
        r"UnsupportedClassVersionError: \S+ has been compiled by a more recent version of the Java Runtime \(class file version (\S+)\), this version of the Java Runtime only recognizes class file versions up to (\S+)"
    ) {
        let has = match_java_classfile_version(captures.get(2).expect("Regex err").as_str());
        let need = match_java_classfile_version(captures.get(1).expect("Regex err 2").as_str());
        return Some(CheckReport {
            title: "Incorrect Java version".to_string(),
            description: if let Some(has) = has
                && let Some(need) = need
            {
                format!(
                    "A mod or Minecraft itself requires Java {need} to be used, but an older version, Java {has} is being used instead. You may have to [download](https://adoptium.net/temurin/releases/?version={need}) a newer Java version and/or select it in your launcher."
                )
            } else {
                "A mod or Minecraft itself requires a different version of Java from the one that is available. You may have to [download](https://adoptium.net/temurin/releases/) a newer Java version and/or select it in your launcher.".to_string()
            },
            severity: Severity::High,
        });
    }
    None
}

// java.lang.NoSuchFieldError

pub fn missing_field(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    if grab!(log, r"java\.lang\.NoSuchFieldError").is_some() {
        return Some(CheckReport {
            title: "Field missing error".to_string(),
            description: "On the logical server some fields may be deleted by Fabric Loader when a mod defines them as client-only. Since this feature was broken before loader `0.15`, some mods may have implemented it incorrectly. See if there's an update for the mod in question, or try downgrading Fabric Loader.".to_string(),
            severity: Severity::High,
        });
    }
    None
}

pub fn quilt(_log: &str, ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(ModLoader::Quilt(_)) = &ctx.loader {
        return Some(CheckReport {
            title: "Quilt detected".to_string(),
            description: "Many mod developers may not officially support Quilt. Consider switching to Fabric if you aren't using any Quilt-specific mods.".to_string(),
            severity: Severity::None,
        });
    }
    None
}

pub fn polymc(_log: &str, ctx: &EnvironmentContext) -> Option<CheckReport> {
    if let Some(Launcher::PolyMC) = &ctx.launcher {
        return Some(CheckReport {
            title: "PolyMC detected".to_string(),
            description: "PolyMC is an outdated launcher maintained by a queerphobic team. Consider switching to [Prism Launcher](https://prismlauncher.org/), a fork with more features and better support.".to_string(),
            severity: Severity::Medium,
        });
    }
    None
}

pub fn optifabric(log: &str, ctx: &EnvironmentContext) -> Option<CheckReport> {
    if ctx
        .known_mods
        .iter()
        .find(|m| m.0 .0 == "optifabric")
        .is_some()
        || grab!(
            log,
            r"Mod '.+' \(\S+\) \S+ is incompatible with any version of mod '.+' \(optifabric\)",
            r"me\.modmuss50\.optifabric"
        )
        .is_some()
    {
        return Some(CheckReport {
            title: "OptiFabric detected".to_string(),
            description: "Optifine is known to cause problems with many mods on Fabric. If you're having strange issues or crashes, consider replacing it with some of the many available [alternatives](https://lambdaurora.dev/optifine_alternatives/).".to_string(),
            severity: Severity::High,
        });
    }
    None
}

pub fn bclib(_log: &str, ctx: &EnvironmentContext) -> Option<CheckReport> {
    if ctx.known_mods.iter().find(|m| m.0 .0 == "bclib").is_some() {
        return Some(CheckReport {
            title: "BCLib detected".to_string(),
            description: "BCLib is known to cause issues with some mods. If you're experiencing crashes or other problems, consider trying without it.".to_string(),
            severity: Severity::Medium,
        });
    }
    None
}

pub fn feather(_log: &str, ctx: &EnvironmentContext) -> Option<CheckReport> {
    if ctx.known_mods.iter().find(|m| m.0 .0 == "feather").is_some() {
        return Some(CheckReport {
            title: "Feather Client detected".to_string(),
            description: "Feather Client is known to cause issues with some mods. If you're experiencing crashes or other problems, consider trying without it.".to_string(),
            severity: Severity::Medium,
        });
    }
    None
}

pub fn indium(log: &str, _ctx: &EnvironmentContext) -> Option<CheckReport> {
    if grab!(
            log,
            r#"because the return value of "net\.fabricmc\.fabric\.api\.renderer\.v1\.RendererAccess\.getRenderer\(\)" is null"#
        )
        .is_some()
    {
        return Some(CheckReport {
            title: "Missing Indium".to_string(),
            description: "A mod is trying to make use of Fabric Rendering API, which may be missing when rendering mods such as Sodium are loaded. If you use Sodium, install [Indium](https://modrinth.com/mod/indium) to resolve this.".to_string(),
            severity: Severity::High,
        });
    }
    None
}
