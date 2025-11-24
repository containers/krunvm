// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Args;
use libc::c_char;
use std::ffi::CString;
use std::fs::File;
#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "linux")]
use std::io::{Error, ErrorKind};
#[cfg(target_os = "macos")]
use std::io::Write;
use std::os::unix::io::AsRawFd;
#[cfg(target_os = "macos")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "macos")]
use std::path::Path;

use crate::bindings;
use crate::utils::{mount_container, umount_container};
use crate::{KrunvmConfig, VmConfig};

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

    /// env(s) in format "key=value" to be exposed to the VM
    #[arg(long = "env")]
    envs: Option<Vec<String>>,
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

        let env_pairs: Vec<CString> = if self.envs.is_some() {
            self.envs
                .unwrap()
                .into_iter()
                .map(|val| CString::new(val).unwrap())
                .collect()
        } else {
            Vec::new()
        };

        set_rlimits();

        let _file = set_lock(&rootfs);

        unsafe { exec_vm(vmcfg, &rootfs, self.command.as_deref(), vm_args, env_pairs) };

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
fn map_volumes(ctx: u32, vmcfg: &VmConfig, rootfs: &str) -> Vec<(String, String)> {
    let mut mounts = Vec::new();
    for (idx, (host_path, guest_path)) in vmcfg.mapped_volumes.iter().enumerate() {
        let full_guest = format!("{}{}", &rootfs, guest_path);
        let full_guest_path = Path::new(&full_guest);
        if !full_guest_path.exists() {
            std::fs::create_dir(full_guest_path)
                .expect("Couldn't create guest_path for mapped volume");
        }
        let tag = format!("krunvm{}", idx);
        let c_tag = CString::new(tag.as_str()).unwrap();
        let c_host = CString::new(host_path.as_str()).unwrap();
        let ret = unsafe { bindings::krun_add_virtiofs(ctx, c_tag.as_ptr(), c_host.as_ptr()) };
        if ret < 0 {
            println!("Error setting VM mapped volume {}", guest_path);
            std::process::exit(-1);
        }
        mounts.push((tag, guest_path.to_string()));
    }
    mounts
}

unsafe fn exec_vm(
    vmcfg: &VmConfig,
    rootfs: &str,
    cmd: Option<&str>,
    args: Vec<CString>,
    env_pairs: Vec<CString>,
) {
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

    #[cfg(target_os = "linux")]
    map_volumes(ctx, vmcfg, rootfs);
    #[cfg(target_os = "macos")]
    let virtiofs_mounts = map_volumes(ctx, vmcfg, rootfs);
    #[cfg(target_os = "macos")]
    let mount_wrapper = build_mount_wrapper(rootfs, cmd, &args, &virtiofs_mounts);

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

    let ret = bindings::krun_set_port_map(ctx, ps.as_ptr());
    if ret < 0 {
        println!("Error setting VM port map");
        std::process::exit(-1);
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

    let mut env: Vec<*const c_char> = Vec::new();
    env.push(hostname.as_ptr());
    env.push(home.as_ptr());
    for value in env_pairs.iter() {
        env.push(value.as_ptr());
    }
    env.push(std::ptr::null());

    #[cfg(target_os = "macos")]
    {
        if let Some((helper_path, helper_args)) = mount_wrapper {
            let mut argv: Vec<*const c_char> = helper_args.iter().map(|a| a.as_ptr()).collect();
            argv.push(std::ptr::null());
            let ret =
                bindings::krun_set_exec(ctx, helper_path.as_ptr(), argv.as_ptr(), env.as_ptr());
            if ret < 0 {
                println!("Error setting VM config");
                std::process::exit(-1);
            }
        } else if let Some(cmd) = cmd {
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
    }

    #[cfg(not(target_os = "macos"))]
    {
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
    }

    let ret = bindings::krun_start_enter(ctx);
    if ret < 0 {
        println!("Error starting VM");
        std::process::exit(-1);
    }
}

#[cfg(target_os = "macos")]
fn build_mount_wrapper(
    rootfs: &str,
    cmd: Option<&str>,
    args: &[CString],
    mounts: &[(String, String)],
) -> Option<(CString, Vec<CString>)> {
    if mounts.is_empty() {
        return None;
    }

    let helper_path = write_mount_script(rootfs, mounts);

    let mut exec_args: Vec<CString> = Vec::new();
    let command = cmd.unwrap_or("/bin/sh");
    exec_args.push(CString::new(command).unwrap());
    exec_args.extend(args.iter().cloned());

    let helper_cstr = CString::new(helper_path).unwrap();
    Some((helper_cstr, exec_args))
}

#[cfg(target_os = "macos")]
fn write_mount_script(rootfs: &str, mounts: &[(String, String)]) -> String {
    let host_path = format!("{}/.krunvm-mount.sh", rootfs);
    let guest_path = "/.krunvm-mount.sh".to_string();

    let mut file = File::create(&host_path).unwrap_or_else(|err| {
        println!("Error creating mount helper script: {}", err);
        std::process::exit(-1);
    });

    writeln!(file, "#!/bin/sh").unwrap();
    writeln!(file, "set -e").unwrap();
    for (tag, guest_path) in mounts {
        writeln!(file, "mount -t virtiofs {} {}", tag, guest_path).unwrap();
    }
    writeln!(file, "exec \"$@\"").unwrap();

    let perms = fs::Permissions::from_mode(0o755);
    if let Err(err) = fs::set_permissions(&host_path, perms) {
        println!("Error setting mount helper permissions: {}", err);
        std::process::exit(-1);
    }

    guest_path
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
