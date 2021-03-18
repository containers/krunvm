// Copyright 2021 Red Hat, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::{crate_version, App, Arg};

const APP_NAME: &str = "krunvm";

fn main() {
    let mut cfg: krunvm::KrunvmConfig = confy::load(APP_NAME).unwrap();

    let mut app = App::new("krunvm")
        .version(crate_version!())
        .author("Sergio Lopez <slp@redhat.com>")
        .about("Manage lightweight VMs created from OCI images")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            App::new("changevm")
                .about("Change the configuration of a lightweight VM")
                .arg(
                    Arg::with_name("cpus")
                        .long("cpus")
                        .help("Number of vCPUs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Amount of RAM in MiB")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("workdir")
                        .long("workdir")
                        .short("w")
                        .help("Working directory inside the lightweight VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("remove-volumes")
                        .long("remove-volumes")
                        .help("Remove all volume mappings"),
                )
                .arg(
                    Arg::with_name("volume")
                        .long("volume")
                        .short("v")
                        .help("Volume in form \"host_path:guest_path\" to be exposed to the guest")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("remove-ports")
                        .long("remove-ports")
                        .help("Remove all port mappings"),
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .help("Port in format \"host_port:guest_port\" to be exposed to the host")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("new-name")
                        .long("name")
                        .help("Assign a new name to the VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the VM to be modified")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("config")
                .about("Configure global values")
                .arg(
                    Arg::with_name("cpus")
                        .long("cpus")
                        .help("Default number of vCPUs for newly created VMs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Default amount of RAM in MiB for newly created VMs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dns")
                        .long("dns")
                        .help("DNS server to use in the lightweight VM")
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("create")
                .about("Create a new lightweight VM")
                .arg(
                    Arg::with_name("cpus")
                        .long("cpus")
                        .help("Number of vCPUs")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Amount of RAM in MiB")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dns")
                        .long("dns")
                        .help("DNS server to use in the lightweight VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("workdir")
                        .long("workdir")
                        .short("w")
                        .help("Working directory inside the lightweight VM")
                        .takes_value(true)
                        .default_value("/root"),
                )
                .arg(
                    Arg::with_name("volume")
                        .long("volume")
                        .short("v")
                        .help("Volume in form \"host_path:guest_path\" to be exposed to the guest")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .help("Port in format \"host_port:guest_port\" to be exposed to the host")
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("name")
                        .long("name")
                        .help("Assign a name to the VM")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("IMAGE")
                        .help("OCI image to use as template")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("delete")
                .about("Delete an existing lightweight VM")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the lightweight VM to be deleted")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            App::new("list").about("List lightweight VMs").arg(
                Arg::with_name("debug")
                    .short("d")
                    .help("print debug information verbosely"),
            ),
        )
        .subcommand(
            App::new("start")
                .about("Start an existing lightweight VM")
                .arg(Arg::with_name("cpus").long("cpus").help("Number of vCPUs"))
                .arg(
                    Arg::with_name("mem")
                        .long("mem")
                        .help("Amount of RAM in MiB"),
                )
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the lightweight VM")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("COMMAND")
                        .help("Command to run inside the VM")
                        .index(2)
                        .default_value("/bin/sh"),
                )
                .arg(
                    Arg::with_name("ARGS")
                        .help("Arguments to be passed to the command executed in the VM")
                        .multiple(true)
                        .last(true),
                ),
        );

    let matches = app.clone().get_matches();

    #[cfg(target_os = "macos")]
    krunvm::check_volume(&mut cfg);
    #[cfg(target_os = "linux")]
    krunvm::check_unshare();

    if let Some(ref matches) = matches.subcommand_matches("changevm") {
        krunvm::changevm::changevm(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("config") {
        krunvm::config::config(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("create") {
        krunvm::create::create(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("delete") {
        krunvm::delete::delete(&mut cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("list") {
        krunvm::list::list(&cfg, matches);
    } else if let Some(ref matches) = matches.subcommand_matches("start") {
        krunvm::start::start(&cfg, matches);
    } else {
        app.print_long_help().unwrap();
        println!();
    }
}
