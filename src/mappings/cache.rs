use crate::constants::{MAPPINGS_CACHE_LIMIT, MAPPINGS_CACHE_PURGE_SIZE};

use super::{download::download_mappings, Mappings};
use anyhow::Result;
use std::collections::HashMap;

struct CacheEntry {
    mappings: Mappings,
    hits: u32,
}

pub struct MappingsCache {
    cache: HashMap<String, CacheEntry>,
}

impl MappingsCache {
    pub fn create() -> MappingsCache {
        MappingsCache {
            cache: HashMap::new(),
        }
    }

    pub async fn get_or_download<'a>(
        &'a mut self,
        mc_version: &str,
    ) -> Result<Option<&'a Mappings>> {
        if !self.cache.contains_key(mc_version) {
            if let Some(downloaded) = download_mappings(mc_version).await? {
                self.try_invalidate();
                self.cache.insert(
                    mc_version.to_string(),
                    CacheEntry {
                        mappings: downloaded,
                        hits: 0,
                    },
                );
            }
        }

        Ok(self.cache.get_mut(mc_version).map(|c| {
            c.hits += 1;
            &c.mappings
        }))
    }

    pub fn try_invalidate(&mut self) {
        if self.cache.len() > MAPPINGS_CACHE_LIMIT {
            let remove = {
                let mut entries = self.cache.iter().collect::<Vec<_>>();
                entries.sort_by_key(|(_, e)| e.hits);
                entries
                    .iter()
                    .take(MAPPINGS_CACHE_PURGE_SIZE)
                    .map(|(key, _)| key.to_string())
                    .collect::<Vec<_>>()
            };
            for ele in remove {
                self.cache.remove(&ele);
            }
        }
    }

    pub fn cached_keys<'a>(&'a self) -> Vec<&'a String> {
        self.cache.keys().collect()
    }

    pub fn get_hits(&self, key: &str) -> u32 {
        self.cache.get(key).map(|c| c.hits).unwrap_or(0)
    }
}
