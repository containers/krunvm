// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{ArgMatches, KrunvmConfig, APP_NAME};

use super::utils::{remove_container, umount_container};

pub fn delete(cfg: &mut KrunvmConfig, matches: &ArgMatches) {
    let name = matches.value_of("NAME").unwrap();

    let vmcfg = match cfg.vmconfig_map.remove(name) {
        None => {
            println!("No VM found with that name");
            std::process::exit(-1);
        }
        Some(vmcfg) => vmcfg,
    };

    umount_container(&cfg, &vmcfg).unwrap();
    remove_container(&cfg, &vmcfg).unwrap();

    confy::store(APP_NAME, &cfg).unwrap();
}
