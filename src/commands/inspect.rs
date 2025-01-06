// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

use crate::{
    utils::{get_buildah_args, BuildahCommand},
    KrunvmConfig,
};
use clap::Args;

/// Run `buildah inspect` on an existing microVM
#[derive(Args, Debug)]
pub struct InspectCmd {
    /// Name of the microVM to be inspected
    name: String,
}

impl InspectCmd {
    pub fn run(self, cfg: &mut KrunvmConfig) {
        let vmcfg = match cfg.vmconfig_map.get(&self.name) {
            None => {
                println!("No VM found with that name");
                std::process::exit(-1);
            }
            Some(vmcfg) => vmcfg,
        };

        let mut args = get_buildah_args(cfg, BuildahCommand::Inspect);
        args.push(vmcfg.container.clone());

        let output = Command::new("buildah")
            .args(&args)
            .stderr(std::process::Stdio::inherit())
            .output();

        if output.is_err() {
            println!("Failed to inspect VM");
            std::process::exit(1);
        }

        let output = match String::from_utf8(output.unwrap().stdout) {
            Err(err) => {
                println!("Failed to parse `buildah inspect` output: #{err}.");
                std::process::exit(1);
            }
            Ok(output) => output,
        };

        println!("{output}");
    }
}
