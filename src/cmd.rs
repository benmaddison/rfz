use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use clap::ArgMatches;
use pipeliner::Pipeline;

use crate::document::Document;
use crate::document_set::Collection;
use crate::errors::DocumentError;

pub type Cmd = fn(&ArgMatches) -> Result<(), DocumentError>;

pub fn select(command: &str) -> Option<Cmd> {
    match command {
        "index" => Some(index),
        "summary" => Some(summary),
        "sync" => Some(sync),
        _ => None,
    }
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
