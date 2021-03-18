// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{ArgMatches, KrunvmConfig, LIBRARY_NAME};

pub fn config(cfg: &mut KrunvmConfig, matches: &ArgMatches) {
    let mut cfg_changed = false;

    if let Some(cpus_str) = matches.value_of("cpus") {
        match cpus_str.parse::<u32>() {
            Err(_) => println!("Invalid value for \"cpus\""),
            Ok(cpus) => {
                if cpus > 8 {
                    println!("Error: the maximum number of CPUs supported is 8");
                } else {
                    cfg.default_cpus = cpus;
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
                    cfg.default_mem = mem;
                    cfg_changed = true;
                }
            }
        }
    }

    if let Some(dns) = matches.value_of("dns") {
        cfg.default_dns = dns.to_string();
        cfg_changed = true;
    }

    if cfg_changed {
        confy::store(LIBRARY_NAME, &cfg).unwrap();
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
