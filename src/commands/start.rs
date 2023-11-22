// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Args;
use libc::{c_char, c_int};
use nix::errno::Errno;
use nix::sys::socket::{socketpair, AddressFamily, SockFlag, SockType};
use std::collections::HashMap;
use std::ffi::CString;

use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::{Error, ErrorKind};

use std::os::fd::{IntoRawFd, OwnedFd};
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "macos")]
use std::path::Path;
use std::process::Stdio;

use nix::fcntl::{fcntl, FcntlArg, FdFlag};

use crate::bindings;
use crate::bindings::krun_set_passt_fd;
use crate::config::{KrunvmConfig, NetworkMode, VmConfig};
use crate::utils::{mount_container, umount_container};

#[derive(Args, Debug)]
/// Start an existing microVM
pub struct StartCmd {
    /// Name of the microVM
    name: String,

    /// Command to run inside the VM
    command: Option<String>,

    /// Arguments to be passed to the command executed in the VM
    args: Vec<String>,

    /// Number of vCPUs
    #[arg(long)]
    cpus: Option<u8>, // TODO: implement or remove this

    /// Amount of RAM in MiB
    #[arg(long)]
    mem: Option<usize>, // TODO: implement or remove this
}

fn start_passt(mapped_ports: &HashMap<String, String>) -> Result<OwnedFd, ()> {
    let (passt_fd, krun_fd) = socketpair(
        AddressFamily::Unix,
        SockType::Stream,
        None,
        SockFlag::empty(),
    )
    .map_err(|e| {
        eprint!("Failed to create socket pair for passt: {e}");
    })?;

    if let Err(e) = fcntl(krun_fd.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC)) {
        eprint!("Failed to set FD_CLOEXEC: {e}");
    }

    let mut cmd = std::process::Command::new("passt");
    cmd.arg("-q")
        .arg("-f")
        .arg("-F")
        .arg(passt_fd.as_raw_fd().to_string());

    if !mapped_ports.is_empty() {
        let comma_separated_ports = mapped_ports
            .iter()
            .map(|(host_port, guest_port)| format!("{}:{}", host_port, guest_port))
            .collect::<Vec<String>>()
            .join(",");

        cmd.arg("-t").arg(comma_separated_ports);
    }

    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    if let Err(e) = cmd.spawn() {
        eprintln!("Failed to start passt: {e}");
        return Err(());
    }

    Ok(krun_fd)
}

impl StartCmd {
    pub fn run(self, cfg: &KrunvmConfig) {
        let vmcfg = match cfg.vmconfig_map.get(&self.name) {
            None => {
                println!("No VM found with name {}", self.name);
                std::process::exit(-1);
            }
            Some(vmcfg) => vmcfg,
        };

        umount_container(cfg, vmcfg).expect("Error unmounting container");
        let rootfs = mount_container(cfg, vmcfg).expect("Error mounting container");

        let vm_args: Vec<CString> = if self.command.is_some() {
            self.args
                .into_iter()
                .map(|val| CString::new(val).unwrap())
                .collect()
        } else {
            Vec::new()
        };

        set_rlimits();

        let _file = set_lock(&rootfs);

        unsafe { exec_vm(vmcfg, &rootfs, self.command.as_deref(), vm_args) };

        umount_container(cfg, vmcfg).expect("Error unmounting container");
    }
}

#[cfg(target_os = "linux")]
fn map_volumes(_ctx: u32, vmcfg: &VmConfig, rootfs: &str) {
    for (host_path, guest_path) in vmcfg.mapped_volumes.iter() {
        let host_dir = CString::new(host_path.to_string()).unwrap();
        let guest_dir = CString::new(format!("{}{}", rootfs, guest_path)).unwrap();

        let ret = unsafe { libc::mkdir(guest_dir.as_ptr(), 0o755) };
        if ret < 0 && Error::last_os_error().kind() != ErrorKind::AlreadyExists {
            println!("Error creating directory {:?}", guest_dir);
            std::process::exit(-1);
        }
        unsafe { libc::umount(guest_dir.as_ptr()) };
        let ret = unsafe {
            libc::mount(
                host_dir.as_ptr(),
                guest_dir.as_ptr(),
                std::ptr::null(),
                libc::MS_BIND | libc::MS_REC,
                std::ptr::null(),
            )
        };
        if ret < 0 {
            println!("Error mounting volume {}", guest_path);
            std::process::exit(-1);
        }
    }
}

#[cfg(target_os = "macos")]
fn map_volumes(ctx: u32, vmcfg: &VmConfig, rootfs: &str) {
    let mut volumes = Vec::new();
    for (host_path, guest_path) in vmcfg.mapped_volumes.iter() {
        let full_guest = format!("{}{}", &rootfs, guest_path);
        let full_guest_path = Path::new(&full_guest);
        if !full_guest_path.exists() {
            std::fs::create_dir(full_guest_path)
                .expect("Couldn't create guest_path for mapped volume");
        }
        let map = format!("{}:{}", host_path, guest_path);
        volumes.push(CString::new(map).unwrap());
    }
    let mut vols: Vec<*const i8> = Vec::new();
    for vol in volumes.iter() {
        vols.push(vol.as_ptr());
    }
    vols.push(std::ptr::null());
    let ret = unsafe { bindings::krun_set_mapped_volumes(ctx, vols.as_ptr()) };
    if ret < 0 {
        println!("Error setting VM mapped volumes");
        std::process::exit(-1);
    }
}

unsafe fn exec_vm(vmcfg: &VmConfig, rootfs: &str, cmd: Option<&str>, args: Vec<CString>) {
    //bindings::krun_set_log_level(9);

    let ctx = bindings::krun_create_ctx() as u32;

    let ret = bindings::krun_set_vm_config(ctx, vmcfg.cpus as u8, vmcfg.mem);
    if ret < 0 {
        println!("Error setting VM config");
        std::process::exit(-1);
    }

    let c_rootfs = CString::new(rootfs).unwrap();
    let ret = bindings::krun_set_root(ctx, c_rootfs.as_ptr());
    if ret < 0 {
        println!("Error setting VM rootfs");
        std::process::exit(-1);
    }

    map_volumes(ctx, vmcfg, rootfs);

    let mut ports = Vec::new();
    for (host_port, guest_port) in vmcfg.mapped_ports.iter() {
        let map = format!("{}:{}", host_port, guest_port);
        ports.push(CString::new(map).unwrap());
    }
    let mut ps: Vec<*const c_char> = Vec::new();
    for port in ports.iter() {
        ps.push(port.as_ptr());
    }
    ps.push(std::ptr::null());

    match vmcfg.network_mode {
        NetworkMode::Tsi => {
            let ret = bindings::krun_set_port_map(ctx, ps.as_ptr());
            if ret < 0 {
                println!("Error setting VM port map");
                std::process::exit(-1);
            }
        }
        NetworkMode::Passt => {
            let Ok(passt_fd) = start_passt(&vmcfg.mapped_ports) else {
                std::process::exit(-1);
            };
            let ret = krun_set_passt_fd(ctx, passt_fd.into_raw_fd() as c_int);
            if ret < 0 {
                let errno = Errno::from_i32(-ret);
                if errno == Errno::ENOTSUP {
                    println!("Failed to set passt fd: your libkrun build does not support virtio-net/passt mode.");
                } else {
                    println!("Failed to set passt fd: {}", errno);
                }
                std::process::exit(-1);
            }
        }
    }

    if !vmcfg.workdir.is_empty() {
        let c_workdir = CString::new(vmcfg.workdir.clone()).unwrap();
        let ret = bindings::krun_set_workdir(ctx, c_workdir.as_ptr());
        if ret < 0 {
            println!("Error setting VM workdir");
            std::process::exit(-1);
        }
    }

    let hostname = CString::new(format!("HOSTNAME={}", vmcfg.name)).unwrap();
    let home = CString::new("HOME=/root").unwrap();
    let env: [*const c_char; 3] = [hostname.as_ptr(), home.as_ptr(), std::ptr::null()];

    if let Some(cmd) = cmd {
        let mut argv: Vec<*const c_char> = Vec::new();
        for a in args.iter() {
            argv.push(a.as_ptr());
        }
        argv.push(std::ptr::null());

        let c_cmd = CString::new(cmd).unwrap();
        let ret = bindings::krun_set_exec(ctx, c_cmd.as_ptr(), argv.as_ptr(), env.as_ptr());
        if ret < 0 {
            println!("Error setting VM config");
            std::process::exit(-1);
        }
    } else {
        let ret = bindings::krun_set_env(ctx, env.as_ptr());
        if ret < 0 {
            println!("Error setting VM environment variables");
            std::process::exit(-1);
        }
    }

    let ret = bindings::krun_start_enter(ctx);
    if ret < 0 {
        println!("Error starting VM");
        std::process::exit(-1);
    }
}

fn set_rlimits() {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    let ret = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) };
    if ret < 0 {
        panic!("Couldn't get RLIMIT_NOFILE value");
    }

    limit.rlim_cur = limit.rlim_max;
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limit) };
    if ret < 0 {
        panic!("Couldn't set RLIMIT_NOFILE value");
    }
}

fn set_lock(rootfs: &str) -> File {
    let lock_path = format!("{}/.krunvm.lock", rootfs);
    let file = File::create(lock_path).expect("Couldn't create lock file");

    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret < 0 {
        println!("Couldn't acquire lock file. Is another instance of this VM already running?");
        std::process::exit(-1);
    }

    file
}
