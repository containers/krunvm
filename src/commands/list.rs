// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::config::{KrunvmConfig, VmConfig};
use clap::Args;

/// List microVMs
#[derive(Args, Debug)]
pub struct ListCmd {
    /// Print debug information verbosely
    #[arg(short)]
    pub debug: bool, //TODO: implement or remove this
}

impl ListCmd {
    pub fn run(self, cfg: &KrunvmConfig) {
        if cfg.vmconfig_map.is_empty() {
            println!("No microVMs found");
        } else {
            for (_name, vm) in cfg.vmconfig_map.iter() {
                println!();
                printvm(vm);
            }
            println!();
        }
    }
}

pub fn printvm(vm: &VmConfig) {
    println!("{}", vm.name);
    println!(" CPUs: {}", vm.cpus);
    println!(" RAM (MiB): {}", vm.mem);
    println!(" DNS server: {}", vm.dns);
    println!(" Buildah container: {}", vm.container);
    println!(" Workdir: {}", vm.workdir);
    println!(" Network mode: {:?}", vm.network_mode);
    println!(" Mapped volumes: {:?}", vm.mapped_volumes);
    println!(" Mapped ports: {:?}", vm.mapped_ports);
}
