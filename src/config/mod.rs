mod migrate;
mod v1;
mod v2;

use crate::APP_NAME;

use crate::config::migrate::migrate_and_load_impl;
pub use v2::{KrunvmConfig, NetworkMode, VmConfig};

pub fn save(cfg: &KrunvmConfig) -> Result<(), ()> {
    confy::store(APP_NAME, cfg).map_err(|e| eprintln!("Failed to load config: {e}"))
}

pub fn load() -> Result<v2::KrunvmConfig, ()> {
    migrate_and_load_impl(
        || confy::load::<v2::KrunvmConfig>(APP_NAME),
        || confy::load::<v1::KrunvmConfig>(APP_NAME),
        save,
    )
}
