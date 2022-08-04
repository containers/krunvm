// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::{KrunvmConfig, VmConfig, APP_NAME};

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

pub fn parse_mapped_ports(port_matches: Vec<&str>) -> HashMap<String, String> {
    let mut mapped_ports = HashMap::new();
    for port in port_matches.iter() {
        let vtuple: Vec<&str> = port.split(':').collect();
        if vtuple.len() != 2 {
            println!("Invalid value for \"port\"");
            std::process::exit(-1);
        }
        let host_port: u16 = match vtuple[0].parse() {
            Ok(p) => p,
            Err(_) => {
                println!("Invalid host port");
                std::process::exit(-1);
            }
        };
        let guest_port: u16 = match vtuple[1].parse() {
            Ok(p) => p,
            Err(_) => {
                println!("Invalid guest port");
                std::process::exit(-1);
            }
        };

        mapped_ports.insert(host_port.to_string(), guest_port.to_string());
    }

    mapped_ports
}

pub fn parse_mapped_volumes(volume_matches: Vec<&str>) -> HashMap<String, String> {
    let mut mapped_volumes = HashMap::new();
    for volume in volume_matches.iter() {
        let vtuple: Vec<&str> = volume.split(':').collect();
        if vtuple.len() != 2 {
            println!("Invalid value for \"volume\"");
            std::process::exit(-1);
        }
        let host_path = Path::new(vtuple[0]);
        if !host_path.is_absolute() {
            println!("Invalid volume, host_path is not an absolute path");
            std::process::exit(-1);
        }
        if !host_path.exists() {
            println!("Invalid volume, host_path does not exists");
            std::process::exit(-1);
        }
        let guest_path = Path::new(vtuple[1]);
        if !guest_path.is_absolute() {
            println!("Invalid volume, guest_path is not an absolute path");
            std::process::exit(-1);
        }
        if guest_path.components().count() != 2 {
            println!(
                "Invalid volume, only single direct root children are supported as guest_path"
            );
            std::process::exit(-1);
        }
        mapped_volumes.insert(
            host_path.to_str().unwrap().to_string(),
            guest_path.to_str().unwrap().to_string(),
        );
    }

    mapped_volumes
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
