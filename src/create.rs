// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Args;
use std::fs;
use std::io::Write;
#[cfg(target_os = "macos")]
use std::path::Path;
use std::process::Command;

use super::utils::{get_buildah_args, mount_container, umount_container, BuildahCommand};
use crate::utils::{path_pairs_to_hash_map, port_pairs_to_hash_map, PathPair, PortPair};
use crate::{KrunvmConfig, VmConfig, APP_NAME};

#[cfg(target_os = "macos")]
const KRUNVM_ROSETTA_FILE: &str = ".krunvm-rosetta";

/// Create a new microVM
#[derive(Args, Debug)]
pub struct CreateCmdArgs {
    /// OCI image to use as template
    image: String,

    /// Assign a name to the VM
    #[arg(long)]
    name: Option<String>,

    /// Number of vCPUs
    #[arg(long)]
    cpus: Option<u32>,

    /// Amount of RAM in MiB
    #[arg(long)]
    mem: Option<u32>,

    /// DNS server to use in the microVM
    #[arg(long)]
    dns: Option<String>,

    /// Working directory inside the microVM
    #[arg(short, long, default_value = "")]
    workdir: String,

    /// Volume(s) in form "host_path:guest_path" to be exposed to the guest
    #[arg(short, long = "volume")]
    volumes: Vec<PathPair>,

    /// Port(s) in format "host_port:guest_port" to be exposed to the host
    #[arg(long = "port")]
    ports: Vec<PortPair>,

    /// Create a x86_64 microVM even on an Aarch64 host
    #[arg(short, long)]
    #[cfg(target_os = "macos")]
    x86: bool,
}

fn fix_resolv_conf(rootfs: &str, dns: &str) -> Result<(), std::io::Error> {
    let resolvconf_dir = format!("{}/etc/", rootfs);
    fs::create_dir_all(resolvconf_dir)?;
    let resolvconf = format!("{}/etc/resolv.conf", rootfs);
    let mut file = fs::File::create(resolvconf)?;
    file.write_all(b"options use-vc\nnameserver ")?;
    file.write_all(dns.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn export_container_config(
    cfg: &KrunvmConfig,
    rootfs: &str,
    image: &str,
) -> Result<(), std::io::Error> {
    let mut args = get_buildah_args(cfg, BuildahCommand::Inspect);
    args.push(image.to_string());

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

    let mut file = fs::File::create(format!("{}/.krun_config.json", rootfs))?;
    file.write_all(&output.stdout)?;

    Ok(())
}

pub fn create(cfg: &mut KrunvmConfig, args: CreateCmdArgs) {
    #[allow(unused_mut)]
    let mut cpus = args.cpus.unwrap_or(cfg.default_cpus);
    let mem = args.mem.unwrap_or(cfg.default_mem);
    let dns = args.dns.unwrap_or_else(|| cfg.default_dns.clone());
    let workdir = args.workdir;
    let mapped_volumes = path_pairs_to_hash_map(args.volumes);
    let mapped_ports = port_pairs_to_hash_map(args.ports);
    let image = args.image;
    let name = args.name;

    if let Some(ref name) = name {
        if cfg.vmconfig_map.contains_key(name) {
            println!("A VM with this name already exists");
            std::process::exit(-1);
        }
    }

    let mut buildah_args = get_buildah_args(cfg, BuildahCommand::From);

    #[cfg(target_os = "macos")]
    let force_x86 = args.x86;

    #[cfg(target_os = "macos")]
    if force_x86 {
        let home = match std::env::var("HOME") {
            Err(e) => {
                println!("Error reading \"HOME\" enviroment variable: {}", e);
                std::process::exit(-1);
            }
            Ok(home) => home,
        };

        let path = format!("{}/{}", home, KRUNVM_ROSETTA_FILE);
        if !Path::new(&path).is_file() {
            println!(
                "
To use Rosetta for Linux you need to create the file...

{}

...with the contents that the \"rosetta\" binary expects to be served from
its specific ioctl.

For more information, please refer to this post:
https://threedots.ovh/blog/2022/06/quick-look-at-rosetta-on-linux/
",
                path
            );
            std::process::exit(-1);
        }

        if cpus != 1 {
            println!("x86 microVMs on Aarch64 are restricted to 1 CPU");
            cpus = 1;
        }
        buildah_args.push("--arch".to_string());
        buildah_args.push("x86_64".to_string());
    }

    buildah_args.push(image.to_string());

    let output = match Command::new("buildah")
        .args(&buildah_args)
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

    let container = std::str::from_utf8(&output.stdout).unwrap().trim();
    let name = if let Some(name) = name {
        name.to_string()
    } else {
        container.to_string()
    };
    let vmcfg = VmConfig {
        name: name.clone(),
        cpus,
        mem,
        dns: dns.to_string(),
        container: container.to_string(),
        workdir: workdir.to_string(),
        mapped_volumes,
        mapped_ports,
    };

    let rootfs = mount_container(cfg, &vmcfg).unwrap();
    export_container_config(cfg, &rootfs, &image).unwrap();
    fix_resolv_conf(&rootfs, &dns).unwrap();
    #[cfg(target_os = "macos")]
    if force_x86 {
        _ = fs::create_dir(format!("{}/.rosetta", rootfs));
    }
    umount_container(cfg, &vmcfg).unwrap();

    cfg.vmconfig_map.insert(name.clone(), vmcfg);
    confy::store(APP_NAME, cfg).unwrap();

    println!("microVM created with name: {}", name);
}
