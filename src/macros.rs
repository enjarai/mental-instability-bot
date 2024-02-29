#[macro_export]
macro_rules! get_config {
    ($arg:expr) => {{
        $arg.data
            .read()
            .await
            .get::<crate::ConfigData>()
            .expect("No config?")
    }};
}
