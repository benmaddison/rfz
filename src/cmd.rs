use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;

use pipeliner::Pipeline;

use crate::collection::Collection;
use crate::document::Document;
use crate::errors::{Error, Result};

pub trait ArgProvider {
    fn jobs(&self) -> usize;
    fn dir(&self) -> PathBuf;
    fn verbosity(&self) -> usize;
    fn path(&self) -> PathBuf;
    fn rsync_cmd(&self) -> &str;
    fn rsync_remote(&self) -> &str;
    fn types(&self) -> Option<Vec<&str>>;
}

type Cmd = fn(&dyn ArgProvider) -> Result<()>;

pub struct CmdExec<'a> {
    func: Cmd,
    args: &'a dyn ArgProvider,
}

impl<'a> CmdExec<'a> {
    pub fn init(command: &str, args: &'a dyn ArgProvider) -> Result<Self> {
        let func = match command {
            "index" => index,
            "summary" => summary,
            "sync" => sync,
            _ => {
                return Err(Error::ImplementationNotFound(format!(
                    "Failed to find an implementation for sub-command '{}'",
                    command
                )))
            }
        };
        Ok(CmdExec { func, args })
    }

    pub fn run(&self) -> Result<()> {
        (self.func)(self.args)
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
        .filter_types(args.types())
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
    let mut proc = Command::new(args.rsync_cmd());
    if args.verbosity() > 0 {
        proc.arg(format!("-{}", "v".repeat(args.verbosity())));
    }
    proc.arg("--archive")
        .arg("--compress")
        .arg("--include=*.html")
        .arg("--exclude=**")
        .arg("--prune-empty-dirs")
        .arg(args.rsync_remote())
        .arg(args.dir());
    let status = proc.status();
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
        verbosity: usize,
        path: Option<PathBuf>,
        rsync_cmd: Option<String>,
        rsync_remote: Option<String>,
        types: Option<Vec<&'static str>>,
    }

    impl ArgProvider for DummyArgs {
        fn jobs(&self) -> usize {
            self.jobs.unwrap()
        }
        fn dir(&self) -> PathBuf {
            self.dir.as_ref().unwrap().to_owned()
        }
        fn verbosity(&self) -> usize {
            self.verbosity.to_owned()
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
        fn types(&self) -> Option<Vec<&str>> {
            self.types.to_owned()
        }
    }

    #[test]
    fn test_index_cmd() -> Result<()> {
        let args = DummyArgs {
            jobs: Some(2),
            dir: Some(resource_path("")),
            verbosity: 0,
            path: None,
            rsync_cmd: None,
            rsync_remote: None,
            types: None,
        };
        let exec = CmdExec::init("index", &args)?;
        exec.run()
    }

    #[test]
    fn test_summary_cmd() -> Result<()> {
        let args = DummyArgs {
            jobs: None,
            dir: None,
            verbosity: 0,
            path: Some(resource_path("rfc6468.html")),
            rsync_cmd: None,
            rsync_remote: None,
            types: None,
        };
        let exec = CmdExec::init("summary", &args)?;
        exec.run()
    }

    #[test]
    fn test_sync_cmd() -> Result<()> {
        let args = DummyArgs {
            jobs: None,
            dir: Some(resource_path("")),
            verbosity: 2,
            path: None,
            rsync_cmd: Some(String::from("/bin/true")),
            rsync_remote: Some(String::from("rsync.example.com::dummy")),
            types: None,
        };
        let exec = CmdExec::init("sync", &args)?;
        exec.run()
    }

    #[test]
    fn test_not_implemented() {
        let args = DummyArgs {
            jobs: None,
            dir: None,
            verbosity: 0,
            path: None,
            rsync_cmd: None,
            rsync_remote: None,
            types: None,
        };
        match CmdExec::init("invalid", &args) {
            Err(Error::ImplementationNotFound(_)) => (),
            _ => panic!("Expected ImplementationNotFound error"),
        }
    }

    #[test]
    fn test_document_not_found() {
        let args = DummyArgs {
            jobs: None,
            dir: None,
            verbosity: 0,
            path: Some(resource_path("not-found")),
            rsync_cmd: None,
            rsync_remote: None,
            types: None,
        };
        let exec = CmdExec::init("summary", &args).unwrap();
        match exec.run() {
            Err(Error::DocumentNotFound(_)) => (),
            _ => panic!("Expected DocumentNotFound error"),
        }
    }
}
