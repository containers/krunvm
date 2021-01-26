// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::{ArgMatches, KrunvmConfig, APP_NAME};

use super::list::printvm;
use super::utils::{parse_mapped_ports, parse_mapped_volumes};

pub fn changevm(cfg: &mut KrunvmConfig, matches: &ArgMatches) {
    let mut cfg_changed = false;

    let name = matches.value_of("NAME").unwrap();

    let mut vmcfg = if let Some(new_name) = matches.value_of("new-name") {
        if cfg.vmconfig_map.contains_key(new_name) {
            println!("A VM with name {} already exists", new_name);
            std::process::exit(-1);
        }

        let mut vmcfg = match cfg.vmconfig_map.remove(name) {
            None => {
                println!("No VM found with name {}", name);
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
        match cfg.vmconfig_map.get_mut(name) {
            None => {
                println!("No VM found with name {}", name);
                std::process::exit(-1);
            }
            Some(vmcfg) => vmcfg,
        }
    };

    if let Some(cpus_str) = matches.value_of("cpus") {
        match cpus_str.parse::<u32>() {
            Err(_) => println!("Invalid value for \"cpus\""),
            Ok(cpus) => {
                if cpus > 8 {
                    println!("Error: the maximum number of CPUs supported is 8");
                } else {
                    vmcfg.cpus = cpus;
                    cfg_changed = true;
                }
            }
        }
    }

    if let Some(mem_str) = matches.value_of("mem") {
        match mem_str.parse::<u32>() {
            Err(_) => println!("Invalid value for \"mem\""),
            Ok(mem) => {
                if mem > 16384 {
                    println!("Error: the maximum amount of RAM supported is 16384 MiB");
                } else {
                    vmcfg.mem = mem;
                    cfg_changed = true;
                }
            }
        }
    }

    if matches.is_present("remove-volumes") {
        vmcfg.mapped_volumes = HashMap::new();
        cfg_changed = true;
    } else {
        let volume_matches = if matches.is_present("volume") {
            matches.values_of("volume").unwrap().collect()
        } else {
            vec![]
        };
        let mapped_volumes = parse_mapped_volumes(volume_matches);

        if !mapped_volumes.is_empty() {
            vmcfg.mapped_volumes = mapped_volumes;
            cfg_changed = true;
        }
    }

    if matches.is_present("remove-ports") {
        vmcfg.mapped_ports = HashMap::new();
        cfg_changed = true;
    } else {
        let port_matches = if matches.is_present("port") {
            matches.values_of("port").unwrap().collect()
        } else {
            vec![]
        };
        let mapped_ports = parse_mapped_ports(port_matches);

        if !mapped_ports.is_empty() {
            vmcfg.mapped_ports = mapped_ports;
            cfg_changed = true;
        }
    }

    if let Some(workdir) = matches.value_of("workdir") {
        vmcfg.workdir = workdir.to_string();
        cfg_changed = true;
    }

    println!();
    printvm(vmcfg);
    println!();

    if cfg_changed {
        confy::store(APP_NAME, &cfg).unwrap();
    }
}
