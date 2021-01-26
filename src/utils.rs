// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::{KrunvmConfig, VmConfig, APP_NAME};

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

pub fn mount_container(cfg: &KrunvmConfig, vmcfg: &VmConfig) -> Result<String, std::io::Error> {
    #[cfg(target_os = "macos")]
    let storage_root = format!("{}/root", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let storage_runroot = format!("{}/runroot", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let mut args = vec![
        "--root",
        &storage_root,
        "--runroot",
        &storage_runroot,
        "mount",
    ];
    #[cfg(target_os = "linux")]
    let mut args = vec!["mount"];

    args.push(&vmcfg.container);

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
                println!("Error executing buildah: {}", err.to_string());
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

pub fn umount_container(cfg: &KrunvmConfig, vmcfg: &VmConfig) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    let storage_root = format!("{}/root", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let storage_runroot = format!("{}/runroot", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let mut args = vec![
        "--root",
        &storage_root,
        "--runroot",
        &storage_runroot,
        "umount",
    ];
    #[cfg(target_os = "linux")]
    let mut args = vec!["umount"];

    args.push(&vmcfg.container);

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
                println!("Error executing buildah: {}", err.to_string());
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

pub fn remove_container(cfg: &KrunvmConfig, vmcfg: &VmConfig) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    let storage_root = format!("{}/root", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let storage_runroot = format!("{}/runroot", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let mut args = vec!["--root", &storage_root, "--runroot", &storage_runroot, "rm"];
    #[cfg(target_os = "linux")]
    let mut args = vec!["rm"];

    args.push(&vmcfg.container);

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
                println!("Error executing buildah: {}", err.to_string());
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
