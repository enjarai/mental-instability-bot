use std::{collections::HashSet, fmt::Display, hash::Hash};

use regex::Regex;

use crate::log_upload::MapStatus;

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
#[derive(PartialEq)]
pub enum Launcher {
    Prism,
    PolyMC, // :concern:
    MultiMC,
    Vanilla,
    CurseForge,
    Theseus, // Modrinth launcher
}

impl Display for Launcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prism => write!(f, "<:prism:1246451647677468714>"),
            Self::PolyMC => write!(f, "<:polymc:1246451649212448860>"),
            Self::MultiMC => write!(f, "<:multimc:1246451644342865992>"),
            Self::Vanilla => write!(f, "<:minecraft:1246451645441642496>"),
            Self::CurseForge => write!(f, "<:curseforge:1246451646909911141>"),
            Self::Theseus => write!(f, "<:modrinth:1255163992658149376>"),
        }
    }
}

#[derive(Clone)]
pub struct ScanMod(pub &'static str, pub &'static str);

#[derive(Eq, Clone, Debug)]
pub struct DiscoveredMod<'a>(pub &'a str, pub &'a str);

impl PartialEq for DiscoveredMod<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for DiscoveredMod<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct KnownMod<'a>(pub ScanMod, pub DiscoveredMod<'a>);

pub struct EnvironmentContext<'a> {
    pub launcher: Option<Launcher>,
    pub mc_version: Option<String>,
    pub loader: Option<ModLoader>,
    pub discovered_mods: HashSet<DiscoveredMod<'a>>,
    pub known_mods: Vec<KnownMod<'a>>,
    pub map_status: &'a MapStatus,
}

impl Display for EnvironmentContext<'_> {
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
        if let MapStatus::Mapped(took) = self.map_status {
            write!(f, "**Remapped in:** `{}ms`\n", took.as_millis())?;
        }
        if !self.known_mods.is_empty() {
            write!(f, "\n")?;
            write!(f, "**Detected Mods:**\n")?;
            for ele in &self.known_mods {
                write!(f, "- {} `{}`\n", ele.0 .1, ele.1 .1)?;
            }
            write!(
                f,
                "- and {} more...\n",
                self.discovered_mods.len() - self.known_mods.len()
            )?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! truncate {
    ($string:expr,$max_len:expr) => {{
        let mut result = $string;
        let length = result.len();
        if length > $max_len {
            result.truncate($max_len - 3);
            result = format!("{result}...");
        }
        result
    }};
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
    ($log:expr,$max_len:expr,$($arg:expr),*) => {'a: {
        $(
            if let Some(cap) = Regex::new($arg).expect("Incorrect regex").captures($log) {
                break 'a Some(cap.get(1).map(|m| truncate!(m.as_str().to_string(), $max_len)));
            }
        )*
        None
    }};
}

#[macro_export]
macro_rules! peek {
    ($log:expr,$($arg:expr),*) => {'a: {
        $(
            if Regex::new($arg).expect("Incorrect regex").is_match($log) {
                break 'a true;
            }
        )*
        false
    }};
}

#[macro_export]
macro_rules! expect {
    ($captures:expr,$index:expr,$max_len:expr) => {{
        truncate!(
            $captures
                .get($index)
                .expect("Regex err")
                .as_str()
                .to_string(),
            $max_len
        )
    }};
}

//

pub fn read_mc_version(log: &str) -> Option<String> {
    grab!(
        log,
        32,
        r"Loading Minecraft ([^\s]+)",
        r"minecraft server version ([^\s]+)",
        r"Minecraft Version: ([^\s]+)"
    )
    .map(|o| o.expect("Regex error!!!"))
}

pub fn get_environment_info<'a>(log: &'a str, map_status: &'a MapStatus) -> EnvironmentContext<'a> {
    let launcher = if peek!(log, r"Prism Launcher version:") {
        Some(Launcher::Prism)
    } else if peek!(log, r"PolyMC version:") {
        Some(Launcher::PolyMC)
    } else if peek!(log, r"MultiMC version:") {
        Some(Launcher::MultiMC)
    } else if peek!(log, r"[\\/]com\.modrinth\.theseus[\\/]") {
        Some(Launcher::Theseus)
    } else {
        None
    };

    let mut loader = None;

    if let Some(fabric_version) = grab!(
        log,
        32,
        r"Loading Minecraft [^\s]+ with Fabric Loader ([^\s]+)",
        r"fabricloader: Fabric Loader ([^\s]+)",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'fabric'"
    ) {
        loader = Some(ModLoader::Fabric(fabric_version));
    } else if peek!(
        log,
        r"ne\.mi\.fm\.lo",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'forge'"
    ) {
        loader = Some(ModLoader::Forge);
    } else if peek!(
        log,
        r"net\.neoforged\.fml\.loading",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'neoforge'"
    ) {
        loader = Some(ModLoader::NeoForge);
    } else if let Some(quilt_version) = grab!(
        log,
        32,
        r"Loading Minecraft [^\s]+ with Quilt Loader ([^\s]+)",
        r"Is Modded: Definitely; [^\s]+ brand changed to 'quilt'"
    ) {
        loader = Some(ModLoader::Quilt(quilt_version));
    }

    let mc_version = read_mc_version(log);

    let discovered_mods = [
        Regex::new(r"\n\s*- (\S+) (\S+)"),
        Regex::new(r"\n\s*(\S+): .+ (\S+)"),
        Regex::new(r"mod '.+' \((\S+)\) (\S+)"),
        Regex::new(
            r"\| \s*\w+ \| [^|]* \| (\S+)\s* \| (\S+)\s* \| [^|]* \| \w+\s* \| [^|]* \| [^|]* \|",
        ),
    ]
    .into_iter()
    .flat_map(|r| {
        r.expect("brombeere")
            .captures_iter(log)
            .map(|cap| {
                let mod_id = cap.get(1).expect("Regex err").as_str();
                let mod_version = cap.get(2).expect("Regex err 2").as_str();
                DiscoveredMod(mod_id, mod_version)
            })
            .collect::<Vec<_>>()
    })
    .collect::<HashSet<_>>();

    let scan_mods = vec![
        ScanMod("fabric", "<:fabric:1246103308842700831> Fabric API"),
        ScanMod("fabric-api", "<:fabric:1246103308842700831> Fabric API"),
        ScanMod(
            "quilt_base",
            "<:quilt:1246170627652718653> Quilt Standard Libraries",
        ),
        ScanMod(
            "quilted_fabric_api",
            "<:quilt:1246170627652718653> Quilted Fabric API",
        ),
        ScanMod(
            "do_a_barrel_roll",
            "<:doabarrelroll:1107712867823792210> Do a Barrel Roll",
        ),
        ScanMod(
            "showmeyourskin",
            "<:showmeyourskin:1107713046987686009> Show Me Your Skin",
        ),
        ScanMod(
            "rolling_down_in_the_deep",
            "<:rollingdowninthedeep:1246194315580145734> Rolling Down in the Deep",
        ),
        ScanMod(
            "mini_tardis",
            "<:minitardis:1246194819739549707> Mini Tardis",
        ),
        ScanMod(
            "skinshuffle",
            "<:skinshuffle:1120649582502756392> SkinShuffle",
        ),
        ScanMod(
            "omnihopper",
            "<:omnihopper:1107713581446873158> Omni-Hopper",
        ),
        ScanMod(
            "recursiveresources",
            "<:recursiveresources:1107713344355442799> Recursive Resources",
        ),
        ScanMod(
            "shared-resources",
            "<:sharedresources:1107713221063872532> Shared Resources",
        ),
        ScanMod(
            "clientpaintings",
            "<:clientpaintings:1107713678712774778> Client Paintings",
        ),
        ScanMod(
            "moderate-loading-screen",
            "<:moderateloadingscreen:1107713920271122462> Mod-erate Loading Screen",
        ),
        ScanMod(
            "blahaj-totem",
            "<:shork:1172685466676502559> Blåhaj of Undying",
        ),
        ScanMod(
            "restart_detector",
            "<:restartdetector:1172685600000847922> Restart Detector",
        ),
        ScanMod("trickster", "<:trickster:1254515857640394876> Trickster"),
        ScanMod(
            "headpats",
            "<:headpats:1282096343006838825> Headpat a Friend!",
        ),
        ScanMod("cicada", "<:cicada:1246197518807863367> CICADA"),
        ScanMod(
            "elytratrims",
            "<:elytratrims:1246408624423702558> Elytra Trims",
        ),
        ScanMod(
            "soundboard",
            "<:soundboard:1246447385362698280> Voice Chat Soundboard",
        ),
        ScanMod("particular", "<:particular:1247196807688228876> Particular"),
        ScanMod(
            "inventory_sort",
            "<:chaiinventorysorter:1247198105745494086> Chai's Inventory Sorter",
        ),
        ScanMod("wowozela", "<:wowozela:1254516583032819732> Wowozela"),
        ScanMod(
            "flightassistant",
            "<:flightassistant:1248205945494966334> FlightAssistant",
        ),
        ScanMod("neruina", "<:neruina:1247198104017571900> Neruina"),
        ScanMod(
            "midnightlib",
            "<:midnightlib:1247198101018513569> MidnightLib",
        ),
        ScanMod("zauber", "<:zauber:1247198102230798516> Zauber"),
        ScanMod("owo", "<:owo:1246492160027656273> oωo"),
        ScanMod(
            "yet_another_config_lib_v3",
            "<:yacl:1248028103775752212> Yet Another Config Lib",
        ),
        // Shitass mods lmao
        ScanMod("optifabric", "<:optifabric:1246484303110606978> OptiFabric"),
        ScanMod("bclib", "<:bclib:1246585932379852901> BCLib"),
        ScanMod("feather", "<:feather:1247213091591028756> Feather Client"),
    ];

    let known_mods = discovered_mods
        .iter()
        .filter_map(|d| {
            scan_mods
                .iter()
                .find(|s| s.0 == d.0)
                .map(|s| KnownMod(s.clone(), d.clone()))
        })
        .collect();

    EnvironmentContext {
        launcher,
        mc_version,
        loader,
        discovered_mods,
        known_mods,
        map_status,
    }
}
