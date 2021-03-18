// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::fs::File;
#[cfg(target_os = "macos")]
use std::io::{self, Read, Write};

use clap::ArgMatches;
use serde_derive::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use text_io::read;

#[allow(unused)]
pub mod bindings;
pub mod changevm;
pub mod config;
pub mod create;
pub mod delete;
pub mod list;
pub mod start;
pub mod utils;

const LIBRARY_NAME: &str = "krunvm";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct VmConfig {
    name: String,
    cpus: u32,
    mem: u32,
    container: String,
    workdir: String,
    dns: String,
    mapped_volumes: HashMap<String, String>,
    mapped_ports: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KrunvmConfig {
    version: u8,
    default_cpus: u32,
    default_mem: u32,
    default_dns: String,
    storage_volume: String,
    vmconfig_map: HashMap<String, VmConfig>,
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

#[cfg(target_os = "macos")]
pub fn check_case_sensitivity(volume: &str) -> Result<bool, io::Error> {
    let first_path = format!("{}/krunvm_test", volume);
    let second_path = format!("{}/krunVM_test", volume);
    {
        let mut first = File::create(&first_path)?;
        first.write_all(b"first")?;
    }
    {
        let mut second = File::create(&second_path)?;
        second.write_all(b"second")?;
    }
    let mut data = String::new();
    {
        let mut test = File::open(&first_path)?;

        test.read_to_string(&mut data)?;
    }
    if data == "first" {
        let _ = std::fs::remove_file(first_path);
        let _ = std::fs::remove_file(second_path);
        Ok(true)
    } else {
        let _ = std::fs::remove_file(first_path);
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
pub fn check_volume(cfg: &mut KrunvmConfig) {
    if !cfg.storage_volume.is_empty() {
        return;
    }

    println!(
        "
On macOS, krunvm requires a dedicated, case-sensitive volume.
You can easily create such volume by executing something like
this on another terminal:

diskutil apfs addVolume disk3 \"Case-sensitive APFS\" krunvm

NOTE: APFS volume creation is a non-destructive action that
doesn't require a dedicated disk nor \"sudo\" privileges. The
new volume will share the disk space with the main container
volume.
"
    );
    loop {
        print!("Please enter the mountpoint for this volume [/Volumes/krunvm]: ");
        io::stdout().flush().unwrap();
        let answer: String = read!("{}\n");

        let volume = if answer.is_empty() {
            "/Volumes/krunvm".to_string()
        } else {
            answer.to_string()
        };

        print!("Checking volume... ");
        match check_case_sensitivity(&volume) {
            Ok(res) => {
                if res {
                    println!("success.");
                    println!("The volume has been configured. Please execute krunvm again");
                    cfg.storage_volume = volume;
                    confy::store(LIBRARY_NAME, cfg).unwrap();
                    std::process::exit(-1);
                } else {
                    println!("failed.");
                    println!("This volume failed the case sensitivity test.");
                }
            }
            Err(err) => {
                println!("error.");
                println!("There was an error running the test: {}", err);
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn check_unshare() {
    let uid = unsafe { libc::getuid() };
    if uid != 0
        && std::env::vars()
            .find(|(key, _)| key == "BUILDAH_ISOLATION")
            .is_none()
    {
        println!("Please re-run krunvm inside a \"buildah unshare\" session");
        std::process::exit(-1);
    }
}
