// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::fs::File;
#[cfg(target_os = "macos")]
use std::io::{self, Read, Write};

use clap::{crate_version, App, Arg, ArgMatches};
use serde_derive::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use text_io::read;

#[allow(unused)]
mod bindings;
mod changevm;
mod config;
mod create;
mod delete;
mod list;
mod start;
mod utils;

const APP_NAME: &str = "krunvm";

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
fn check_case_sensitivity(volume: &str) -> Result<bool, io::Error> {
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
fn check_volume(cfg: &mut KrunvmConfig) {
    if !cfg.storage_volume.is_empty() {
        return;
    }

    println!(
        "
On macOS, krunvm requires a dedicated, case-sensitive volume.
You can easily such volume by executing something like this on
another terminal:

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
                    confy::store(APP_NAME, cfg).unwrap();
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

fn check_unshare() {
    if std::env::vars()
        .find(|(key, _)| key == "BUILDAH_ISOLATION")
        .is_none()
    {
        println!("Please re-run krunvm inside a \"buildah unshare\" session");
        std::process::exit(-1);
    }
}

fn main() {
    let mut cfg: KrunvmConfig = confy::load(APP_NAME).unwrap();

    let mut app = App::new("krunvm")
        .version(crate_version!())
        .author("Sergio Lopez <slp@redhat.com>")
        .about("Manage lightweight VMs created from OCI images")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            App::new("changevm")
                .about("Change the configuration of a lightweight VM")
                .arg(Arg::with_name("cpus").long("cpus").help("Number of vCPUs"))
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Amount of RAM in MiB"),
                )
                .arg(
                    Arg::with_name("workdir")
                        .long("workdir")
                        .short("w")
                        .help("Working directory inside the lightweight VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("remove-volumes")
                        .long("remove-volumes")
                        .help("Remove all volume mappings"),
                )
                .arg(
                    Arg::with_name("volume")
                        .long("volume")
                        .short("v")
                        .help("Volume in form \"host_path:guest_path\" to be exposed to the guest")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("remove-ports")
                        .long("remove-ports")
                        .help("Remove all port mappings"),
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .help("Port in format \"host_port:guest_port\" to be exposed to the host")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("new-name")
                        .long("name")
                        .help("Assign a new name to the VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the VM to be modified")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("config")
                .about("Configure global values")
                .arg(
                    Arg::with_name("cpus")
                        .long("cpus")
                        .help("Default number of vCPUs for newly created VMs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Default amount of RAM in MiB for newly created VMs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dns")
                        .long("dns")
                        .help("DNS server to use in the lightweight VM")
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("create")
                .about("Create a new lightweight VM")
                .arg(
                    Arg::with_name("cpus")
                        .long("cpus")
                        .help("Number of vCPUs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Amount of RAM in MiB")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dns")
                        .long("dns")
                        .help("DNS server to use in the lightweight VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("workdir")
                        .long("workdir")
                        .short("w")
                        .help("Working directory inside the lightweight VM")
                        .takes_value(true)
                        .default_value("/root"),
                )
                .arg(
                    Arg::with_name("volume")
                        .long("volume")
                        .short("v")
                        .help("Volume in form \"host_path:guest_path\" to be exposed to the guest")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .help("Port in format \"host_port:guest_port\" to be exposed to the host")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("name")
                        .long("name")
                        .help("Assign a name to the VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("IMAGE")
                        .help("OCI image to use as template")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("delete")
                .about("Delete an existing lightweight VM")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the lightweight VM to be deleted")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            App::new("list").about("List lightweight VMs").arg(
                Arg::with_name("debug")
                    .short("d")
                    .help("print debug information verbosely"),
            ),
        )
        .subcommand(
            App::new("start")
                .about("Start an existing lightweight VM")
                .arg(Arg::with_name("cpus").long("cpus").help("Number of vCPUs"))
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Amount of RAM in MiB"),
                )
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the lightweight VM")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("COMMAND")
                        .help("Command to run inside the VM")
                        .index(2)
                        .default_value("/bin/sh"),
                )
                .arg(
                    Arg::with_name("ARGS")
                        .help("Arguments to be passed to the command executed in the VM")
                        .multiple(true)
                        .last(true),
                ),
        );

    let matches = app.clone().get_matches();

    #[cfg(target_os = "macos")]
    check_volume(&mut cfg);
    #[cfg(target_os = "linux")]
    check_unshare();

    if let Some(ref matches) = matches.subcommand_matches("changevm") {
        changevm::changevm(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("config") {
        config::config(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("create") {
        create::create(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("delete") {
        delete::delete(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("list") {
        list::list(&cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("start") {
        start::start(&cfg, matches);
    } else {
        app.print_long_help().unwrap();
        println!();
    }
}
