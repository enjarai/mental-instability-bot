#[macro_export]
macro_rules! get_config {
    ($arg:expr) => {{
        $arg.data
            .read()
            .await
            .get::<$crate::ConfigData>()
            .expect("No config?")
    }};
}

#[macro_export]
macro_rules! get_mappings_cache {
    ($arg:expr) => {{
        $arg.data
            .write()
            .await
            .get_mut::<$crate::MappingsCacheKey>()
            .expect("No mappings cache?")
    }};
}
