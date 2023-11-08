// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::APP_NAME;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use crate::config::{KrunvmConfig, VmConfig};

pub enum BuildahCommand {
    From,
    Inspect,
    Mount,
    Unmount,
    Remove,
}

#[cfg(target_os = "linux")]
pub fn get_buildah_args(_cfg: &KrunvmConfig, cmd: BuildahCommand) -> Vec<String> {
    match cmd {
        BuildahCommand::From => vec!["from".to_string()],
        BuildahCommand::Inspect => vec!["inspect".to_string()],
        BuildahCommand::Mount => vec!["mount".to_string()],
        BuildahCommand::Unmount => vec!["umount".to_string()],
        BuildahCommand::Remove => vec!["rm".to_string()],
    }
}

#[cfg(target_os = "macos")]
pub fn get_buildah_args(cfg: &KrunvmConfig, cmd: BuildahCommand) -> Vec<String> {
    let mut hbpath = std::env::current_exe().unwrap();
    hbpath.pop();
    hbpath.pop();
    let hbpath = hbpath.as_path().display();
    let policy_json = format!("{}/etc/containers/policy.json", hbpath);
    let registries_json = format!("{}/etc/containers/registries.conf", hbpath);
    let storage_root = format!("{}/root", cfg.storage_volume);
    let storage_runroot = format!("{}/runroot", cfg.storage_volume);

    let mut args = vec![
        "--root".to_string(),
        storage_root,
        "--runroot".to_string(),
        storage_runroot,
    ];

    match cmd {
        BuildahCommand::From => {
            args.push("--signature-policy".to_string());
            args.push(policy_json);
            args.push("--registries-conf".to_string());
            args.push(registries_json);

            args.push("from".to_string());
            args.push("--os".to_string());
            args.push("linux".to_string());
        }
        BuildahCommand::Inspect => {
            args.push("inspect".to_string());
        }
        BuildahCommand::Mount => {
            args.push("mount".to_string());
        }
        BuildahCommand::Unmount => {
            args.push("umount".to_string());
        }
        BuildahCommand::Remove => {
            args.push("rm".to_string());
        }
    }
    args
}

#[derive(Debug, Clone)]
pub struct PortPair {
    pub host_port: String,
    pub guest_port: String,
}

pub fn port_pairs_to_hash_map(
    port_pairs: impl IntoIterator<Item = PortPair>,
) -> HashMap<String, String> {
    port_pairs
        .into_iter()
        .map(|pair: PortPair| (pair.host_port, pair.guest_port))
        .collect()
}

impl FromStr for PortPair {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let vtuple: Vec<&str> = input.split(':').collect();
        if vtuple.len() != 2 {
            return Err("Too many ':' separators");
        }
        let host_port: u16 = match vtuple[0].parse() {
            Ok(p) => p,
            Err(_) => {
                return Err("Invalid host port");
            }
        };
        let guest_port: u16 = match vtuple[1].parse() {
            Ok(p) => p,
            Err(_) => {
                return Err("Invalid guest port");
            }
        };
        Ok(PortPair {
            host_port: host_port.to_string(),
            guest_port: guest_port.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PathPair {
    pub host_path: String,
    pub guest_path: String,
}

pub fn path_pairs_to_hash_map(
    volume_pairs: impl IntoIterator<Item = PathPair>,
) -> HashMap<String, String> {
    volume_pairs
        .into_iter()
        .map(|pair: PathPair| (pair.host_path, pair.guest_path))
        .collect()
}

impl FromStr for PathPair {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let vtuple: Vec<&str> = input.split(':').collect();
        if vtuple.len() != 2 {
            return Err("Too many ':' separators");
        }

        let host_path = Path::new(vtuple[0]);
        if !host_path.is_absolute() {
            return Err("Invalid volume, host_path is not an absolute path");
        }
        if !host_path.exists() {
            return Err("Invalid volume, host_path does not exists");
        }
        let guest_path = Path::new(vtuple[1]);
        if !guest_path.is_absolute() {
            return Err("Invalid volume, guest_path is not an absolute path");
        }
        if guest_path.components().count() != 2 {
            return Err(
                "Invalid volume, only single direct root children are supported as guest_path",
            );
        }
        Ok(Self {
            host_path: vtuple[0].to_string(),
            guest_path: vtuple[1].to_string(),
        })
    }
}

#[cfg(target_os = "macos")]
fn fix_root_mode(rootfs: &str) {
    let mut args = vec!["-w", "user.containers.override_stat", "0:0:0555"];
    args.push(rootfs);

    let output = match Command::new("xattr")
        .args(&args)
        .stderr(std::process::Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("{} requires xattr to manage the OCI images, and it wasn't found on this system.", APP_NAME);
            } else {
                println!("Error executing xattr: {}", err);
            }
            std::process::exit(-1);
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != 0 {
        println!("xattr returned an error: {}", exit_code);
        std::process::exit(-1);
    }
}

#[allow(unused_variables)]
pub fn mount_container(cfg: &KrunvmConfig, vmcfg: &VmConfig) -> Result<String, std::io::Error> {
    let mut args = get_buildah_args(cfg, BuildahCommand::Mount);
    args.push(vmcfg.container.clone());

    let output = match Command::new("buildah")
        .args(&args)
        .stderr(std::process::Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("{} requires buildah to manage the OCI images, and it wasn't found on this system.", APP_NAME);
            } else {
                println!("Error executing buildah: {}", err);
            }
            std::process::exit(-1);
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != 0 {
        println!(
            "buildah returned an error: {}",
            std::str::from_utf8(&output.stdout).unwrap()
        );
        std::process::exit(-1);
    }

    let rootfs = std::str::from_utf8(&output.stdout).unwrap().trim();

    #[cfg(target_os = "macos")]
    fix_root_mode(rootfs);

    Ok(rootfs.to_string())
}

#[allow(unused_variables)]
pub fn umount_container(cfg: &KrunvmConfig, vmcfg: &VmConfig) -> Result<(), std::io::Error> {
    let mut args = get_buildah_args(cfg, BuildahCommand::Unmount);
    args.push(vmcfg.container.clone());

    let output = match Command::new("buildah")
        .args(&args)
        .stderr(std::process::Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("{} requires buildah to manage the OCI images, and it wasn't found on this system.", APP_NAME);
            } else {
                println!("Error executing buildah: {}", err);
            }
            std::process::exit(-1);
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != 0 {
        println!(
            "buildah returned an error: {}",
            std::str::from_utf8(&output.stdout).unwrap()
        );
        std::process::exit(-1);
    }

    Ok(())
}

#[allow(unused_variables)]
pub fn remove_container(cfg: &KrunvmConfig, vmcfg: &VmConfig) -> Result<(), std::io::Error> {
    let mut args = get_buildah_args(cfg, BuildahCommand::Remove);
    args.push(vmcfg.container.clone());

    let output = match Command::new("buildah")
        .args(&args)
        .stderr(std::process::Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("{} requires buildah to manage the OCI images, and it wasn't found on this system.", APP_NAME);
            } else {
                println!("Error executing buildah: {}", err);
            }
            std::process::exit(-1);
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != 0 {
        println!(
            "buildah returned an error: {}",
            std::str::from_utf8(&output.stdout).unwrap()
        );
        std::process::exit(-1);
    }

    Ok(())
}
