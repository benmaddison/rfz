extern crate clap;
extern crate directories;
extern crate kuchiki;
extern crate lazycell;
extern crate num_cpus;
extern crate pipeliner;

use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use directories::ProjectDirs;
use pipeliner::Pipeline;

mod document;
mod document_set;
mod errors;

use document::Document;
use document_set::Collection;
use errors::DocumentError;

fn main() -> Result<(), DocumentError> {
    let dirs = match ProjectDirs::from("", "", "rfz") {
        Some(dirs) => dirs,
        None => return Err(DocumentError::UserDirectories),
    };
    let cpus = num_cpus::get().to_string();
    let args = App::new("rfz")
        .version("0.1.0")
        .author("Ben Maddison")
        .about("Metadata extraction for IETF documents")
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .long("jobs")
                .takes_value(true)
                .global(true)
                .default_value(&cpus)
                .help(
                    "Number of concurrent jobs \
                                                                to run",
                ),
        )
        .arg(
            Arg::with_name("dir")
                .short("d")
                .long("dir")
                .takes_value(true)
                .global(true)
                .default_value_os(dirs.data_dir().as_os_str())
                .help("Directory containing IETF html docs"),
        )
        .setting(AppSettings::SubcommandRequired)
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
    let result = match args.subcommand() {
        ("index", Some(sub_args)) => index(sub_args),
        ("summary", Some(sub_args)) => summary(sub_args),
        ("sync", Some(sub_args)) => sync(sub_args),
        _ => Ok(()),
    };
    result
}

fn index(args: &ArgMatches) -> Result<(), DocumentError> {
    let jobs = usize::from_str(args.value_of("jobs").unwrap()).unwrap();
    let dir = PathBuf::from(args.value_of("dir").unwrap());
    let collection = match Collection::from_dir(dir) {
        Ok(set) => set,
        Err(e) => return Err(e),
    };
    let stdout = stdout();
    let mut stdout_writer = stdout.lock();
    for result in collection
        .to_map()?
        .newest(1)
        .with_threads(jobs)
        .map(|doc| doc.fmt_line())
    {
        match result {
            Ok(line) => {
                if writeln!(stdout_writer, "{}", line).is_err() {
                    return Ok(());
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn summary(args: &ArgMatches) -> Result<(), DocumentError> {
    let path = PathBuf::from(args.value_of("doc").unwrap());
    match Document::from_path(path) {
        Some(result) => match result {
            Ok(doc) => println!("{}", doc.fmt_summary()?),
            Err(e) => return Err(e),
        },
        None => return Err(DocumentError::NotFound),
    }
    Ok(())
}

fn sync(args: &ArgMatches) -> Result<(), DocumentError> {
    let dir = PathBuf::from(args.value_of("dir").unwrap());
    let command = args.value_of("command").unwrap();
    let remote = args.value_of("remote").unwrap();
    let status = Command::new(command)
        .arg("--archive")
        .arg("--compress")
        .arg("--progress")
        .arg("--include=*.html")
        .arg("--exclude=**")
        .arg("--prune-empty-dirs")
        .arg(remote)
        .arg(dir)
        .status();
    match status {
        Ok(_) => Ok(()),
        Err(e) => Err(DocumentError::SyncError(e)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub fn resource_path(name: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/resources");
        d.push(name);
        d
    }

    #[test]
    fn test_dummy() {
        assert_eq!(2+2, 4)
    }
}
