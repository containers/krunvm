// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

#[link(name = "krun")]
extern "C" {
    pub fn krun_set_log_level(level: u32) -> i32;
    pub fn krun_create_ctx() -> i32;
    pub fn krun_free_ctx(ctx: u32) -> i32;
    pub fn krun_set_vm_config(ctx: u32, num_vcpus: u8, ram_mib: u32) -> i32;
    pub fn krun_set_root(ctx: u32, root_path: *const i8) -> i32;
    pub fn krun_set_mapped_volumes(ctx: u32, mapped_volumes: *const *const i8) -> i32;
    pub fn krun_set_port_map(ctx: u32, port_map: *const *const i8) -> i32;
    pub fn krun_set_workdir(ctx: u32, workdir_path: *const i8) -> i32;
    pub fn krun_set_exec(
        ctx: u32,
        exec_path: *const i8,
        argv: *const *const i8,
        envp: *const *const i8,
    ) -> i32;
    pub fn krun_set_env(ctx: u32, envp: *const *const i8) -> i32;
    pub fn krun_start_enter(ctx: u32) -> i32;
}
