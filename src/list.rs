// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{KrunvmConfig, VmConfig};
use clap::Args;

/// List microVMs
#[derive(Args, Debug)]
pub struct ListCmdArgs {
    /// Print debug information verbosely
    #[arg(short)]
    debug: bool, //TODO: implement or remove this
}

pub fn printvm(vm: &VmConfig) {
    println!("{}", vm.name);
    println!(" CPUs: {}", vm.cpus);
    println!(" RAM (MiB): {}", vm.mem);
    println!(" DNS server: {}", vm.dns);
    println!(" Buildah container: {}", vm.container);
    println!(" Workdir: {}", vm.workdir);
    println!(" Mapped volumes: {:?}", vm.mapped_volumes);
    println!(" Mapped ports: {:?}", vm.mapped_ports);
}

pub fn list(cfg: &KrunvmConfig, _args: ListCmdArgs) {
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
