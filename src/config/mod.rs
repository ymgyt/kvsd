mod initialize;
pub(crate) use initialize::Initializer;

mod config;
pub(crate) use config::Config;

pub(crate) mod filepath {
    pub const NAMESPACES: &str = "namespaces";
    pub const NS_SYSTEM: &str = "system";
    pub const NS_DEFAULT: &str = "default";
}
