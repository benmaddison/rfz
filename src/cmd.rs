use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;

use pipeliner::Pipeline;

use crate::document::Document;
use crate::document_set::Collection;
use crate::errors::{Error, Result};

pub trait ArgProvider {
    fn jobs(&self) -> usize;
    fn dir(&self) -> PathBuf;
    fn path(&self) -> PathBuf;
    fn rsync_cmd(&self) -> &str;
    fn rsync_remote(&self) -> &str;
}

pub type Cmd = fn(&dyn ArgProvider) -> Result<()>;

pub fn select(command: &str) -> Option<Cmd> {
    match command {
        "index" => Some(index),
        "summary" => Some(summary),
        "sync" => Some(sync),
        _ => None,
    }
}

fn index(args: &dyn ArgProvider) -> Result<()> {
    let collection = match Collection::from_dir(args.dir()) {
        Ok(set) => set,
        Err(e) => return Err(e),
    };
    let _stdout = stdout();
    #[cfg(not(test))]
    let mut writer = _stdout.lock();
    #[cfg(test)]
    let mut writer = std::io::sink();
    for result in collection
        .to_map()?
        .newest(1)
        .with_threads(args.jobs())
        .map(|doc| doc.fmt_line())
    {
        match result {
            Ok(line) => {
                if writeln!(writer, "{}", line).is_err() {
                    return Ok(());
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    Ok(())
}

fn summary(args: &dyn ArgProvider) -> Result<()> {
    match Document::from_path(args.path()) {
        Some(result) => match result {
            Ok(doc) => println!("{}", doc.fmt_summary()?),
            Err(e) => return Err(e),
        },
        None => {
            return Err(Error::DocumentNotFound(format!(
                "Failed to create a valid document from path '{:?}'",
                args.path()
            )))
        }
    }
    Ok(())
}

fn sync(args: &dyn ArgProvider) -> Result<()> {
    let status = Command::new(args.rsync_cmd())
        .arg("--archive")
        .arg("--compress")
        .arg("--progress")
        .arg("--include=*.html")
        .arg("--exclude=**")
        .arg("--prune-empty-dirs")
        .arg(args.rsync_remote())
        .arg(args.dir())
        .status();
    match status {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::SyncError(e)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test::resource_path;

    struct DummyArgs {
        jobs: Option<usize>,
        dir: Option<PathBuf>,
        path: Option<PathBuf>,
        rsync_cmd: Option<String>,
        rsync_remote: Option<String>,
    }

    impl ArgProvider for DummyArgs {
        fn jobs(&self) -> usize {
            self.jobs.unwrap()
        }
        fn dir(&self) -> PathBuf {
            self.dir.as_ref().unwrap().to_owned()
        }
        fn path(&self) -> PathBuf {
            self.path.as_ref().unwrap().to_owned()
        }
        fn rsync_cmd(&self) -> &str {
            self.rsync_cmd.as_ref().unwrap()
        }
        fn rsync_remote(&self) -> &str {
            self.rsync_remote.as_ref().unwrap()
        }
    }

    #[test]
    fn test_index_cmd() -> Result<()> {
        let args = DummyArgs {
            jobs: Some(2),
            dir: Some(resource_path("")),
            path: None,
            rsync_cmd: None,
            rsync_remote: None,
        };
        index(&args)
    }

    #[test]
    fn test_summary_cmd() -> Result<()> {
        let args = DummyArgs {
            jobs: None,
            dir: None,
            path: Some(resource_path("rfc6468.html")),
            rsync_cmd: None,
            rsync_remote: None,
        };
        summary(&args)
    }

    #[test]
    fn test_sync_cmd() -> Result<()> {
        let args = DummyArgs {
            jobs: None,
            dir: Some(resource_path("")),
            path: None,
            rsync_cmd: Some(String::from("/bin/true")),
            rsync_remote: Some(String::from("rsync.example.com::dummy")),
        };
        sync(&args)
    }
}
