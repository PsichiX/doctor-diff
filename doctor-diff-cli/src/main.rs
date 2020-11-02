use clap::{App, Arg, SubCommand};
use doctor_diff_core::patch::{patch_apply, patch_create, patch_request};
use std::path::PathBuf;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("patch")
                .arg(
                    Arg::with_name("workspace")
                        .help("Path to workspace directory")
                        .long("workspace")
                        .short("w")
                        .takes_value(true)
                        .default_value("."),
                )
                .subcommand(
                    SubCommand::with_name("request").arg(
                        Arg::with_name("hashes")
                            .help("Path to output hases file")
                            .long("h")
                            .short("h")
                            .required(true)
                            .takes_value(true),
                    ),
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .arg(
                            Arg::with_name("hashes")
                                .help("Path to input hashes file")
                                .long("h")
                                .short("h")
                                .required(true)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("archive")
                                .help("Path to output archive file")
                                .long("a")
                                .short("a")
                                .required(true)
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("apply").arg(
                        Arg::with_name("archive")
                            .help("Path to input archive file")
                            .long("a")
                            .short("a")
                            .required(true)
                            .takes_value(true),
                    ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("patch", Some(matches)) => {
            let workspace = PathBuf::from(matches.value_of("workspace").unwrap().to_owned());
            match matches.subcommand() {
                ("request", Some(matches)) => {
                    let hashes = PathBuf::from(matches.value_of("hashes").unwrap().to_owned());
                    patch_request(&workspace, &hashes).expect("Could not prepare patch request");
                }
                ("create", Some(matches)) => {
                    let hashes = PathBuf::from(matches.value_of("hashes").unwrap().to_owned());
                    let archive = PathBuf::from(matches.value_of("archive").unwrap().to_owned());
                    patch_create(&workspace, &hashes, &archive).expect("Could not create patch");
                }
                ("apply", Some(matches)) => {
                    let archive = PathBuf::from(matches.value_of("archive").unwrap().to_owned());
                    patch_apply(&workspace, &archive).expect("Could not apply patch");
                }
                _ => {}
            }
        }
        _ => {}
    }
}
