use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct KrunvmConfig {
    pub version: u8,
    pub default_cpus: u32,
    pub default_mem: u32,
    pub default_dns: String,
    pub storage_volume: String,
    pub vmconfig_map: HashMap<String, VmConfig>,
}

impl Default for KrunvmConfig {
    fn default() -> KrunvmConfig {
        KrunvmConfig {
            version: 1,
            default_cpus: 2,
            default_mem: 1024,
            default_dns: "1.1.1.1".to_string(),
            storage_volume: String::new(),
            vmconfig_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VmConfig {
    pub name: String,
    pub cpus: u32,
    pub mem: u32,
    pub container: String,
    pub workdir: String,
    pub dns: String,
    pub mapped_volumes: HashMap<String, String>,
    pub mapped_ports: HashMap<String, String>,
}
