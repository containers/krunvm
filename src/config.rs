// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{KrunvmConfig, APP_NAME};
use clap::Args;

/// Configure global values
#[derive(Args, Debug)]
pub struct ConfigCmdArgs {
    // Default number of vCPUs for newly created VMs
    #[arg(long)]
    cpus: Option<u32>,

    ///Default amount of RAM in MiB for newly created VMs
    #[arg(long)]
    mem: Option<u32>,

    /// DNS server to use in the microVM
    #[arg(long)]
    dns: Option<String>,
}

pub fn config(cfg: &mut KrunvmConfig, args: ConfigCmdArgs) {
    let mut cfg_changed = false;

    if let Some(cpus) = args.cpus {
        if cpus > 8 {
            println!("Error: the maximum number of CPUs supported is 8");
        } else {
            cfg.default_cpus = cpus;
            cfg_changed = true;
        }
    }

    if let Some(mem) = args.mem {
        if mem > 16384 {
            println!("Error: the maximum amount of RAM supported is 16384 MiB");
        } else {
            cfg.default_mem = mem;
            cfg_changed = true;
        }
    }

    if let Some(dns) = args.dns {
        cfg.default_dns = dns;
        cfg_changed = true;
    }

    if cfg_changed {
        confy::store(APP_NAME, &cfg).unwrap();
    }

    println!("Global configuration:");
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
