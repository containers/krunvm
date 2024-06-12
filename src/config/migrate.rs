use crate::config::{v1, v2};
use confy::ConfyError;

pub fn migrate_and_load_impl(
    load_v2: impl FnOnce() -> Result<v2::KrunvmConfig, ConfyError>,
    load_v1: impl FnOnce() -> Result<v1::KrunvmConfig, ConfyError>,
    save_v2: impl FnOnce(&v2::KrunvmConfig) -> Result<(), ()>,
) -> Result<v2::KrunvmConfig, ()> {
    fn check_version(got: u8, expected: u8) -> Result<(), ()> {
        if expected != got {
            eprintln!(
                "Invalid config version number {} expected {}",
                got, expected
            );
            Err(())
        } else {
            Ok(())
        }
    }

    let v2_load_err = match load_v2() {
        Ok(conf) => {
            check_version(conf.version, 2)?;
            return Ok(conf);
        }
        Err(e) => e,
    };

    let v1_load_err = match load_v1() {
        Ok(cfg) => {
            check_version(cfg.version, 1)?;
            let v2_config = cfg.into();
            save_v2(&v2_config)?;
            return Ok(v2_config);
        }
        Err(e) => e,
    };

    eprintln!("Failed to load config: ");
    eprintln!("Tried to load as as v2 config, got error: {v2_load_err}");
    eprintln!("Tried to load as as v1 config, got error: {v1_load_err}");
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NetworkMode;
    use std::collections::HashMap;

    #[test]
    fn load_without_migrate() {
        let cfg = v2::KrunvmConfig {
            default_dns: "8.8.8.8".into(),
            ..v2::KrunvmConfig::default()
        };

        let returned_cfg = migrate_and_load_impl(
            || Ok(cfg.clone()),
            || panic!("Loading v1 should not be attempted"),
            |_| panic!("Migration should not occur"),
        )
        .unwrap();
        assert_eq!(returned_cfg, cfg);
    }

    #[test]
    fn load_migrating_to_v2() {
        let v1_vms = [(
            "fedora".to_string(),
            v1::VmConfig {
                name: "fedora".to_string(),
                mapped_ports: Default::default(),
                cpus: 2,
                dns: "1.1.1.1".to_string(),
                mapped_volumes: Default::default(),
                workdir: "/".to_string(),
                container: "fedora".to_string(),
                mem: 8192,
            },
        )];

        let v1_cfg = v1::KrunvmConfig {
            default_dns: "8.8.8.8".into(),
            vmconfig_map: HashMap::from(v1_vms),
            ..v1::KrunvmConfig::default()
        };

        let result_v2_vms = [(
            "fedora".to_string(),
            v2::VmConfig {
                name: "fedora".to_string(),
                mapped_ports: Default::default(),
                cpus: 2,
                dns: "1.1.1.1".to_string(),
                mapped_volumes: Default::default(),
                workdir: "/".to_string(),
                container: "fedora".to_string(),
                mem: 8192,
                network_mode: NetworkMode::Tsi,
            },
        )];

        let result_v2_cfg = v2::KrunvmConfig {
            default_dns: "8.8.8.8".into(),
            vmconfig_map: HashMap::from(result_v2_vms),
            default_network_mode: NetworkMode::Tsi,
            ..v2::KrunvmConfig::default()
        };

        let mut load_v2_called = false;
        let mut load_v1_called = false;
        let mut save_called = false;

        let returned_cfg = migrate_and_load_impl(
            || {
                load_v2_called = true;
                Err(ConfyError::BadConfigDirectoryStr)
            },
            || {
                load_v1_called = true;
                Ok(v1_cfg)
            },
            |migrated| {
                save_called = true;
                assert_eq!(migrated, &result_v2_cfg);
                Ok(())
            },
        )
        .unwrap();

        assert!(load_v2_called, "Load v2 must be called");
        assert!(load_v1_called, "Load v1 must be called");
        assert!(save_called, "Save must be called");

        assert_eq!(returned_cfg, result_v2_cfg);
    }
}
