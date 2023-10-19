// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{KrunvmConfig, APP_NAME};
use clap::Args;

use super::utils::{remove_container, umount_container};

/// Delete an existing microVM
#[derive(Args, Debug)]
pub struct DeleteCmd {
    /// Name of the microVM to be deleted
    name: String,
}

impl DeleteCmd {
    pub fn run(self, cfg: &mut KrunvmConfig) {
        let vmcfg = match cfg.vmconfig_map.remove(&self.name) {
            None => {
                println!("No VM found with that name");
                std::process::exit(-1);
            }
            Some(vmcfg) => vmcfg,
        };

        umount_container(cfg, &vmcfg).unwrap();
        remove_container(cfg, &vmcfg).unwrap();

        confy::store(APP_NAME, &cfg).unwrap();
    }
}
