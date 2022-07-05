// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{ArgMatches, KrunvmConfig, VmConfig};

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

pub fn list(cfg: &KrunvmConfig, _matches: &ArgMatches) {
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
