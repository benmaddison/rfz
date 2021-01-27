use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::result;
use std::str::FromStr;

use clap::{crate_authors, crate_description, crate_name, crate_version};
use directories::ProjectDirs;

use crate::cmd::{ArgProvider, CmdExec};
use crate::errors::{Error, Result};

pub trait DefaultsProvider {
    fn dir(&self) -> &OsStr;
    fn jobs(&self) -> &str;
}

pub struct Defaults {
    dir: OsString,
    jobs: String,
}

impl Defaults {
    pub fn get() -> Result<Self> {
        let dir = match ProjectDirs::from("", "", "rfz") {
            Some(dirs) => dirs.data_dir().as_os_str().to_owned(),
            None => {
                return Err(Error::UserDirectories(
                    "Failed to infer user directory locations".to_string(),
                ))
            }
        };
        let jobs = num_cpus::get().to_string();
        Ok(Defaults { dir, jobs })
    }
}

impl DefaultsProvider for Defaults {
    fn dir(&self) -> &OsStr {
        &self.dir
    }

    fn jobs(&self) -> &str {
        &self.jobs
    }
}

#[derive(Debug)]
pub struct Cli<'a>(clap::ArgMatches<'a>);

impl<'a> Cli<'a> {
    pub fn init(defaults: &'a dyn DefaultsProvider) -> Self {
        match Self::init_from(defaults, None) {
            Ok(cli) => cli,
            Err(e) => e.exit(),
        }
    }

    fn init_from(
        defaults: &'a dyn DefaultsProvider,
        argv: Option<Vec<&str>>,
    ) -> result::Result<Self, clap::Error> {
        let app = clap::app_from_crate!()
            .setting(clap::AppSettings::SubcommandRequired)
            .arg(
                clap::Arg::with_name("jobs")
                    .short("j")
                    .long("jobs")
                    .takes_value(true)
                    .global(true)
                    .default_value(defaults.jobs())
                    .help("Number of concurrent jobs to run"),
            )
            .arg(
                clap::Arg::with_name("dir")
                    .short("d")
                    .long("dir")
                    .takes_value(true)
                    .global(true)
                    .default_value_os(defaults.dir())
                    .help("Directory containing IETF html docs"),
            )
            .subcommand(clap::SubCommand::with_name("index").about(
                "List the latest version of each document \
                 with associated metadata",
            ))
            .subcommand(
                clap::SubCommand::with_name("summary")
                    .about("Print a summary of the metadata in <doc>")
                    .arg(
                        clap::Arg::with_name("doc")
                            .required(true)
                            .help("Path to the document"),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("sync")
                    .about("Syncronize the local document mirror")
                    .arg(
                        clap::Arg::with_name("remote")
                            .short("r")
                            .long("remote")
                            .default_value("rsync.tools.ietf.org::tools.html")
                            .help("Remote 'rsync' target to sync from"),
                    )
                    .arg(
                        clap::Arg::with_name("command")
                            .long("command")
                            .default_value("rsync")
                            .help("Rsync command"),
                    ),
            );
        let args = match argv {
            Some(argv) => app.get_matches_from_safe(argv),
            None => app.get_matches_safe(),
        };
        Ok(Cli(args?))
    }

    pub fn run(&self) -> Result<()> {
        match self.0.subcommand() {
            (subcommand, Some(sub_matches)) => {
                let args = CliArgs::from(sub_matches);
                let exec = CmdExec::init(subcommand, &args)?;
                exec.run()
            }
            _ => Err(Error::CliError("No sub-command was found".to_string())),
        }
    }
}

struct CliArgs<'a>(&'a clap::ArgMatches<'a>);

impl<'a> CliArgs<'a> {
    fn from(sub_matches: &'a clap::ArgMatches<'a>) -> Self {
        CliArgs(sub_matches)
    }
}

impl ArgProvider for CliArgs<'_> {
    fn jobs(&self) -> usize {
        usize::from_str(self.0.value_of("jobs").unwrap()).unwrap()
    }

    fn dir(&self) -> PathBuf {
        PathBuf::from(self.0.value_of("dir").unwrap())
    }

    fn path(&self) -> PathBuf {
        PathBuf::from(self.0.value_of("doc").unwrap())
    }

    fn rsync_cmd(&self) -> &str {
        self.0.value_of("command").unwrap()
    }

    fn rsync_remote(&self) -> &str {
        self.0.value_of("remote").unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test::resource_path;

    use std::str::FromStr;

    struct DummyDefaults;

    impl DefaultsProvider for DummyDefaults {
        fn jobs(&self) -> &str {
            "1"
        }
        fn dir(&self) -> &OsStr {
            OsStr::new("/home/foo/rfz")
        }
    }

    #[test]
    fn test_cli_defaults() -> Result<()> {
        let defaults = Defaults::get()?;
        assert!(usize::from_str(defaults.jobs()).unwrap() > 0);
        Ok(())
    }

    #[test]
    fn test_empty_args() {
        let defaults = DummyDefaults {};
        let argv = Some(vec!["rfz"]);
        match Cli::init_from(&defaults, argv) {
            Err(e) => assert_eq!(e.kind, clap::ErrorKind::MissingSubcommand),
            Ok(_) => panic!("Expected MissingSubcommand Error"),
        }
    }

    #[test]
    fn test_dummy_index() {
        let defaults = DummyDefaults {};
        let argv = Some(vec!["rfz", "index"]);
        let cli = Cli::init_from(&defaults, argv).unwrap();
        match cli.0.subcommand() {
            (subcommand, Some(args)) => {
                assert_eq!(subcommand, "index");
                let cli_args = CliArgs::from(args);
                assert_eq!(cli_args.jobs(), 1);
                assert_eq!(cli_args.dir(), PathBuf::from("/home/foo/rfz"));
            }
            _ => panic!("Cli parsing failed"),
        }
    }

    #[test]
    fn test_dummy_summary() {
        let defaults = DummyDefaults {};
        let argv = Some(vec!["rfz", "summary", "/home/foo/rfz/bar.html"]);
        let cli = Cli::init_from(&defaults, argv).unwrap();
        match cli.0.subcommand() {
            (subcommand, Some(args)) => {
                assert_eq!(subcommand, "summary");
                let cli_args = CliArgs::from(args);
                assert_eq!(cli_args.path(), PathBuf::from("/home/foo/rfz/bar.html"));
            }
            _ => panic!("Cli parsing failed"),
        }
    }

    #[test]
    fn test_dummy_sync() {
        let defaults = DummyDefaults {};
        let argv = Some(vec!["rfz", "sync"]);
        let cli = Cli::init_from(&defaults, argv).unwrap();
        match cli.0.subcommand() {
            (subcommand, Some(args)) => {
                assert_eq!(subcommand, "sync");
                let cli_args = CliArgs::from(args);
                assert_eq!(cli_args.rsync_cmd(), "rsync");
                assert_eq!(cli_args.rsync_remote(), "rsync.tools.ietf.org::tools.html");
            }
            _ => panic!("Cli parsing failed"),
        }
    }

    #[test]
    fn test_exec_index() -> Result<()> {
        let defaults = Defaults::get()?;
        let dir = resource_path("");
        let argv = Some(vec!["rfz", "index", "-d", dir.to_str().unwrap()]);
        let cli = Cli::init_from(&defaults, argv).unwrap();
        cli.run()
    }
}
