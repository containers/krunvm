// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::config::{KrunvmConfig, NetworkMode};
use clap::Args;

/// Configure global values
#[derive(Args, Debug)]
pub struct ConfigCmd {
    // Default number of vCPUs for newly created VMs
    #[arg(long)]
    cpus: Option<u32>,

    ///Default amount of RAM in MiB for newly created VMs
    #[arg(long)]
    mem: Option<u32>,

    /// DNS server to use in the microVM
    #[arg(long)]
    dns: Option<String>,

    /// Default network connection mode to use
    #[arg(long)]
    net: Option<NetworkMode>,
}

impl ConfigCmd {
    pub fn run(self, cfg: &mut KrunvmConfig) {
        let mut cfg_changed = false;

        if let Some(cpus) = self.cpus {
            if cpus > 8 {
                println!("Error: the maximum number of CPUs supported is 8");
            } else {
                cfg.default_cpus = cpus;
                cfg_changed = true;
            }
        }

        if let Some(mem) = self.mem {
            if mem > 16384 {
                println!("Error: the maximum amount of RAM supported is 16384 MiB");
            } else {
                cfg.default_mem = mem;
                cfg_changed = true;
            }
        }

        if let Some(dns) = self.dns {
            cfg.default_dns = dns;
            cfg_changed = true;
        }

        if let Some(network_mode) = self.net {
            if network_mode != cfg.default_network_mode {
                cfg.default_network_mode = network_mode;
                cfg_changed = true;
            }
        }

        if cfg_changed {
            crate::config::save(cfg).unwrap();
        }

        println!("Global config:");
        println!(
            "Default number of CPUs for newly created VMs: {}",
            cfg.default_cpus
        );
        println!(
            "Default amount of RAM (MiB) for newly created VMs: {}",
            cfg.default_mem
        );
        println!(
            "Default DNS server for newly created VMs: {}",
            cfg.default_dns
        );
    }
}
