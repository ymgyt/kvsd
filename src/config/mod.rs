//! Config module manage controllable values

mod initialize;
pub use self::initialize::Initializer;

mod config;
pub use self::config::Config;

pub(crate) mod filepath {
    pub const NAMESPACES: &str = "namespaces";
    pub const NS_SYSTEM: &str = "system";
    pub const NS_DEFAULT: &str = "default";
}
