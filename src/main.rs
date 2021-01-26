extern crate clap;
extern crate directories;
extern crate kuchiki;
extern crate lazycell;
extern crate num_cpus;
extern crate pipeliner;

mod cli;
mod cmd;
mod document;
mod document_set;
mod errors;

use cli::{Cli, Defaults};
use errors::DocumentError;

fn main() -> Result<(), DocumentError> {
    let defaults = Defaults::get()?;
    let cli = Cli::init(&defaults);
    cli.run()
}

#[cfg(test)]
mod test {
    pub fn resource_path(name: &str) -> std::path::PathBuf {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/resources");
        d.push(name);
        d
    }

    #[test]
    fn test_dummy() {
        assert_eq!(2 + 2, 4)
    }
}
