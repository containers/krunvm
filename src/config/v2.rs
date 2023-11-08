use crate::config::v1;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub enum NetworkMode {
    #[default]
    Tsi,
    Passt,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct KrunvmConfig {
    pub version: u8,
    pub default_cpus: u32,
    pub default_mem: u32,
    pub default_dns: String,
    pub default_network_mode: NetworkMode,
    pub storage_volume: String,
    pub vmconfig_map: HashMap<String, VmConfig>,
}

impl Default for KrunvmConfig {
    fn default() -> KrunvmConfig {
        KrunvmConfig {
            version: 2,
            default_cpus: 2,
            default_mem: 1024,
            default_dns: "1.1.1.1".to_string(),
            default_network_mode: NetworkMode::default(),
            storage_volume: String::new(),
            vmconfig_map: HashMap::new(),
        }
    }
}

impl From<v1::KrunvmConfig> for KrunvmConfig {
    fn from(old: v1::KrunvmConfig) -> Self {
        KrunvmConfig {
            version: 2,
            default_cpus: old.default_cpus,
            default_mem: old.default_mem,
            default_dns: old.default_dns,
            default_network_mode: NetworkMode::default(),
            storage_volume: old.storage_volume,
            vmconfig_map: old
                .vmconfig_map
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VmConfig {
    pub name: String,
    pub cpus: u32,
    pub mem: u32,
    pub container: String,
    pub workdir: String,
    pub dns: String,
    pub network_mode: NetworkMode,
    pub mapped_volumes: HashMap<String, String>,
    pub mapped_ports: HashMap<String, String>,
}

impl From<v1::VmConfig> for VmConfig {
    fn from(old: v1::VmConfig) -> Self {
        VmConfig {
            name: old.name,
            cpus: old.cpus,
            mem: old.mem,
            container: old.container,
            workdir: old.workdir,
            dns: old.dns,
            mapped_volumes: old.mapped_volumes,
            mapped_ports: old.mapped_ports,
            network_mode: NetworkMode::default(),
        }
    }
}

impl FromStr for NetworkMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("tsi") {
            Ok(NetworkMode::Tsi)
        } else if s.eq_ignore_ascii_case("passt") {
            Ok(NetworkMode::Passt)
        } else {
            Err("Invalid network mode")
        }
    }
}
