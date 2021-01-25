use std::ffi::{OsStr, OsString};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use directories::ProjectDirs;

use crate::cmd::{select, Cmd};
use crate::errors::DocumentError;

pub struct Defaults {
    dir: OsString,
    jobs: String,
}

impl Defaults {
    pub fn get() -> Result<Self, DocumentError> {
        let dir = match ProjectDirs::from("", "", "rfz") {
            Some(dirs) => dirs.data_dir().as_os_str().to_owned(),
            None => return Err(DocumentError::UserDirectories),
        };
        let jobs = num_cpus::get().to_string();
        Ok(Defaults { dir, jobs })
    }

    fn dir(&self) -> &OsStr {
        &self.dir
    }

    fn jobs(&self) -> &str {
        &self.jobs
    }
}

pub fn parse(defaults: &Defaults) -> Result<(Cmd, ArgMatches), DocumentError> {
    let matches = App::new("rfz")
        .version("0.1.0")
        .author("Ben Maddison")
        .about("Metadata extraction for IETF documents")
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .long("jobs")
                .takes_value(true)
                .global(true)
                .default_value(defaults.jobs())
                .help("Number of concurrent jobs to run"),
        )
        .arg(
            Arg::with_name("dir")
                .short("d")
                .long("dir")
                .takes_value(true)
                .global(true)
                .default_value_os(defaults.dir())
                .help("Directory containing IETF html docs"),
        )
        .subcommand(SubCommand::with_name("index").about(
            "List the latest version of each document \
             with associated metadata",
        ))
        .subcommand(
            SubCommand::with_name("summary")
                .about("Print a summary of the metadata in <doc>")
                .arg(
                    Arg::with_name("doc")
                        .required(true)
                        .help("Path to the document"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sync")
                .about("Syncronize the local document mirror")
                .arg(
                    Arg::with_name("remote")
                        .short("r")
                        .long("remote")
                        .default_value("rsync.tools.ietf.org::tools.html")
                        .help("Remote 'rsync' target to sync from"),
                )
                .arg(
                    Arg::with_name("command")
                        .long("command")
                        .default_value("rsync")
                        .help("Rsync command"),
                ),
        )
        .get_matches();
    match matches.subcommand() {
        (subcommand, Some(sub_matches)) => match select(subcommand) {
            Some(command) => Ok((command, sub_matches.to_owned())),
            None => Err(DocumentError::NotFound),
        },
        _ => Err(DocumentError::NotFound),
    }
}
