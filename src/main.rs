// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::fs::File;
#[cfg(target_os = "macos")]
use std::io::{self, Read, Write};

use crate::changevm::ChangeVmCmdArgs;
use crate::config::ConfigCmdArgs;
use crate::create::CreateCmdArgs;
use crate::delete::DeleteCmdArgs;
use crate::list::ListCmdArgs;
use crate::start::StartCmdArgs;
use clap::{Parser, Subcommand};
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

#[cfg(target_os = "linux")]
fn check_unshare() {
    let uid = unsafe { libc::getuid() };
    if uid != 0 && !std::env::vars().any(|(key, _)| key == "BUILDAH_ISOLATION") {
        println!("Please re-run krunvm inside a \"buildah unshare\" session");
        std::process::exit(-1);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Sets the level of verbosity
    #[arg(short)]
    verbosity: Option<u8>, //TODO: implement or remove this
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Start(StartCmdArgs),
    Create(CreateCmdArgs),
    List(ListCmdArgs),
    Delete(DeleteCmdArgs),
    #[command(name = "changevm")]
    ChangeVm(ChangeVmCmdArgs),
    Config(ConfigCmdArgs),
}

fn main() {
    let mut cfg: KrunvmConfig = confy::load(APP_NAME).unwrap();
    let cli_args = Cli::parse();

    #[cfg(target_os = "macos")]
    check_volume(&mut cfg);
    #[cfg(target_os = "linux")]
    check_unshare();

    match cli_args.command {
        Command::Start(args) => start::start(&cfg, args),
        Command::Create(args) => create::create(&mut cfg, args),
        Command::List(args) => list::list(&cfg, args),
        Command::Delete(args) => delete::delete(&mut cfg, args),
        Command::ChangeVm(args) => changevm::changevm(&mut cfg, args),
        Command::Config(args) => config::config(&mut cfg, args),
    }
}
