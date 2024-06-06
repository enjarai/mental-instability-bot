pub mod cache;
pub mod download;

use std::collections::HashMap;

use regex::{Captures, Regex};

pub struct Mappings {
    // `class_23232` -> `net.minecraft.something.Something`
    pub full_classes: HashMap<String, String>,
    // `class_23232` -> `Something`
    pub partial_classes: HashMap<String, String>,
    // `method_23232` -> `doSomething`
    pub methods: HashMap<String, String>,
    // `field_23232` -> `somethingData`
    pub fields: HashMap<String, String>,
}

macro_rules! replace_part {
    ($hay:expr,$map:expr,$regex:expr,$mapper:expr) => {{
        let regex = Regex::new($regex).expect("regex");
        regex.replace_all($hay, |caps: &Captures| {
            $map.get(caps.get(1).expect("regex err").as_str())
                .map($mapper)
                .unwrap_or_else(|| caps.get(0).expect("regex err").as_str().to_string())
        })
    }};
}

impl Mappings {
    pub fn remap_log(&self, log: &str) -> String {
        let result = replace_part!(
            log,
            self.full_classes,
            r"net\.minecraft\.(class_[0-9]+)",
            |s| s.to_string()
        );

        let result = replace_part!(
            &result,
            self.full_classes,
            r"net\/minecraft\/(class_[0-9]+)",
            |s| s.replace(".", "/")
        );

        let result = replace_part!(
            &result,
            self.partial_classes,
            r"(class_[0-9]+)",
            |s| s.to_string()
        );

        let result = replace_part!(
            &result,
            self.methods,
            r"(method_[0-9]+)",
            |s| s.to_string()
        );

        let result = replace_part!(
            &result,
            self.methods,
            r"(field_[0-9]+)",
            |s| s.to_string()
        );

        result.into_owned()
    }
}
