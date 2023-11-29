// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Args;
use std::collections::HashMap;

use crate::config::{KrunvmConfig, NetworkMode};
use crate::utils::{path_pairs_to_hash_map, port_pairs_to_hash_map, PathPair, PortPair};

use super::list::printvm;

/// Change the configuration of a microVM
#[derive(Args, Debug)]
pub struct ChangeVmCmd {
    /// Name of the VM to be modified
    name: String,

    /// Assign a new name to the VM
    #[arg(long)]
    new_name: Option<String>,

    /// Number of vCPUs
    #[arg(long)]
    cpus: Option<u32>,

    /// Amount of RAM in MiB
    #[arg(long)]
    mem: Option<u32>,

    /// Working directory inside the microVM
    #[arg(short, long)]
    workdir: Option<String>,

    /// Remove all volume mappings
    #[arg(long)]
    remove_volumes: bool,

    /// Volume(s) in form "host_path:guest_path" to be exposed to the guest
    #[arg(short, long = "volume")]
    volumes: Vec<PathPair>,

    /// Remove all port mappings
    #[arg(long)]
    remove_ports: bool,

    /// Port(s) in format "host_port:guest_port" to be exposed to the host
    #[arg(long = "port")]
    ports: Vec<PortPair>,

    /// Set the network connection mode for the microVM
    #[arg(long)]
    net: Option<NetworkMode>,
}

impl ChangeVmCmd {
    pub fn run(self, cfg: &mut KrunvmConfig) {
        let mut cfg_changed = false;

        let vmcfg = if let Some(new_name) = &self.new_name {
            if cfg.vmconfig_map.contains_key(new_name) {
                println!("A VM with name {} already exists", new_name);
                std::process::exit(-1);
            }

            let mut vmcfg = match cfg.vmconfig_map.remove(&self.name) {
                None => {
                    println!("No VM found with name {}", &self.name);
                    std::process::exit(-1);
                }
                Some(vmcfg) => vmcfg,
            };

            cfg_changed = true;
            let name = new_name.to_string();
            vmcfg.name = name.clone();
            cfg.vmconfig_map.insert(name.clone(), vmcfg);
            cfg.vmconfig_map.get_mut(&name).unwrap()
        } else {
            match cfg.vmconfig_map.get_mut(&self.name) {
                None => {
                    println!("No VM found with name {}", self.name);
                    std::process::exit(-1);
                }
                Some(vmcfg) => vmcfg,
            }
        };

        if let Some(cpus) = self.cpus {
            if cpus > 8 {
                println!("Error: the maximum number of CPUs supported is 8");
            } else {
                vmcfg.cpus = cpus;
                cfg_changed = true;
            }
        }

        if let Some(mem) = self.mem {
            if mem > 16384 {
                println!("Error: the maximum amount of RAM supported is 16384 MiB");
            } else {
                vmcfg.mem = mem;
                cfg_changed = true;
            }
        }

        if self.remove_volumes {
            vmcfg.mapped_volumes = HashMap::new();
            cfg_changed = true;
        } else {
            let mapped_volumes = path_pairs_to_hash_map(self.volumes);

            if !mapped_volumes.is_empty() {
                vmcfg.mapped_volumes = mapped_volumes;
                cfg_changed = true;
            }
        }
        // TODO: don't just silently ignore --volume args when --remove_volumes is specified

        if self.remove_ports {
            vmcfg.mapped_ports = HashMap::new();
            cfg_changed = true;
        } else {
            let mapped_ports = port_pairs_to_hash_map(self.ports);

            if !mapped_ports.is_empty() {
                vmcfg.mapped_ports = mapped_ports;
                cfg_changed = true;
            }
        }
        // TODO: don't just silently ignore --port args when --remove_ports is specified

        if let Some(workdir) = self.workdir {
            vmcfg.workdir = workdir.to_string();
            cfg_changed = true;
        }

        if let Some(network_mode) = self.net {
            vmcfg.network_mode = network_mode;
            cfg_changed = true;
        }

        println!();
        printvm(vmcfg);
        println!();

        if cfg_changed {
            crate::config::save(cfg).unwrap();
        }
    }
}
