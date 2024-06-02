use std::fmt::Display;

use regex::Regex;

pub enum ModLoader {
    Fabric(Option<String>),
    Forge,
    NeoForge,
    Quilt(Option<String>),
}

impl Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fabric(Some(version)) => {
                write!(f, "<:fabric:1246103308842700831> `{version}`")
            }
            Self::Fabric(None) => write!(f, "<:fabric:1246103308842700831>"),
            Self::Forge => write!(f, "<:forge:1246170624364380221>"),
            Self::NeoForge => write!(f, "<:neoforge:1246170626159415326>"),
            Self::Quilt(Some(version)) => {
                write!(f, "<:quilt:1246170627652718653> `{version}`")
            }
            Self::Quilt(None) => write!(f, "<:quilt:1246170627652718653>"),
        }
    }
}

#[allow(dead_code)]
pub enum Launcher {
    Prism,
    PolyMC, // :concern:
    MultiMC,
    Vanilla,
    CurseForge,
}

impl Display for Launcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prism => write!(f, "<:prism:1246451647677468714>"),
            Self::PolyMC => write!(f, "<:polymc:1246451649212448860>"),
            Self::MultiMC => write!(f, "<:multimc:1246451644342865992>"),
            Self::Vanilla => write!(f, "<:minecraft:1246451645441642496>"),
            Self::CurseForge => write!(f, "<:curseforge:1246451646909911141>"),
        }
    }
}

pub struct ScanMod(pub &'static str, pub &'static str);

pub struct DiscoveredMod(pub ScanMod, pub String);

pub struct EnvironmentContext {
    pub launcher: Option<Launcher>,
    pub mc_version: Option<String>,
    pub loader: Option<ModLoader>,
    pub known_mods: Vec<DiscoveredMod>,
}

impl Display for EnvironmentContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(launcher) = &self.launcher {
            write!(f, "**Launcher:** {}\n", launcher)?;
        }
        if let Some(version) = &self.mc_version {
            write!(f, "**Minecraft:** `{}`\n", version)?;
        }
        if let Some(loader) = &self.loader {
            write!(f, "**Loader:** {}\n", loader)?;
        }
        if !self.known_mods.is_empty() {
            write!(f, "\n")?;
            write!(f, "**Known Mods:**\n")?;
            for ele in &self.known_mods {
                write!(f, "- {} `{}`\n", ele.0.1, ele.1)?;
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! grab_all {
    ($log:expr,$($arg:expr),*) => {'a: {
        $(
            if let Some(cap) = Regex::new($arg).expect("Incorrect regex").captures($log) {
                break 'a Some(cap);
            }
        )*
        None
    }};
}

#[macro_export]
macro_rules! grab {
    ($log:expr,$($arg:expr),*) => {'a: {
        $(
            if let Some(cap) = Regex::new($arg).expect("Incorrect regex").captures($log) {
                break 'a Some(cap.get(1).map(|m| m.as_str().to_string()));
            }
        )*
        None
    }};
}

macro_rules! known_mods {
    ($log:expr,$($arg:expr),*) => {{
        let mut vec = vec![];
        $(
            if let Some(mat) = grab!(
                $log,
                &format!(r"\n\s*- {} (\S+)", $arg.0),
                &format!(r"\n\s*{}: .+ (\S+)", $arg.0),
                &format!(r"mod '.+' \({}\) (\S+)", $arg.0),
                &format!(r"\| \s*\w+ \| [^|]* \| {}\s* \| (\S+)\s* \| [^|]* \| \w+\s* \| [^|]* \| [^|]* \|", $arg.0)
            ) {
                vec.push(DiscoveredMod($arg, mat.expect("Regex issue what")));
            }
        )*
        vec
    }};
}

pub fn get_environment_info(log: &str) -> EnvironmentContext {
    let launcher = if let Some(_) = grab!(
        log,
        r"Prism Launcher version:"
    ) {
        Some(Launcher::Prism)
    } else if let Some(_) = grab!(
        log,
        r"PolyMC version:"
    ) {
        Some(Launcher::PolyMC)
    } else if let Some(_) = grab!(
        log,
        r"MultiMC version:"
    ) {
        Some(Launcher::MultiMC)
    } else {
        None
    };


    let mut loader = None;

    if let Some(fabric_version) = grab!(
        log,
        r"Loading Minecraft [^\s]+ with Fabric Loader ([^\s]+)",
        r"fabricloader: Fabric Loader ([^\s]+)",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'fabric'"
    ) {
        loader = Some(ModLoader::Fabric(fabric_version));
    } else if let Some(_) = grab!(
        log,
        r"ne\.mi\.fm\.lo",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'forge'"
    ) {
        loader = Some(ModLoader::Forge);
    } else if let Some(_) = grab!(
        log,
        r"net\.neoforged\.fml\.loading",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'neoforge'"
    ) {
        loader = Some(ModLoader::NeoForge);
    } else if let Some(quilt_version) = grab!(
        log,
        r"Loading Minecraft [^\s]+ with Quilt Loader ([^\s]+)",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'quilt'"
    ) {
        loader = Some(ModLoader::Quilt(quilt_version));
    }

    let mc_version = grab!(
        log,
        r"Loading Minecraft ([^\s]+)",
        r"minecraft server version ([^\s]+)",
        r"Minecraft Version: ([^\s]+)"
    )
    .map(|o| o.expect("Regex error!!!"));

    let known_mods = known_mods!(
        log,
        ScanMod(
            "fabric",
            "<:fabric:1246103308842700831> Fabric API"
        ),
        ScanMod(
            "fabric-api",
            "<:fabric:1246103308842700831> Fabric API"
        ),
        ScanMod(
            "quilt_base",
            "<:quilt:1246170627652718653> Quilt Standard Libraries"
        ),
        ScanMod(
            "quilted_fabric_api",
            "<:quilt:1246170627652718653> Quilted Fabric API"
        ),
        ScanMod(
            "do_a_barrel_roll",
            "<:doabarrelroll:1107712867823792210> Do a Barrel Roll"
        ),
        ScanMod(
            "showmeyourskin",
            "<:showmeyourskin:1107713046987686009> Show Me Your Skin"
        ),
        ScanMod(
            "rolling_down_in_the_deep",
            "<:rollingdowninthedeep:1246194315580145734> Rolling Down in the Deep"
        ),
        ScanMod(
            "mini_tardis",
            "<:minitardis:1246194819739549707> Mini Tardis"
        ),
        ScanMod(
            "skinshuffle",
            "<:skinshuffle:1120649582502756392> SkinShuffle"
        ),
        ScanMod(
            "omnihopper",
            "<:omnihopper:1107713581446873158> Omni-Hopper"
        ),
        ScanMod(
            "recursiveresources",
            "<:recursiveresources:1107713344355442799> Recursive Resources"
        ),
        ScanMod(
            "shared-resources",
            "<:sharedresources:1107713221063872532> Shared Resources"
        ),
        ScanMod(
            "clientpaintings",
            "<:clientpaintings:1107713678712774778> Client Paintings"
        ),
        ScanMod(
            "moderate-loading-screen",
            "<:moderateloadingscreen:1107713920271122462> Mod-erate Loading Screen"
        ),
        ScanMod(
            "blahaj-totem",
            "<:shork:1172685466676502559> Blåhaj of Undying"
        ),
        ScanMod(
            "restart_detector",
            "<:restartdetector:1172685600000847922> Restart Detector"
        ),
        ScanMod(
            "cicada",
            "<:cicada:1246197518807863367> CICADA"
        ),
        ScanMod(
            "elytratrims",
            "<:elytratrims:1246408624423702558> Elytra Trims"
        ),
        ScanMod(
            "soundboard",
            "<:soundboard:1246447385362698280> Voice Chat Soundboard"
        ),
        ScanMod(
            "owo",
            "<:owo:1246492160027656273> oωo (owo-lib)"
        ),
        // Shitass mods lmao
        ScanMod(
            "optifabric",
            "<:optifabric:1246484303110606978> OptiFabric"
        ),
        ScanMod(
            "bclib",
            "<:bclib:1246585932379852901> BCLib"
        )
    );

    EnvironmentContext {
        launcher,
        mc_version,
        loader,
        known_mods,
    }
}
