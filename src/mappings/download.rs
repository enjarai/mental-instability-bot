use anyhow::Result;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor, Read},
};
use zip::ZipArchive;

use crate::{commands::version::get_yarn_version, util::create_http};

use super::Mappings;

pub async fn download_mappings(mc_version: &str) -> Result<Option<Mappings>> {
    if let Some(yarn_version) = get_yarn_version(mc_version).await?.version {
        let yarn_jar = &create_http()?
            .get(format!(
                "{}/net/fabricmc/yarn/{}/yarn-{}-v2.jar",
                crate::constants::FABRIC_MAVEN_URL,
                yarn_version,
                yarn_version
            ))
            .send()
            .await?
            .bytes()
            .await?;

        let mut zip = ZipArchive::new(Cursor::new(yarn_jar))?;

        let tiny_file = zip.by_name("mappings/mappings.tiny")?;

        Ok(Some(parse_mappings(tiny_file)?))
    } else {
        Ok(None)
    }
}

pub fn parse_mappings<T: Read>(file: T) -> Result<Mappings> {
    let mut full_classes: HashMap<String, String> = HashMap::new();
    let mut partial_classes: HashMap<String, String> = HashMap::new();
    let mut methods: HashMap<String, String> = HashMap::new();
    let mut fields: HashMap<String, String> = HashMap::new();

    for ele in BufReader::new(file).lines() {
        if let Ok(ele) = ele {
            let mut line = ele.trim().split("\t");

            if let Some(line_type) = line.next() {
                if line_type == "c" {
                    if let Some(class) = line.next()
                        && let Some(obf_name) = class.split("/").nth(2)
                        && let Some(mapped_class) = line.next()
                        && let Some(mapped_name) = mapped_class.split("/").last()
                    {
                        let mapped_class = mapped_class.replace("/", ".");

                        full_classes.insert(obf_name.to_string(), mapped_class);
                        partial_classes.insert(obf_name.to_string(), mapped_name.to_string());
                    }
                } else if line_type == "m" {
                    line.next();
                    if let Some(obf_name) = line.next()
                        && let Some(mapped_name) = line.next()
                    {
                        methods.insert(obf_name.to_string(), mapped_name.to_string());
                    }
                } else if line_type == "f" {
                    line.next();
                    if let Some(obf_name) = line.next()
                        && let Some(mapped_name) = line.next()
                    {
                        fields.insert(obf_name.to_string(), mapped_name.to_string());
                    }
                }
            }
        }
    }

    Ok(Mappings {
        full_classes,
        partial_classes,
        methods,
        fields,
    })
}
